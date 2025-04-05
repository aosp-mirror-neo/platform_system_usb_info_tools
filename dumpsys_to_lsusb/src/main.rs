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

use anyhow::{anyhow, Result};
use framework_usb_protos_rs::enums::UsbEndPointDirection;
use framework_usb_protos_rs::enums::UsbEndPointType;
use framework_usb_protos_rs::usb::UsbDeviceProto;
use framework_usb_protos_rs::usb::UsbServiceDumpProto;
use protobuf::Message;
use std::io::stdout;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use typec_sysfs_helper::typec_class_utils::OutputWriter;

const INDENT_STEP: usize = 2;

/// Extracts the bus and device numbers from a USB path.
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

/// Prints USB device information in a format similar to `lsusb -v`.
///
/// The function iterates through the provided `devices` and prints detailed information about
/// each device, including its device descriptor, configuration descriptors, interface descriptors,
/// and endpoint descriptors. The output is formatted using the provided `OutputWriter`.lsusb
fn lsusb_v<W: Write>(devices: &[UsbDeviceProto], outp_writer: &mut OutputWriter<W>) -> Result<()> {
    for device in devices {
        let (busnum, devnum) = extract_bus_dev(&device.name.clone().unwrap_or_default());
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
                Some(device.class.map_or("0".to_string(), |s| s.to_string())),
                print_aligned_string_output,
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
                "num_configurations",
                Some(configs.len().to_string()),
                print_aligned_string_output::<W>,
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

/// Prints USB device information in a format similar to `lsusb -t`.
///
/// The function iterates through the provided `devices` and prints a tree-like representation
/// of the USB bus and device hierarchy.
fn lsusb_t() -> Result<()> {
    // TODO: implement
    Ok(())
}

fn main() -> Result<()> {
    logger::init(
        logger::Config::default()
            .with_tag_on_device("usb_info")
            .with_max_level(log::LevelFilter::Info),
    );

    let output = Command::new("dumpsys").arg("usb").arg("--proto").output()?;

    let Ok(usb_service_dump) = UsbServiceDumpProto::parse_from_bytes(&output.stdout) else {
        return Err(anyhow!("Cannot decode dumpsys usb data"));
    };

    let mut outp_writer = OutputWriter::new(stdout(), 0);
    let usb_host_manager = usb_service_dump.host_manager;
    lsusb_v(&usb_host_manager.devices, &mut outp_writer)?;

    // TODO: parametrize calling lsusb_v or lsusb_t.
    lsusb_t()?;

    Ok(())
}
