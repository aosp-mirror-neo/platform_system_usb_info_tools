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

//! usb_pd_utils

/// Contains information for decoding a specific piece of data from a
/// Vendor Data Object (VDO).
pub struct VdoField {
    pub index: i32,
    pub mask: u32,
    pub description: &'static str,
}

impl VdoField {
    /// Returns the "description: field_value" string for a VDO field.
    pub fn decode_vdo(&self, vdo: u32) -> String {
        let field_val: u32 = (vdo & self.mask) >> self.index;
        [self.description, &format!(": 0x{:x}", field_val)].concat()
    }
}

#[derive(Debug, PartialEq)]
pub enum PDRev {
    None,
    PD20,
    PD30,
    PD31,
}

/// Constants specific to USB PD revision 2.0.
pub mod pd20_data {
    use super::VdoField;

    // Vendor Data Objects (VDO) decoding information (source: USB PD spec).
    pub const ID_HEADER_VDO: &[VdoField] = &[
        VdoField { index: 0, mask: 0x0000ffff, description: "USB Vendor ID" },
        VdoField { index: 16, mask: 0x03ff0000, description: "Reserved" },
        VdoField { index: 26, mask: 0x04000000, description: "Modal Operation Supported" },
        VdoField { index: 27, mask: 0x38000000, description: "Product Type" },
        VdoField { index: 30, mask: 0x40000000, description: "USB Capable as a USB Device" },
        VdoField { index: 31, mask: 0x80000000, description: "USB Capable as a USB Host" },
    ];

    pub const CERT_STAT_VDO: &[VdoField] =
        &[VdoField { index: 0, mask: 0xffffffff, description: "XID" }];

    pub const PRODUCT_VDO: &[VdoField] = &[
        VdoField { index: 0, mask: 0x0000ffff, description: "bcdDevice" },
        VdoField { index: 16, mask: 0xffff0000, description: "USB Product ID" },
    ];
}

/// Constants specific to USB PD revision 3.0.
pub mod pd30_data {
    use super::VdoField;

    pub const ID_HEADER_VDO: &[VdoField] = &[
        VdoField { index: 0, mask: 0x0000ffff, description: "USB Vendor ID" },
        VdoField { index: 16, mask: 0x007f0000, description: "Reserved" },
        VdoField { index: 23, mask: 0x03800000, description: "Product Type (DFP)" },
        VdoField { index: 26, mask: 0x04000000, description: "Modal Operation Supported" },
        VdoField { index: 27, mask: 0x38000000, description: "Product Type (UFP/Cable Plug)" },
        VdoField { index: 30, mask: 0x40000000, description: "USB Capable as a USB Device" },
        VdoField { index: 31, mask: 0x80000000, description: "USB Capable as a USB Host" },
    ];

    pub const CERT_STAT_VDO: &[VdoField] =
        &[VdoField { index: 0, mask: 0xffffffff, description: "XID" }];

    pub const PRODUCT_VDO: &[VdoField] = &[
        VdoField { index: 0, mask: 0x0000ffff, description: "bcdDevice" },
        VdoField { index: 16, mask: 0xffff0000, description: "USB Product ID" },
    ];
}

/// Constants specific to USB PD revision 3.1.
pub mod pd31_data {
    use super::VdoField;

    pub const CERT_STAT_VDO: &[VdoField] =
        &[VdoField { index: 0, mask: 0xffffffff, description: "XID" }];

    pub const PRODUCT_VDO: &[VdoField] = &[
        VdoField { index: 0, mask: 0x0000ffff, description: "bcdDevice" },
        VdoField { index: 16, mask: 0xffff0000, description: "USB Product ID" },
    ];

    pub const ID_HEADER_VDO: &[VdoField] = &[
        VdoField { index: 0, mask: 0x0000ffff, description: "USB Vendor ID" },
        VdoField { index: 16, mask: 0x001f0000, description: "Reserved" },
        VdoField { index: 21, mask: 0x00600000, description: "Connector Type" },
        VdoField { index: 23, mask: 0x03800000, description: "Product Type (DFP)" },
        VdoField { index: 26, mask: 0x04000000, description: "Modal Operation Supported" },
        VdoField { index: 27, mask: 0x38000000, description: "Product Type (UFP/Cable Plug)" },
        VdoField { index: 30, mask: 0x40000000, description: "USB Capable as a USB Device" },
        VdoField { index: 31, mask: 0x80000000, description: "USB Capable as a USB Host" },
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_vdo() {
        let vdo: u32 = 0x12341234;
        let field = VdoField {
            index: 4,
            mask: 0x000000f0,
            description: "Value of the 2nd least significant byte",
        };
        assert_eq!(field.decode_vdo(vdo), "Value of the 2nd least significant byte: 0x3")
    }
}
