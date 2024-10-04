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

use anyhow::{Context, Result};
use log::warn;
use regex::Regex;
use std::{fs, io::Write, path::Path};

pub const TYPEC_SYSFS_PATH: &str = "/sys/class/typec";

pub const PORT_REGEX: &str = "^port[0-9]+$";

const ERR_READ_DIR: &str = "Failed to iterate over files in directory";
const ERR_FILE_NAME_FROM_PATH: &str = "Cannot get file name at path";
const ERR_FILE_NAME_UTF_INVALID: &str = "The file name is UTF-8 invalid at path";
const ERR_DIR_ENTRY_ACCESS: &str = "Cannot access entry in directory";
const ERR_FILE_TYPE_FROM_PATH: &str = "Cannot get the file type at path";
const ERR_WRITE_TO_BUFFER: &str = "Cannot write to output buffer. Attempted to print";

pub struct OutputWriter<W: Write> {
    buffer: W,
    indent: usize,
}

impl<W: Write> OutputWriter<W> {
    pub fn new(buffer: W, indent: usize) -> Self {
        Self { buffer, indent }
    }

    /// Prints a file's contents in a "name: content" format and also adds indentation to multiline strings.
    fn print_file_formatted(&mut self, file_path: &Path) -> Result<()> {
        let file_name = file_path
            .file_name()
            .with_context(|| format!("{ERR_FILE_NAME_FROM_PATH} {:?}", file_path))?
            .to_str()
            .with_context(|| format!("{ERR_FILE_NAME_UTF_INVALID} {:?}", file_path))?;
        let file_contents = fs::read_to_string(file_path)?;
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

    /// Prints all files in a directory in a "name: content" format.
    fn print_dir_files(&mut self, dir_path: &Path) -> Result<()> {
        let dir_name = dir_path
            .file_name()
            .with_context(|| format!("{ERR_FILE_NAME_FROM_PATH} {:?}", dir_path))?
            .to_str()
            .with_context(|| format!("{ERR_FILE_NAME_UTF_INVALID} {:?}", dir_path))?;
        writeln!(self.buffer, "{:indent$}{dir_name}", "", indent = self.indent)
            .with_context(|| format!("{ERR_WRITE_TO_BUFFER} {:?}", dir_name))?;

        for entry in
            fs::read_dir(dir_path).with_context(|| format!("{ERR_READ_DIR} {:?}", dir_path))?
        {
            let _ = entry
                .map(|entry| {
                    entry
                        .file_type()
                        .inspect_err(|err| {
                            warn!("{ERR_FILE_TYPE_FROM_PATH} {:?}: {err}", entry.path())
                        })
                        .is_ok_and(|file_type| file_type.is_file())
                        .then(|| {
                            self.indent += 2;
                            let _ = self
                                .print_file_formatted(&entry.path())
                                .inspect_err(|err| warn!("{:?}: {err}", &entry.path()));
                            self.indent -= 2;
                        });
                })
                .inspect_err(|err| warn!("{ERR_DIR_ENTRY_ACCESS} {:?}: {err}", dir_path));
        }
        Ok(())
    }
}

/// Prints relevant type-c connector class information for the port located at the sysfs path "port_path".
pub fn print_port_info<W: Write>(port_path: &Path, out_writer: &mut OutputWriter<W>) {
    let _ = out_writer.print_dir_files(port_path).inspect_err(|err| warn!("{:#}", err));
}

/// Looks at subdirectories of a given directory and executes a passed function
/// on directories matching a given regular expression.
pub fn parse_dirs_and_execute<W: Write>(
    dir_path: &Path,
    out_writer: &mut OutputWriter<W>,
    regex: &str,
    func: fn(&Path, &mut OutputWriter<W>),
) -> Result<()> {
    let re = Regex::new(regex).unwrap();
    for entry in fs::read_dir(dir_path).with_context(|| format!("{ERR_READ_DIR} {:?}", dir_path))? {
        let _ = entry
            .map(|entry| {
                entry
                    .file_type()
                    .inspect_err(|err| warn!("{ERR_FILE_TYPE_FROM_PATH} {:?}: {err}", entry.path()))
                    .is_ok_and(|file_type| file_type.is_dir() || file_type.is_symlink()) // Assumption: all the symlinks created by the Type-C connector class point to directories.
                    .then(|| {
                        entry
                            .file_name()
                            .to_str()
                            .is_some_and(|file_name| re.is_match(file_name))
                            .then(|| func(&entry.path(), out_writer));
                    })
            })
            .inspect_err(|err| warn!(" {:?}: {err}", dir_path));
    }
    Ok(())
}
