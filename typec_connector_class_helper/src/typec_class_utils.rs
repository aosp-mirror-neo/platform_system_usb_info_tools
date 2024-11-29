// Copyright 2024 The Android Open Source Project
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

//! typec_class_utils

use crate::usb_pd_utils::{pd20_data, pd30_data, pd31_data, PDRev, VdoField};
use anyhow::{bail, Context, Result};
use log::warn;
use regex::Regex;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub const TYPEC_SYSFS_PATH: &str = "/sys/class/typec";

pub const PORT_REGEX: &str = "^port[0-9]+$";

const ERR_READ_DIR: &str = "Failed to iterate over files in directory";
const ERR_READ_FILE: &str = "Cannot read file from path";
const ERR_FILE_NAME_FROM_PATH: &str = "Cannot get file name at path";
const ERR_FILE_NAME_UTF_INVALID: &str = "The file name is UTF-8 invalid at path";
const ERR_DIR_ENTRY_ACCESS: &str = "Cannot access entry in directory";
const ERR_WRITE_TO_BUFFER: &str = "Cannot write to output buffer. Attempted to print";
const ERR_PATH_NOT_DIR_OR_SYMLINK: &str = "The path is not a directory or a symbolic link";

pub struct OutputWriter<W: Write> {
    buffer: W,
    indent: usize,
}

impl<W: Write> OutputWriter<W> {
    pub fn new(buffer: W, indent: usize) -> Self {
        Self { buffer, indent }
    }

    /// Prints a string with indentation.
    fn print_str_indent(&mut self, str_to_print: &str) -> Result<()> {
        writeln!(self.buffer, "{:indent$}{str_to_print}", "", indent = self.indent)
            .with_context(|| format!("{ERR_WRITE_TO_BUFFER} {:?}", str_to_print))?;
        Ok(())
    }

    /// Prints a file's contents in a "name: content" format and also adds indentation to multiline strings.
    fn print_file_formatted(&mut self, file_path: &Path) -> Result<()> {
        let file_name = get_path_basename(file_path)?;
        let file_contents = fs::read_to_string(file_path)
            .with_context(|| format!("{ERR_READ_FILE} {:?}", file_path))?;
        let formatted_contents = file_contents
            .trim_ascii_end()
            .replace("\n", format!("\n{:indent$}", "", indent = self.indent).as_str());
        writeln!(
            self.buffer,
            "{:indent$}{file_name}: {formatted_contents}",
            "",
            indent = self.indent
        )
        .with_context(|| format!("{ERR_WRITE_TO_BUFFER} {:?}", formatted_contents))?;
        Ok(())
    }

    /// Prints the name of the directory followed by all the files in it in a "name: content" format.
    /// The files are sorted alphabetically by path to ensure consistency.
    pub fn print_dir_files(&mut self, dir_path: &Path) -> Result<()> {
        let dir_name = get_path_basename(dir_path)?;
        self.print_str_indent(dir_name)?;

        for path in get_sorted_paths_from_dir(dir_path, |path| path.is_file())? {
            self.indent += 2;
            let _ = self.print_file_formatted(&path).inspect_err(|err| warn!("{:?}: {err}", path));
            self.indent -= 2;
        }

        Ok(())
    }
}

/// Retrieves a sorted vector of paths in the given directory. Filters the paths according to `path_filter`.
/// Logs the errors and ignores faulty entries/paths.
fn get_sorted_paths_from_dir(
    dir_path: &Path,
    path_filter: fn(&PathBuf) -> bool,
) -> Result<Vec<PathBuf>> {
    let mut sorted_paths: Vec<PathBuf> = fs::read_dir(dir_path)
        .with_context(|| format!("{ERR_READ_DIR} {:?}", dir_path))?
        .filter_map(|entry| {
            entry
                .inspect_err(|err| warn!("{ERR_DIR_ENTRY_ACCESS} {:?}: {err}", dir_path))
                .map(|entry| entry.path())
                .ok()
        })
        .filter(path_filter)
        .collect();
    sorted_paths.sort();
    Ok(sorted_paths)
}

/// Returns the rightmost element of a path.
fn get_path_basename(path: &Path) -> Result<&str> {
    let basename = path
        .file_name()
        .with_context(|| format!("{ERR_FILE_NAME_FROM_PATH} {:?}", path))?
        .to_str()
        .with_context(|| format!("{ERR_FILE_NAME_UTF_INVALID} {:?}", path))?;
    Ok(basename)
}

/// Decodes and prints VDOs from the files specified relative to `identity_path_buf` path.
/// * `filename_vdo_fields_arr` - Is an array of tuples containing the name of the file where the VDO can be found and the VDO description array (`&[VdoField]`).
pub fn print_decoded_vdos_from_files<W: Write>(
    identity_path_buf: &mut PathBuf,
    filename_vdo_fields_arr: &[(&str, &[VdoField])],
    out_writer: &mut OutputWriter<W>,
) {
    for (filename, vdo_fields) in filename_vdo_fields_arr {
        identity_path_buf.push(filename);
        let _ = print_vdo(identity_path_buf, vdo_fields, out_writer)
            .inspect_err(|err| warn!("{:#}", err));
        identity_path_buf.pop();
    }
}

/// Prints the contents of an identity directory including VDO fields which are determined by product type.
pub fn print_partner_identity<W: Write>(
    partner_path: &Path,
    out_writer: &mut OutputWriter<W>,
) -> Result<()> {
    let mut identity_path_buf: PathBuf = partner_path.join("identity");

    if !(identity_path_buf.is_dir() | identity_path_buf.is_symlink()) {
        bail!("{ERR_PATH_NOT_DIR_OR_SYMLINK}: {:?}", identity_path_buf);
    }

    let _ = out_writer.print_str_indent("identity").inspect_err(|err| warn!("{:#}", err));

    out_writer.indent += 2;
    let filename_vdo_fields_arr: &[(&str, &[VdoField]); 3] = match get_pd_revision(partner_path) {
        PDRev::PD20 => &[
            ("id_header", pd20_data::ID_HEADER_VDO),
            ("cert_stat", pd20_data::CERT_STAT_VDO),
            ("product", pd20_data::PRODUCT_VDO),
        ],
        PDRev::PD30 => &[
            ("id_header", pd30_data::ID_HEADER_VDO),
            ("cert_stat", pd30_data::CERT_STAT_VDO),
            ("product", pd30_data::PRODUCT_VDO),
        ],
        PDRev::PD31 => &[
            ("id_header", pd31_data::ID_HEADER_VDO),
            ("cert_stat", pd31_data::CERT_STAT_VDO),
            ("product", pd31_data::PRODUCT_VDO),
        ],
        PDRev::None => &[("id_header", &[]), ("cert_stat", &[]), ("product", &[])],
    };
    print_decoded_vdos_from_files(&mut identity_path_buf, filename_vdo_fields_arr, out_writer);

    out_writer.indent -= 2;

    Ok(())
}

/// Reads the usb_power_delivery_revision file in a given directory
/// and returns a PDRev enum noting the supported PD Revision.
pub fn get_pd_revision(dir_path: &Path) -> PDRev {
    let pd_path: PathBuf = dir_path.join("usb_power_delivery_revision");
    let pd_revision_str = fs::read_to_string(&pd_path)
        .inspect_err(|err| warn!("{ERR_READ_FILE} {:?} {:#}", &pd_path, err))
        .unwrap_or_default();
    match pd_revision_str.trim_ascii() {
        "2.0" => PDRev::PD20,
        "3.0" => PDRev::PD30,
        "3.1" => PDRev::PD31,
        _ => PDRev::None,
    }
}

/// Reads a vdo value from a text file and converts it to a u32 variable. Then, prints out the values of each field according to the vdo_description.
pub fn print_vdo<W: Write>(
    vdo_path: &Path,
    vdo_description: &[VdoField],
    out_writer: &mut OutputWriter<W>,
) -> Result<()> {
    let vdo = read_vdo(vdo_path)?;
    let vdo_str = format!("{}: 0x{:x}", get_path_basename(vdo_path)?, vdo);
    let _ = out_writer.print_str_indent(&vdo_str).inspect_err(|err| warn!("{:#}", err));
    out_writer.indent += 2;
    for vdo_field in vdo_description {
        let _ = out_writer
            .print_str_indent(&vdo_field.decode_vdo(vdo))
            .inspect_err(|err| warn!("{:#}", err)); // log error but continue iterating.
    }
    out_writer.indent -= 2;
    Ok(())
}

/// Reads a file containing a 32 bit VDO value and loads it into a u32.
pub fn read_vdo(vdo_path: &Path) -> Result<u32> {
    let vdo_str =
        fs::read_to_string(vdo_path).with_context(|| format!("{ERR_READ_FILE} {:?}", vdo_path))?;
    let vdo_str = vdo_str.trim_ascii();
    let vdo: u32 =
        u32::from_str_radix(vdo_str.trim_start_matches("0x"), 16).with_context(|| {
            format!("Cannot convert string read from path: {:?} to u32: {vdo_str}", vdo_path)
        })?;
    Ok(vdo)
}

/// Prints the immediate information in the partner directory, then prints identity, alternate mode, and power data objects (PDO) information.
pub fn print_partner<W: Write>(port_path: &Path, out_writer: &mut OutputWriter<W>) -> Result<()> {
    let partner_dir_name = [get_path_basename(port_path)?, "-partner"].concat();
    let partner_path_buf: PathBuf = port_path.join(partner_dir_name);

    if !(partner_path_buf.is_dir() | partner_path_buf.is_symlink()) {
        bail!("{ERR_PATH_NOT_DIR_OR_SYMLINK}: {:?}", partner_path_buf);
    }

    let _ = out_writer.print_dir_files(&partner_path_buf).inspect_err(|err| warn!("{:#}", err));

    out_writer.indent += 2;
    let _ =
        print_partner_identity(&partner_path_buf, out_writer).inspect_err(|err| warn!("{:#}", err));
    out_writer.indent -= 2;
    Ok(())
}

/// Prints relevant type-c connector class information for the port located at the sysfs path "port_path".
pub fn print_port_info<W: Write>(port_path: &Path, out_writer: &mut OutputWriter<W>) {
    let _ = out_writer.print_dir_files(port_path).inspect_err(|err| warn!("{:#}", err));
    out_writer.indent += 2;
    let _ = print_partner(port_path, out_writer).inspect_err(|err| warn!("{:#}", err));
    out_writer.indent -= 2;
}

/// Looks at subdirectories of a given directory and executes a passed function on directories matching a given regular expression.
/// Directories are sorted alphabetically by path to ensure consistency in the order of processing.
pub fn parse_dirs_and_execute<W: Write>(
    dir_path: &Path,
    out_writer: &mut OutputWriter<W>,
    regex: &str,
    func: fn(&Path, &mut OutputWriter<W>),
) -> Result<()> {
    let re = Regex::new(regex).unwrap();
    for path in get_sorted_paths_from_dir(dir_path, |path| path.is_dir() | path.is_symlink())? {
        // Assumption: all the symlinks created by the Type-C connector class point to directories.
        get_path_basename(&path)
            .inspect_err(|err| warn!("{:#}", err))
            .is_ok_and(|file_name| re.is_match(file_name))
            .then(|| func(&path, out_writer));
    }
    Ok(())
}
