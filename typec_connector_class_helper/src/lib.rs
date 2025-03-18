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

//! Provides utilities for parsing and displaying information from the Type-C connector
//! class in the Linux kernel. It includes functions for reading and decoding VDOs, identifying
//! different types of devices, and printing information about ports, partners, cables, and plugs.

pub mod typec_class_utils;
pub mod usb_pd_utils;
