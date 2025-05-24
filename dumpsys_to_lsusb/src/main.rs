// Copyright 2025 The Android Open Source Project
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Parses `dumpsys usb` data from an Android device and outputs it in a format similar to `lsusb` verbose and tree (usbutils).
use anyhow::{anyhow, bail, Context, Result};
use clap::{Parser, ValueEnum};
use framework_usb_protos_rs::enums::UsbEndPointDirection;
use framework_usb_protos_rs::enums::UsbEndPointType;
use framework_usb_protos_rs::usb::UsbDeviceProto;
use framework_usb_protos_rs::usb::UsbServiceDumpProto;
use log::debug;
use protobuf::Message;
use std::collections::HashMap;
use std::fmt;
use std::fs::{self, read_link};
use std::io::stdout;
use std::io::Write;
use std::num::ParseIntError;
use std::path::{Path, PathBuf};
use std::process::Command;
use typec_sysfs_helper::typec_class_utils::OutputWriter;
use walkdir::WalkDir;

const INDENT_STEP: usize = 2;
const INDENT_STEP_TREE: usize = 4;
const USB_SYSFS_PATH_PREFIX: &str = "/sys/bus/usb/devices";
const TREE_ROOT_SYMBOL: &str = "/:  ";
const BRANCH_SYMBOL: &str = "|__";
const LEAF_KEY: &str = "";

/// Encapsulates the output format for the `dumpsys_to_lsusb` tool.
#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
enum OutputFormat {
    /// Prints detailed information about USB devices, their configurations, interfaces, and endpoints.
    Verbose,
    /// Prints a tree-like representation of the device hierarchy.
    Tree,
}

/// Defines the log levels that can be set via command-line arguments.
///
/// Reflects the log levels from log::LevelFilter.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum LogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for log::LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Off => log::LevelFilter::Off,
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Trace => log::LevelFilter::Trace,
        }
    }
}

#[derive(Parser, Debug)]
#[command(version = "1.0", about = "Display information about connected USB devices", long_about = None)]
struct Args {
    #[arg(value_enum, short, long, default_value_t = OutputFormat::Verbose)]
    format: OutputFormat,
    #[arg(value_enum, long, default_value_t = LogLevel::Info)]
    log_level: LogLevel,
}

/// Extracts the bus number and the device number from a USB path.
///
/// The function expects a path in the format `/dev/bus/usb/<bus>/<dev>`. If the path does not
/// match this format, it returns (0, 0). The USB path represents the device location in sysfs,
/// where the bus number is the bus on which the device is connected, and the device number is the
/// device's address on that bus.
fn extract_bus_dev(usb_path: &str) -> (u8, u8) {
    let path = Path::new(usb_path);
    if path.starts_with("/dev/bus/usb") {
        let path_comp: Vec<_> = path.components().collect();
        let bus_str = path_comp[path_comp.len() - 2].as_os_str().to_string_lossy();
        let dev_str = path_comp[path_comp.len() - 1].as_os_str().to_string_lossy();
        return (
            bus_str.parse::<u8>().unwrap_or_default(),
            dev_str.parse::<u8>().unwrap_or_default(),
        );
    }
    (0, 0)
}

/// Prints a key-value pair with aligned output, using the provided `OutputWriter`.
///
/// The value is printed as a string, and if the value is `None`, an empty string is printed. The
/// key is left-aligned to 20 characters, and the value follows.
fn print_aligned_string_output<W: Write>(
    key: &str,
    value: Option<String>,
    outp_writer: &mut OutputWriter<W>,
) -> Result<()> {
    outp_writer.print_str_indent(&format!("{:<20} {}", key, value.unwrap_or_default()))
}

/// Prints a key-value pair with aligned output, using the provided `OutputWriter`.
///
/// The value is printed as a hexadecimal number, and if the value is `None`, 0 is printed. The
/// key is left-aligned to 20 characters, and the value is a 4-digit hexadecimal number.
fn print_aligned_hex_output<W: Write>(
    key: &str,
    value: Option<String>,
    outp_writer: &mut OutputWriter<W>,
) -> Result<()> {
    outp_writer.print_str_indent(&format!(
        "{:<20} 0x{:04x}",
        key,
        value.unwrap_or_default().parse::<i32>().unwrap_or_default()
    ))
}

/// Holds the information needed to print a field of a structure.
///
/// The `key` is the name of the field, `value` is the value of the field, and `print_fn` is
/// the function that prints the key-value pair using an `OutputWriter`.struct PrintableStructField<'a, W: std::io::Write> {
struct PrintableStructField<'a, W: std::io::Write> {
    key: &'a str,
    value: Option<String>,
    print_fn: fn(&str, Option<String>, &mut OutputWriter<W>) -> Result<()>,
}

impl<'a, W: std::io::Write> PrintableStructField<'a, W> {
    fn new(
        key: &'a str,
        value: Option<String>,
        print_fn: fn(&str, Option<String>, &mut OutputWriter<W>) -> Result<()>,
    ) -> Self {
        PrintableStructField { key, value, print_fn }
    }
}

/// Prints a list of fields to the output writer.
fn print_struct_fields<W: Write>(
    fields_to_print: &[PrintableStructField<'_, W>],
    outp_writer: &mut OutputWriter<W>,
) -> Result<()> {
    fields_to_print.iter().try_for_each(|f| (f.print_fn)(f.key, f.value.clone(), outp_writer))
}

/// Finds the path to the device's sysfs directory.
///
/// The function uses the bus number and the device number to construct the path to the device's
/// sysfs directory. It iterates through all directories under the sysfs path prefix, and returns
/// the path to the directory whose devnum matches the provided devnum. If no matching directory
/// is found, it returns an error.
fn find_sysfs_usb_path(busnum: u8, devnum: u8) -> Result<PathBuf> {
    let path_root = format!("{USB_SYSFS_PATH_PREFIX}/usb{busnum}");
    let dir_walker = WalkDir::new(path_root.clone()).into_iter();
    for entry in dir_walker.filter_entry(|e| e.file_type().is_dir()) {
        let mut entry_path = entry?.into_path();
        entry_path.push("devnum");
        if let Ok(devnum_str) = fs::read_to_string(&entry_path) {
            if devnum_str.trim().parse::<u8>().unwrap_or_default() == devnum {
                entry_path.pop();
                return Ok(entry_path);
            }
        }
    }
    bail!("Cannot find sysfs path for device {busnum}, {devnum}, {path_root}")
}

/// Reads the USB device information from sysfs.
///
/// The function reads the USB device information from sysfs, which is the kernel's view of the
/// device. It uses the path to the device's sysfs directory and the file name to read the information.
/// The read value is trimmed.
fn read_sysfs_info(device_sysfs_path: &Path, file_name: &str) -> Result<String> {
    let file_path = device_sysfs_path.join(file_name);
    let sysfs_contents = fs::read_to_string(&file_path)
        .with_context(|| format!("Failed to read sysfs file: {:?}", file_path))?;
    Ok(sysfs_contents.trim().to_string())
}

/// Prints USB device information in a format similar to `lsusb -v`.
///
/// The function iterates through the provided `devices` and prints detailed information about
/// each device, including its device descriptor, configuration descriptors, interface descriptors,
/// and endpoint descriptors. The output is formatted using the provided `OutputWriter`.lsusb
fn lsusb_v<W: Write>(devices: &[UsbDeviceProto], outp_writer: &mut OutputWriter<W>) -> Result<()> {
    for device in devices {
        let (busnum, devnum) = extract_bus_dev(&device.name.clone().unwrap_or_default());
        let device_path: PathBuf = find_sysfs_usb_path(busnum, devnum)?;
        outp_writer.print_str_indent(&format!(
            "Bus {} Device {}: ID {:04x}:{:04x} {} {}",
            busnum,
            devnum,
            device.vendor_id.unwrap_or_default(),
            device.product_id.unwrap_or_default(),
            device.manufacturer_name.clone().unwrap_or_default(),
            device.product_name.clone().unwrap_or_default(),
        ))?;

        // Device info
        outp_writer.print_str_indent("Device Descriptor:")?;
        outp_writer.indent += INDENT_STEP;
        let configs = &device.configurations;
        let device_fields_to_print = &[
            PrintableStructField::new(
                "class",
                // TODO b/419018462: add mechanism to decode class based on https://www.usb.org/defined-class-codes
                Some(device.class.map_or("0".to_string(), |s| s.to_string())),
                print_aligned_hex_output,
            ),
            PrintableStructField::new(
                "subclass",
                Some(device.subclass.map_or("0".to_string(), |s| s.to_string())),
                print_aligned_string_output,
            ),
            PrintableStructField::new(
                "protocol",
                Some(device.protocol.map_or("0".to_string(), |s| s.to_string())),
                print_aligned_string_output,
            ),
            PrintableStructField::new(
                "serial_number",
                device.serial_number.clone(),
                print_aligned_string_output,
            ),
            PrintableStructField::new(
                "speed",
                read_sysfs_info(&device_path, "speed").map(|s| format!("{}M", s.trim())).ok(),
                print_aligned_string_output,
            ),
            PrintableStructField::new(
                "bcdUSB",
                Some(read_sysfs_info(&device_path, "version")?),
                print_aligned_string_output,
            ),
            PrintableStructField::new(
                "num_configurations",
                Some(configs.len().to_string()),
                print_aligned_string_output,
            ),
        ];
        print_struct_fields(device_fields_to_print, outp_writer)?;

        // Configurations info
        for config in configs {
            outp_writer.print_str_indent("Configuration Descriptor:")?;
            outp_writer.indent += INDENT_STEP;

            let interfaces = &config.interfaces;
            let config_fields_to_print = &[
                PrintableStructField::new(
                    "id",
                    config.id.map(|s| s.to_string()),
                    print_aligned_string_output,
                ),
                PrintableStructField::new("name", config.name.clone(), print_aligned_string_output),
                PrintableStructField::new(
                    "attributes",
                    config.attributes.map(|s| s.to_string()),
                    print_aligned_hex_output,
                ),
                PrintableStructField::new(
                    "max_power",
                    config.max_power.map(|s| s.to_string() + "mA"),
                    print_aligned_string_output,
                ),
                PrintableStructField::new(
                    "num_interfaces",
                    Some(config.interfaces.len().to_string()),
                    print_aligned_string_output,
                ),
            ];
            print_struct_fields(config_fields_to_print, outp_writer)?;

            // Interfaces info
            for interface in interfaces {
                outp_writer.print_str_indent("Interface Descriptor:")?;
                outp_writer.indent += INDENT_STEP;

                let endpoints = &interface.endpoints;
                let interface_fields_to_print = &[
                    PrintableStructField::new(
                        "id",
                        interface.id.map(|s| s.to_string()),
                        print_aligned_string_output,
                    ),
                    PrintableStructField::new(
                        "name",
                        interface.name.clone(),
                        print_aligned_string_output,
                    ),
                    PrintableStructField::new(
                        "alternate_settings",
                        interface.alternate_settings.map(|s| s.to_string()),
                        print_aligned_string_output,
                    ),
                    PrintableStructField::new(
                        "class",
                        interface.class.map(|s| s.to_string()),
                        print_aligned_string_output,
                    ),
                    PrintableStructField::new(
                        "subclass",
                        interface.subclass.map(|s| s.to_string()),
                        print_aligned_string_output,
                    ),
                    PrintableStructField::new(
                        "protocol",
                        interface.protocol.map(|s| s.to_string()),
                        print_aligned_string_output,
                    ),
                    PrintableStructField::new(
                        "num_endpoints",
                        Some(endpoints.len().to_string()),
                        print_aligned_string_output,
                    ),
                ];
                print_struct_fields(interface_fields_to_print, outp_writer)?;

                // Endpoints info
                for endp in endpoints {
                    outp_writer.print_str_indent("Endpoint Descriptor:")?;
                    outp_writer.indent += INDENT_STEP;
                    let endpoint_type: Option<UsbEndPointType> =
                        endp.type_.map(|e| e.enum_value_or_default());
                    let endpoint_direction: Option<UsbEndPointDirection> =
                        endp.direction.map(|e| e.enum_value_or_default());
                    let endp_fields_to_print = &[
                        PrintableStructField::new(
                            "endpoint_number",
                            endp.endpoint_number.map(|s| s.to_string()),
                            print_aligned_string_output,
                        ),
                        PrintableStructField::new(
                            "type",
                            endpoint_type.map(|e| format!("{:?}", e)),
                            print_aligned_string_output,
                        ),
                        PrintableStructField::new(
                            "address",
                            endp.address.map(|s| s.to_string()),
                            print_aligned_hex_output,
                        ),
                        PrintableStructField::new(
                            "direction",
                            endpoint_direction.map(|e| format!("{:?}", e)),
                            print_aligned_string_output,
                        ),
                        PrintableStructField::new(
                            "attributes",
                            endp.attributes.map(|s| s.to_string()),
                            print_aligned_string_output,
                        ),
                        PrintableStructField::new(
                            "max_packet_size",
                            endp.max_packet_size.map(|s| s.to_string()),
                            print_aligned_hex_output,
                        ),
                        PrintableStructField::new(
                            "interval",
                            endp.interval.map(|s| s.to_string()),
                            print_aligned_hex_output,
                        ),
                    ];
                    print_struct_fields(endp_fields_to_print, outp_writer)?;
                    outp_writer.indent -= INDENT_STEP;
                }
                outp_writer.indent -= INDENT_STEP;
            }
            outp_writer.indent -= INDENT_STEP;
        }
        outp_writer.indent -= INDENT_STEP;
    }
    Ok(())
}

/// Holds the representation of a device in the context of the tree command
#[derive(Debug)]
struct UsbDeviceReprTree {
    devnum: u8,
    interface: Option<u8>,
    configuration: Option<u8>,
    class: Option<i32>,
    driver: Option<String>,
    speed: Option<u32>,
}

impl fmt::Display for UsbDeviceReprTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output: String = format!("Dev {:03}", self.devnum);
        if let Some(cfg) = self.configuration {
            output.push_str(&format!(", Cfg {cfg}"));
        }
        if let Some(intf) = self.interface {
            output.push_str(&format!(", If {intf}"));
        }
        output.push_str(&format!(
            ", Class=0x{:04x}, Driver={}, {}M",
            self.class.unwrap_or_default(),
            self.driver.clone().unwrap_or_default(),
            self.speed.unwrap_or_default()
        ));
        write!(f, "{}", output)
    }
}

/// Organizes USB devices in a hierarchy using hash maps.
///
/// The `UsbDeviceHierarchy` is a map where keys are strings like "Bus 001" or "Port 001".
/// Each key maps to a `HierarchyNode`, which can be either:
///   - `Branch(UsbDeviceHierarchy)`: Represents a sub-level, i.e., a port with its sub-ports
///   - `Leaf(UsbDeviceReprTree)`: Contains the actual device information.
///
/// Conceptual example of the hierarchy this structure represents:
///
/// Bus 001: Branch (
/// ├── Port 001: Leaf(Dev 001, Class=root_hub, Driver=vhci_hcd/4p, 1000M)
/// ├── Port 002: Branch (
/// │   ├── Port 001: Leaf(Dev 003, Class=Audio, Driver=snd-usb-audio, 5000M)
/// │   └── Port 003: Leaf(Dev 005, Class=Audio, Driver=snd-usb-audio, 5000M)
/// |   )
/// └── Port 003: Leaf(Dev 004, Class=Vendor Specific Class, Driver=usbfs, 5000M)
/// )
/// Bus 002 Branch (
/// └── Port 001: Leaf(Dev 002, Class=root_hub, Driver=vhci_hcd/4p, 1000M)
/// )
///
/// Note: Internally, a `Leaf` node (device information) is stored under an empty
/// string key (`LEAF_KEY`) within a `Branch` node.
type UsbDeviceHierarchy = HashMap<String, HierarchyNode>;

/// Holds the information about a node in the USB device hierarchy.
///
/// The node can either be a branch (i.e., a sub-hierarchy) or a leaf (i.e., the device
/// information).
#[derive(Debug)]
enum HierarchyNode {
    Branch(UsbDeviceHierarchy),
    Leaf(UsbDeviceReprTree),
}

#[derive(Debug, PartialEq, Eq)]
enum UsbPathComponent {
    Bus(u8),
    Port(u8),
    Configuration(u8),
    Interface(u8),
}

/// Adds the information about a device to the hierarchy.
fn add_device_to_hierarchy(
    hierarchy: &mut UsbDeviceHierarchy,
    dev_path: &Path,
    mut dev_repr: UsbDeviceReprTree,
) -> Result<()> {
    let path_to_device = dev_path.file_name().and_then(|os_str| os_str.to_str()).unwrap_or("");

    // Traverse the path.
    // USB device path format: /sys/bus/usb/devices/<busnum>-<port[.port]>...:<config num>.<interface num>
    //
    // Symbols meaning:
    //     * "-" separates the bus nr. from the rest of the path (<bus_num>-<rest_of_path>)
    //     * "." separates either:
    //         * ports (<bus_num>.<port>.<port>.<port>)
    //         * config and if numbers (<config_num>.<if_num>)
    //     * ":" comes before a configuration number
    let mut path_components: Vec<UsbPathComponent> = Vec::new();

    // Split the path into a first part that starts with bus number and second that starts with config number by ":"
    let (bus_part, config_part) = match path_to_device.split_once(":") {
        Some((bus, config)) => (bus, Some(config)),
        _ => (path_to_device, None),
    };
    // Parse bus part, extract bus
    let dash_count = bus_part.matches("-").count();
    let (bus_num, ports) = match dash_count {
        1 => bus_part
            .split_once("-")
            .expect("Malformed device path, delimiter count was 1, split_once must succeed"),
        _ => bail!("Malformed device path, no bus number included"),
    };
    path_components.push(UsbPathComponent::Bus(bus_num.parse::<u8>()?));
    // Extract ports
    let mut port_components = ports
        .split('.')
        .map(|s| Ok::<UsbPathComponent, ParseIntError>(UsbPathComponent::Port(s.parse::<u8>()?)))
        .collect::<Result<Vec<_>, _>>()?;
    path_components.append(&mut port_components);

    // Parse config part
    if let Some(config_part) = config_part {
        let (conf_num, if_num) = config_part.split_once(".").expect("Malformed device path, configuration number and interface number should be split by a `.`");
        path_components.push(UsbPathComponent::Configuration(conf_num.parse::<u8>()?));
        path_components.push(UsbPathComponent::Interface(if_num.parse::<u8>()?));
    };

    // Add to hierarchy
    let mut hierarchy_ptr = hierarchy;
    for component in path_components {
        match component {
            UsbPathComponent::Bus(bus_num) => {
                let bus_key = format!("Bus {:03}", bus_num); // current key
                let next_node = hierarchy_ptr
                    .entry(bus_key)
                    .or_insert_with(|| HierarchyNode::Branch(HashMap::new()));
                if let HierarchyNode::Branch(next_hierarchy_ptr) = next_node {
                    // update the hierarchy pointer.
                    hierarchy_ptr = next_hierarchy_ptr;
                } else {
                    bail!("Found a device already at the same physical position. This should not happen.");
                }
            }
            UsbPathComponent::Port(port_num) => {
                let port_key = format!("Port {:03}", port_num);
                let next_node = hierarchy_ptr
                    .entry(port_key)
                    .or_insert_with(|| HierarchyNode::Branch(HashMap::new()));
                if let HierarchyNode::Branch(next_hierarchy_ptr) = next_node {
                    // update the hierarchy pointer.
                    hierarchy_ptr = next_hierarchy_ptr;
                } else {
                    bail!("Found a device already at the same physical position. This should not happen.");
                }
            }
            UsbPathComponent::Configuration(cfg_num) => {
                dev_repr.configuration = Some(cfg_num);
            }
            UsbPathComponent::Interface(if_num) => {
                dev_repr.interface = Some(if_num);
            }
        }
    }
    hierarchy_ptr.insert(LEAF_KEY.to_string(), HierarchyNode::Leaf(dev_repr));
    Ok(())
}

/// Prints one level of the device hierarchy and recursively calls itself for the next level.
fn print_device_hierarchy<W: Write>(
    hierarchy: &UsbDeviceHierarchy,
    outp_writer: &mut OutputWriter<W>,
) -> Result<()> {
    let mut keys_sorted: Vec<&String> = hierarchy.keys().collect();
    keys_sorted.sort_unstable();
    for key in keys_sorted {
        let prefix = match outp_writer.indent {
            0 => TREE_ROOT_SYMBOL,
            _ => BRANCH_SYMBOL,
        };
        match hierarchy.get(key).unwrap() {
            // unwrap is acceptable as if the key that has just been taken from the map has disappeared this is a sign of a serious issue.
            HierarchyNode::Branch(sub_hierarchy) => {
                // check if it is the node just above the leaf
                if sub_hierarchy.len() == 1 && sub_hierarchy.contains_key(LEAF_KEY) {
                    if let Some(HierarchyNode::Leaf(value)) = sub_hierarchy.get(LEAF_KEY) {
                        outp_writer.print_str_indent(&format!("{prefix}{}: {}", key, value))?;
                    } else {
                        bail!("Malformed hierarchy {:?}", hierarchy);
                    }
                } else {
                    outp_writer.print_str_indent(&format!("{prefix}{}", key))?;
                    outp_writer.indent += INDENT_STEP_TREE;
                    print_device_hierarchy(sub_hierarchy, outp_writer)?;
                    outp_writer.indent -= INDENT_STEP_TREE;
                }
            }
            HierarchyNode::Leaf(dev_repr) => {
                outp_writer.print_str_indent(&format!("{prefix}{}", dev_repr))?;
            }
        }
    }
    Ok(())
}

/// Prints USB device information in a format similar to `lsusb -t`.
///
/// The function iterates through the provided `devices` and prints a tree-like representation
/// of the USB bus and device hierarchy.
fn lsusb_t<W: Write>(devices: &[UsbDeviceProto], outp_writer: &mut OutputWriter<W>) -> Result<()> {
    let mut device_hierarchy: UsbDeviceHierarchy = HashMap::new();
    for device in devices {
        let (busnum, devnum) = extract_bus_dev(&device.name.clone().unwrap_or_default());
        let mut device_path: PathBuf = find_sysfs_usb_path(busnum, devnum)?;
        debug!("{:?}", device_path);
        let device_repr = UsbDeviceReprTree {
            devnum,
            interface: None,
            configuration: None,
            class: device.class,
            driver: {
                device_path.push("driver");
                let driver = read_link(&device_path).ok().and_then(|target_path: PathBuf| {
                    target_path.file_name().map(|os_str| os_str.to_string_lossy().into_owned())
                });
                device_path.pop();
                driver
            },
            speed: read_sysfs_info(&device_path, "speed")
                .map(|s| s.parse::<u32>().unwrap_or(0))
                .ok(),
        };
        add_device_to_hierarchy(&mut device_hierarchy, &device_path, device_repr)?;
    }
    debug!("Constructed device hierarchy: {:?}", device_hierarchy);
    print_device_hierarchy(&device_hierarchy, outp_writer)?;
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    logger::init(
        logger::Config::default()
            .with_tag_on_device("usb_info")
            .with_max_level(args.log_level.into()),
    );

    // Fetch dumpsys usb data.
    let output = Command::new("dumpsys").arg("usb").arg("--proto").output()?;
    let Ok(usb_service_dump) = UsbServiceDumpProto::parse_from_bytes(&output.stdout) else {
        return Err(anyhow!("Cannot decode dumpsys usb data"));
    };
    let usb_host_manager = usb_service_dump.host_manager;

    // Produce the output.
    let mut outp_writer = OutputWriter::new(stdout(), 0);
    match args.format {
        OutputFormat::Verbose => lsusb_v(&usb_host_manager.devices, &mut outp_writer)?,
        OutputFormat::Tree => lsusb_t(&usb_host_manager.devices, &mut outp_writer)?,
    }

    Ok(())
}
