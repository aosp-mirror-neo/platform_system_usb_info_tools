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

//! Provides utilities for decoding USB Power Delivery (PD) data,
//! including Vendor Data Objects (VDOs) and their fields.

/// Contains information for decoding a specific piece of data from a
/// Vendor Data Object (VDO). A VDO is a 32-bit value that contains
/// information about a device or cable, and is used in USB PD
/// communication.
pub struct VdoField {
    /// index
    pub index: i32,
    /// mask
    pub mask: u32,
    /// description
    pub description: &'static str,
}

impl VdoField {
    /// Returns the "description: field_value" string for a VDO field.
    pub fn decode_vdo(&self, vdo: u32) -> String {
        let field_val: u32 = (vdo & self.mask) >> self.index;
        [self.description, &format!(": 0x{:x}", field_val)].concat()
    }
}

// Masks for id_header fields.
/// UFP_PRODUCT_TYPE_MASK
///
/// ref: USB Power Delivery R2.0 V1.3, Section 6.4.4.3.1.1 ID Header VDO, Table 6-23 ID Header VDO
pub const UFP_PRODUCT_TYPE_MASK: u32 = 0x38000000;
/// DFP_PRODUCT_TYPE_MASK
///
/// ref: USB Power Delivery R3.0 V2.0, Section 6.4.4.3.1.1 ID Header VDO, Table 6-29 ID Header VDO
pub const DFP_PRODUCT_TYPE_MASK: u32 = 0x03800000;

/// Represents the USB Power Delivery (PD) revision.
#[derive(Debug, PartialEq)]
pub enum PdRev {
    /// Represents an unknown or unspecified PD revision.
    None,
    /// Represents the PD 2.0 revision.
    Pd20,
    /// Represents the PD 3.0 revision.
    Pd30,
    /// Represents the PD 3.1 revision.
    Pd31,
}

/// Represents the type of cable.
///
/// ref: USB Power Delivery R2.0 V1.3, Section 6.4.4.3.1.1.4 Product Type (Cable Plug), Table 6-25 Product Types (Cable Plug)
#[derive(Debug, PartialEq)]
pub enum CableType {
    /// Represents a passive cable.
    ///
    /// ref: USB Power Delivery R2.0 V1.3, Section 1.6 Terms and Abbreviations, Table 1-1 Terms and Abbreviations, Row: Passive Cable
    Passive,
    /// Represents an active cable.
    ///
    /// ref: USB Power Delivery R2.0 V1.3, Section 1.6 Terms and Abbreviations, Table 1-1 Terms and Abbreviations, Row: Active Cable
    Active,
    /// Represents a VCONN Powered USB Device (VPD).
    ///
    /// ref: USB Power Delivery R3.0 V2.0, Section 1.6 Terms and Abbreviations, Table 1-1 Terms and Abbreviations, VCONN Powered USB Device (VPD)
    Vpd,
}

/// Represents the type of port.
///
/// ref: USB Power Delivery R2.0 V1.3
#[derive(Debug, PartialEq)]
pub enum PortType {
    /// Represents a USB-PD Upstream Facing Port.
    ///
    /// ref: USB Power Delivery R2.0 V1.3, Section 1.6 Terms and Abbreviations, Table 1-1 Terms and Abbreviations, Row: Upstream Facing Port (UFP)
    Ufp,
    /// Represents a USB-PD Downstream Facing Port.
    ///
    /// ref: USB Power Delivery R2.0 V1.3, Section 1.6 Terms and Abbreviations, Table 1-1 Terms and Abbreviations, Row: Downstream Facing Port (DFP)
    Dfp,
    /// Represents a USB-PD Dual-Role Data Port.
    ///
    /// ref: USB Power Delivery R2.0 V1.3, Section 1.6 Terms and Abbreviations, Table 1-1 Terms and Abbreviations, Row: Dual-Role Data (DRD)
    Drd,
}

/// Represents the type of product.
///
/// ref: USB Power Delivery R2.0 V1.3
#[derive(Debug, PartialEq)]
pub enum ProductType {
    /// Represents a cable, including its cable type and PD revision.
    Cable((CableType, PdRev)),
    /// Represents an Alternate Mode Adapter (AMA), including its PD revision.
    ///
    /// ref: USB Power Delivery R2.0 V1.3, Section 1.6 Terms and Abbreviations, Table 1-1 Terms and Abbreviations, Row: Alternate Mode Adapter (AMA)
    Ama(PdRev),
    /// Represents a VCONN Powered USB Device (VPD), including its PD revision.
    ///
    /// ref: TODO 3.0
    Vpd(PdRev),
    /// Represents a port, including its port type and PD revision.
    ///
    /// ref: USB Power Delivery R2.0 V1.3, Section 1.6 Terms and Abbreviations, Table 1-1 Terms and Abbreviations, Row: Port
    Port((PortType, PdRev)),
    /// Represents a product of an unspecified or unknown type.
    Other,
}

/// Constants specific to USB PD revision 2.0.
///
/// ref: USB Power Delivery R2.0 V1.3
pub mod pd20_data {
    use super::VdoField;

    // Expected product type identifiers extracted from id_header VDO - both UFP and Cable Plug types.
    /// Indicates a product is an Alternate Mode Adapter
    ///
    /// ref: USB Power Delivery R2.0 V1.3, Section 6.4.4.3.1.1 ID Header VDO, Table 6-23 ID Header VDO, Row: B29..27
    pub const AMA_COMP: u32 = 0x28000000;

    /// Indicates a product is a passive cable
    ///
    /// ref: USB Power Delivery R2.0 V1.3, Section 6.4.4.3.1.1 ID Header VDO, Table 6-23 ID Header VDO, Row: B29..27
    pub const PASSIVE_CABLE_COMP: u32 = 0x18000000;

    /// Indicates a product is an active cable
    ///
    /// ref: USB Power Delivery R2.0 V1.3, Section 6.4.4.3.1.1 ID Header VDO, Table 6-23 ID Header VDO, Row: B29..27
    pub const ACTIVE_CABLE_COMP: u32 = 0x20000000;

    // TODO: add the PDUSB HUB and PDUSB Peripheral product types
    // ref: USB Power Delivery R2.0 V1.3, Section 6.4.4.3.1.1 ID Header VDO, Table 6-23 ID Header VDO, Row: B29..27

    // Vendor Data Objects (VDO) decoding information
    /// Contains information about the product, including its vendor ID, product type, and USB capabilities.
    ///
    /// ref: USB Power Delivery R2.0 V1.3, Section 6.4.4.3.1.1 ID Header VDO, Table 6-23 ID Header VDO
    pub const ID_HEADER_VDO: &[VdoField] = &[
        VdoField { index: 0, mask: 0x0000ffff, description: "USB Vendor ID" },
        VdoField { index: 16, mask: 0x03ff0000, description: "Reserved" },
        VdoField { index: 26, mask: 0x04000000, description: "Modal Operation Supported" },
        VdoField { index: 27, mask: 0x38000000, description: "Product Type" },
        VdoField { index: 30, mask: 0x40000000, description: "USB Capable as a USB Device" },
        VdoField { index: 31, mask: 0x80000000, description: "USB Capable as a USB Host" },
    ];

    /// Contains information about the certification status of a product.
    ///
    /// ref: USB Power Delivery R2.0 V1.3, Section 6.4.4.3.1.2 Cert Stat VDO, Table 6-26 Cert Stat VDO
    pub const CERT_STAT_VDO: &[VdoField] =
        &[VdoField { index: 0, mask: 0xffffffff, description: "XID" }]; // XID assigned by USB-IF

    /// Contains information about the product, including its bcdDevice and USB Product ID.
    ///
    /// ref: USB Power Delivery R2.0 V1.3, Section 6.4.4.3.1.3 Product VDO, Table 6-27 Product VDO
    pub const PRODUCT_VDO: &[VdoField] = &[
        VdoField { index: 0, mask: 0x0000ffff, description: "bcdDevice" },
        VdoField { index: 16, mask: 0xffff0000, description: "USB Product ID" },
    ];

    /// Contains information about a passive cable.
    ///
    /// ref: USB Power Delivery R2.0 V1.3, Section 6.4.4.3.1.4.1 Passive Cable VDO, Table 6-28 Passive Cable VDO
    pub const PASSIVE_VDO: &[VdoField] = &[
        VdoField { index: 0, mask: 0x00000007, description: "USB Speed" },
        VdoField { index: 3, mask: 0x00000008, description: "Reserved" },
        VdoField { index: 4, mask: 0x00000010, description: "Vbus Through Cable" },
        VdoField { index: 5, mask: 0x00000060, description: "Vbus Current Handling" },
        VdoField { index: 7, mask: 0x00000080, description: "SSRX2 Directionality Support" },
        VdoField { index: 8, mask: 0x00000100, description: "SSRX1 Directionality Support" },
        VdoField { index: 9, mask: 0x00000200, description: "SSTX2 Directionality Support" },
        VdoField { index: 10, mask: 0x00000400, description: "SSTX1 Directionality Support" },
        VdoField { index: 11, mask: 0x00001800, description: "Cable Termination Type" },
        VdoField { index: 13, mask: 0x0001e000, description: "Cable Latency" },
        VdoField { index: 17, mask: 0x00020000, description: "Reserved" },
        VdoField { index: 18, mask: 0x000c0000, description: "USB Type-C Plug to USB Type" },
        VdoField { index: 20, mask: 0x00f00000, description: "Reserved" },
        VdoField { index: 24, mask: 0x0f000000, description: "Firmware Version" },
        VdoField { index: 28, mask: 0xf0000000, description: "HW Version" },
    ];

    /// Contains information about an active cable.
    ///
    /// ref: USB Power Delivery R2.0 V1.3, Section 6.4.4.3.1.4.2 Active Cable VDO, Table 6-29 Active Cable VDO
    pub const ACTIVE_VDO: &[VdoField] = &[
        VdoField { index: 0, mask: 0x00000007, description: "USB Speed" },
        VdoField { index: 3, mask: 0x00000008, description: "SOP'' Controller Present" },
        VdoField { index: 4, mask: 0x00000010, description: "Vbus Through Cable" },
        VdoField { index: 5, mask: 0x00000060, description: "Vbus Current Handling" },
        VdoField { index: 7, mask: 0x00000080, description: "SSRX2 Directionality Support" },
        VdoField { index: 8, mask: 0x00000100, description: "SSRX1 Directionality Support" },
        VdoField { index: 9, mask: 0x00000200, description: "SSTX2 Directionality Support" },
        VdoField { index: 10, mask: 0x00000400, description: "SSTX1 Directionality Support" },
        VdoField { index: 11, mask: 0x00001800, description: "Cable Termination Type" },
        VdoField { index: 13, mask: 0x0001e000, description: "Cable Latency" },
        VdoField { index: 17, mask: 0x00020000, description: "Reserved" },
        VdoField { index: 18, mask: 0x000c0000, description: "USB Type-C Plug to USB Type" },
        VdoField { index: 20, mask: 0x00f00000, description: "Reserved" },
        VdoField { index: 24, mask: 0x0f000000, description: "Firmware Version" },
        VdoField { index: 28, mask: 0xf0000000, description: "HW Version" },
    ];

    /// Contains information about an Alternate Mode Adapter (AMA).
    ///
    /// ref: USB Power Delivery R2.0 V1.3, Section 6.4.4.3.1.5 Alternate Mode Adapter VDO, Table 6-30 AMA VDO
    pub const AMA_VDO: &[VdoField] = &[
        VdoField { index: 0, mask: 0x00000007, description: "USB SS Signaling Support" },
        VdoField { index: 3, mask: 0x00000008, description: "Vbus Required" },
        VdoField { index: 4, mask: 0x00000010, description: "Vconn Required" },
        VdoField { index: 5, mask: 0x000000e0, description: "Vconn Power" },
        VdoField { index: 8, mask: 0x00000100, description: "SSRX2 Directionality Support" },
        VdoField { index: 9, mask: 0x00000200, description: "SSRX1 Directionality Support" },
        VdoField { index: 10, mask: 0x00000400, description: "SSTX2 Directionality Support" },
        VdoField { index: 11, mask: 0x00000800, description: "SSTX1 Directionality Support" },
        VdoField { index: 12, mask: 0x00fff000, description: "Reserved" },
        VdoField { index: 24, mask: 0x0f000000, description: "Firmware Version" },
        VdoField { index: 28, mask: 0xf0000000, description: "Hardware Version" },
    ];

    // TODO: Decode USB SuperSpeed Signaling Support ref: USB Power Delivery R2.0 V1.3, Section 6.4.4.3.1.5 Alternate Mode Adapter VDO, Table 6-30 AMA VDO, Row B2..0
}

/// Constants specific to USB PD revision 3.0.
///
/// ref: USB Power Delivery R3.0 V2.0
pub mod pd30_data {
    use super::VdoField;

    /// Indicates a product is an Alternate Mode Adapter
    ///
    /// ref: USB Power Delivery R3.0 V2.0, Section 6.4.4.3.1.1 ID Header VDO, Table 6-29 ID Header VDO, Row: B29..27
    pub const AMA_COMP: u32 = 0x28000000;
    /// Indicates a product is a VCONN Powered USB Device
    ///
    /// ref: USB Power Delivery R3.0 V2.0, Section 6.4.4.3.1.1 ID Header VDO, Table 6-29 ID Header VDO, Row: B29..27
    pub const VPD_COMP: u32 = 0x30000000;
    ///  Indicates a product is a PDUSB Hub's Upstream Facing Port
    ///
    /// ref: USB Power Delivery R3.0 V2.0, Section 6.4.4.3.1.1 ID Header VDO, Table 6-29 ID Header VDO, Row: B29..27
    pub const HUB_COMP: u32 = 0x08000000;
    /// Indicates a product is a PDUSB Peripheral
    ///
    /// ref: USB Power Delivery R3.0 V2.0, Section 6.4.4.3.1.1 ID Header VDO, Table 6-29 ID Header VDO, Row: B29..27
    pub const PERIPHERAL_COMP: u32 = 0x10000000;
    /// Indicates a product is a PDUSB Hub's Downstream Facing Port
    ///
    /// ref: USB Power Delivery R3.0 V2.0, Section 6.4.4.3.1.1 ID Header VDO, Table 6-29 ID Header VDO, Row: B25..23
    pub const DFP_HUB_COMP: u32 = 0x00800000;
    /// Indicates a product is a PDUSB Host's Downstream Facing Port
    ///
    /// ref: USB Power Delivery R3.0 V2.0, Section 6.4.4.3.1.1 ID Header VDO, Table 6-29 ID Header VDO, Row: B25..23
    pub const DFP_HOST_COMP: u32 = 0x01000000;
    /// Indicates a product is a Power Brick
    ///
    /// ref: USB Power Delivery R3.0 V2.0, Section 6.4.4.3.1.1 ID Header VDO, Table 6-29 ID Header VDO, Row: B25..23
    pub const POWER_BRICK_COMP: u32 = 0x01800000;

    // Expected id_header field results.
    /// Indicates a product is a passive cable
    ///
    /// ref: USB Power Delivery R3.0 V2.0, Section 6.4.4.3.1.1 ID Header VDO, Table 6-29 ID Header VDO, Row: B29..27
    pub const PASSIVE_CABLE_COMP: u32 = 0x18000000;
    /// Indicates a product is an active cable
    ///
    /// ref: USB Power Delivery R3.0 V2.0, Section 6.4.4.3.1.1 ID Header VDO, Table 6-29 ID Header VDO, Row: B29..27
    pub const ACTIVE_CABLE_COMP: u32 = 0x20000000;

    /// Contains information about the product, including its vendor ID, product type, and USB capabilities.
    ///
    /// ref: USB Power Delivery R3.0 V2.0, Section 6.4.4.3.1.1 ID Header VDO, Table 6-29 ID Header VDO
    pub const ID_HEADER_VDO: &[VdoField] = &[
        VdoField { index: 0, mask: 0x0000ffff, description: "USB Vendor ID" },
        VdoField { index: 16, mask: 0x007f0000, description: "Reserved" },
        VdoField { index: 23, mask: 0x03800000, description: "Product Type (DFP)" },
        VdoField { index: 26, mask: 0x04000000, description: "Modal Operation Supported" },
        VdoField { index: 27, mask: 0x38000000, description: "Product Type (UFP/Cable Plug)" },
        VdoField { index: 30, mask: 0x40000000, description: "USB Capable as a USB Device" },
        VdoField { index: 31, mask: 0x80000000, description: "USB Capable as a USB Host" },
    ];

    /// Contains information about the certification status of a product.
    ///
    /// ref: USB Power Delivery R3.0 V2.0, Section 6.4.4.3.1.2 Cert Stat VDO, Table 6-33 Cert Stat VDO
    pub const CERT_STAT_VDO: &[VdoField] =
        &[VdoField { index: 0, mask: 0xffffffff, description: "XID" }];

    /// Contains information about the product, including its bcdDevice and USB Product ID.
    ///
    /// ref: USB Power Delivery R3.0 V2.0, Section 6.4.4.3.1.3 Product VDO , Table 6-34 Product VDO
    pub const PRODUCT_VDO: &[VdoField] = &[
        VdoField { index: 0, mask: 0x0000ffff, description: "bcdDevice" },
        VdoField { index: 16, mask: 0xffff0000, description: "USB Product ID" },
    ];

    /// Contains information about an Alternate Mode Adapter (AMA).
    ///
    /// ref: USB Power Delivery R3.0 V2.0, Section 6.4.4.3.1.8 Alternate Mode Adapter VDO , Table 6-41 AMA VDO
    pub const AMA_VDO: &[VdoField] = &[
        VdoField { index: 0, mask: 0x00000007, description: "USB Highest Speed" },
        VdoField { index: 3, mask: 0x00000008, description: "Vbus Required" },
        VdoField { index: 4, mask: 0x00000010, description: "Vconn Required" },
        VdoField { index: 5, mask: 0x000000e0, description: "Vconn Power" },
        VdoField { index: 8, mask: 0x001fff00, description: "Reserved" },
        VdoField { index: 21, mask: 0x00e00000, description: "VDO Version" },
        VdoField { index: 24, mask: 0x0f000000, description: "Firmware Version" },
        VdoField { index: 28, mask: 0xf0000000, description: "Hardware Version" },
    ];

    /// Contains information about a VCONN Powered USB Device (VPD).
    ///
    /// ref: USB Power Delivery R3.0 V2.0, Section 6.4.4.3.1.9 Vconn Powered USB Device VDO, Table 6-42 VPD VDO
    pub const VPD_VDO: &[VdoField] = &[
        VdoField { index: 0, mask: 0x00000001, description: "Charge Through Support" },
        VdoField { index: 1, mask: 0x0000007e, description: "Ground Impedance" },
        VdoField { index: 7, mask: 0x00001f80, description: "Vbus Impedance" },
        VdoField { index: 13, mask: 0x00002000, description: "Reserved" },
        VdoField { index: 14, mask: 0x00004000, description: "Charge Through Current Support" },
        VdoField { index: 15, mask: 0x00018000, description: "Maximum Vbus Voltage" },
        VdoField { index: 17, mask: 0x001e0000, description: "Reserved" },
    ];

    /// Contains information about a passive cable.
    ///
    /// ref: USB Power Delivery R3.0 V2.0, Section 6.4.4.3.1.6 Passive Cable VDO, Table 6-38 Passive Cable VDO
    pub const PASSIVE_VDO: &[VdoField] = &[
        VdoField { index: 0, mask: 0x00000007, description: "USB Speed" },
        VdoField { index: 3, mask: 0x00000018, description: "Reserved" },
        VdoField { index: 5, mask: 0x00000060, description: "Vbus Current Handling" },
        VdoField { index: 7, mask: 0x00000180, description: "Reserved" },
        VdoField { index: 9, mask: 0x00000600, description: "Maximum Vbus Voltage" },
        VdoField { index: 11, mask: 0x00001800, description: "Cable Termination Type" },
        VdoField { index: 13, mask: 0x0001e000, description: "Cable Latency" },
        VdoField { index: 17, mask: 0x00020000, description: "Reserved" },
        VdoField { index: 18, mask: 0x000c0000, description: "USB Type-C Plug to USB Type" },
        VdoField { index: 20, mask: 0x00100000, description: "Reserved" },
        VdoField { index: 21, mask: 0x00e00000, description: "VDO Version" },
        VdoField { index: 24, mask: 0x0f000000, description: "Firmware Version" },
        VdoField { index: 28, mask: 0xf0000000, description: "HW Version" },
    ];

    /// Contains information about a USB-PD Upstream Facing Port (UFP).
    ///
    /// ref: USB Power Delivery R3.0 V2.0, Section 6.4.4.3.1.4 UFP VDOs, Table 6-35 UFP VDO 1
    pub const UFP_VDO1: &[VdoField] = &[
        VdoField { index: 0, mask: 0x00000007, description: "USB Highest Speed" },
        VdoField { index: 3, mask: 0x00000038, description: "Alternate Modes" },
        VdoField { index: 6, mask: 0x00ffffc0, description: "Reserved" },
        VdoField { index: 24, mask: 0x0f000000, description: "Device Capability" },
        VdoField { index: 28, mask: 0x10000000, description: "Reserved" },
        VdoField { index: 29, mask: 0xe0000000, description: "UFP VDO Version" },
    ];

    /// Contains information about a USB-PD Upstream Facing Port (UFP).
    ///
    /// ref: USB Power Delivery R3.0 V2.0, Section 6.4.4.3.1.4 UFP VDOs, Table 6-36 UFP VDO 2
    pub const UFP_VDO2: &[VdoField] = &[
        VdoField { index: 0, mask: 0x0000007f, description: "USB3 Max Power" },
        VdoField { index: 7, mask: 0x00003f80, description: "USB3 Min Power" },
        VdoField { index: 14, mask: 0x0000c000, description: "Reserved" },
        VdoField { index: 16, mask: 0x007f0000, description: "USB4 Max Power" },
        VdoField { index: 23, mask: 0x3f800000, description: "USB4 Min Power" },
        VdoField { index: 30, mask: 0xc0000000, description: "Reserved" },
    ];

    /// Contains information about a USB-PD Downstream Facing Port (DFP).
    ///
    /// ref: USB Power Delivery R3.0 V2.0, Section 6.4.4.3.1.5 DFP VDO, Table 6-37 DFP VDO
    pub const DFP_VDO: &[VdoField] = &[
        VdoField { index: 0, mask: 0x0000001f, description: "Port Number" },
        VdoField { index: 5, mask: 0x00ffffe0, description: "Reserved" },
        VdoField { index: 24, mask: 0x07000000, description: "Host Capability" },
        VdoField { index: 27, mask: 0x18000000, description: "Reserved" },
        VdoField { index: 29, mask: 0xe0000000, description: "DFP VDO Version" },
        VdoField { index: 21, mask: 0x00e00000, description: "VDO Version" },
        VdoField { index: 24, mask: 0x0f000000, description: "Firmware Version" },
        VdoField { index: 28, mask: 0xf0000000, description: "HW Version" },
    ];

    /// Contains information about an active cable.
    ///
    /// ref: USB Power Delivery R3.0 V2.0, Section 6.4.4.3.1.7 Active Cable VDOs, Table 6-39 Active Cable VDO 1
    pub const ACTIVE_VDO1: &[VdoField] = &[
        VdoField { index: 0, mask: 0x00000007, description: "USB Speed" },
        VdoField { index: 3, mask: 0x00000008, description: "SOP'' Controller Present" },
        VdoField { index: 4, mask: 0x00000010, description: "Vbus Through Cable" },
        VdoField { index: 5, mask: 0x00000060, description: "Vbus Current Handling" },
        VdoField { index: 7, mask: 0x00000080, description: "SBU Type" },
        VdoField { index: 8, mask: 0x00000100, description: "SBU Supported" },
        VdoField { index: 9, mask: 0x00000600, description: "Maximum Vbus Voltage" },
        VdoField { index: 11, mask: 0x00001800, description: "Cable Termination Type" },
        VdoField { index: 13, mask: 0x0001e000, description: "Cable Latency" },
        VdoField { index: 17, mask: 0x00020000, description: "Reserved" },
        VdoField { index: 18, mask: 0x000c0000, description: "Connector Type" },
        VdoField { index: 20, mask: 0x00100000, description: "Reserved" },
    ];

    /// Contains information about an active cable.
    ///
    /// ref: USB Power Delivery R3.0 V2.0, Section 6.4.4.3.1.7 Active Cable VDOs , Table 6-40 Active Cable VDO 2
    pub const ACTIVE_VDO2: &[VdoField] = &[
        VdoField { index: 0, mask: 0x00000001, description: "USB Gen" },
        VdoField { index: 1, mask: 0x00000002, description: "Reserved" },
        VdoField { index: 2, mask: 0x00000004, description: "Optically Insulated Active Cable" },
        VdoField { index: 3, mask: 0x00000008, description: "USB Lanes Supported" },
        VdoField { index: 4, mask: 0x00000010, description: "USB 3.2 Supported" },
        VdoField { index: 5, mask: 0x00000020, description: "USB 2.0 Supported" },
        VdoField { index: 6, mask: 0x000000c0, description: "USB 2.0 Hub Hops Command" },
        VdoField { index: 8, mask: 0x00000100, description: "USB4 Supported" },
        VdoField { index: 9, mask: 0x00000200, description: "Active Element" },
        VdoField { index: 10, mask: 0x00000400, description: "Physical Connection" },
        VdoField { index: 11, mask: 0x00000800, description: "U3 to U0 Transition Mode" },
        VdoField { index: 12, mask: 0x00007000, description: "U3/CLd Power" },
        VdoField { index: 15, mask: 0x00008000, description: "Reserved" },
        VdoField { index: 16, mask: 0x00ff0000, description: "Shutdown Tempurature" },
        VdoField { index: 24, mask: 0xff000000, description: "Max Operating Tempurature" },
    ];
}

/// Constants specific to USB PD revision 3.1.
///
/// ref: USB Power Delivery R3.1 V1.8
pub mod pd31_data {
    use super::VdoField;

    /// Indicates a product is a PDUSB Hub's Upstream Facing Port
    ///
    /// ref: USB Power Delivery R3.1 V1.8, Section 6.4.4.3.1.1.3 Product Type (UFP), Table 6-34 Product Types (UFP), Row: PDUSB Hub
    pub const HUB_COMP: u32 = 0x08000000;
    /// Indicates a product is a PDUSB Peripheral
    ///
    /// ref: USB Power Delivery R3.1 V1.8, Section 6.4.4.3.1.1.3 Product Type (UFP), Table 6-34 Product Types (UFP), Row: PDUSB Peripheral
    pub const PERIPHERAL_COMP: u32 = 0x10000000;
    /// Indicates a product is a PDUSB Hub's Downstream Facing Port
    ///
    /// ref: USB Power Delivery R3.1 V1.8, Section 6.4.4.3.1.1.6 Product Type (DFP), Table 6-36 Product Types (DFP), Row: PDUSB Hub
    pub const DFP_HUB_COMP: u32 = 0x00800000;
    /// Indicates a product is a PDUSB Host's Downstream Facing Port
    ///
    /// ref: USB Power Delivery R3.1 V1.8, Section 6.4.4.3.1.1.6 Product Type (DFP), Table 6-36 Product Types (DFP), Row: PDUSB Host
    pub const DFP_HOST_COMP: u32 = 0x01000000;
    /// Indicates a product is a Power Brick
    ///
    /// ref: USB Power Delivery R3.1 V1.8, Section 6.4.4.3.1.1.6 Product Type (DFP), Table 6-36 Product Types (DFP), Row: Power Brick
    pub const POWER_BRICK_COMP: u32 = 0x01800000;

    // Expected id_header field results.
    /// Indicates a product is a passive cable
    ///
    /// ref: USB Power Delivery R3.1 V1.8, Section 6.4.4.3.1.1.4 Product Type (Cable Plug), Table 6-35 Product Types (Cable Plug/VPD), Row: Passive Cable
    pub const PASSIVE_CABLE_COMP: u32 = 0x18000000;
    /// Indicates a product is an active cable
    ///
    /// ref: USB Power Delivery R3.1 V1.8, Section 6.4.4.3.1.1.4 Product Type (Cable Plug), Table 6-35 Product Types (Cable Plug/VPD), Row: Active Cable
    pub const ACTIVE_CABLE_COMP: u32 = 0x20000000;
    /// Indicates a product is a VCONN Powered USB Device
    ///
    /// ref: USB Power Delivery R3.1 V1.8, Section 6.4.4.3.1.1.4 Product Type (Cable Plug), Table 6-35 Product Types (Cable Plug/VPD), Row: VCONN Powered USB Device
    pub const VPD_COMP: u32 = 0x30000000;

    /// Contains information about the product, including its vendor ID, product type, and USB capabilities.
    ///
    /// ref: USB Power Delivery R3.1 V1.8, Section 6.4.4.3.1.1 ID Header VDO, Table 6-33 ID Header VDO
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

    /// Contains information about the certification status of a product.
    ///
    /// ref: USB Power Delivery R3.1 V1.8, Section 6.4.4.3.1.2 Cert Stat VDO, Table 6-37 Cert Stat VDO
    pub const CERT_STAT_VDO: &[VdoField] =
        &[VdoField { index: 0, mask: 0xffffffff, description: "XID" }];

    /// Contains information about the product, including its bcdDevice and USB Product ID.
    ///
    /// ref: USB Power Delivery R3.1 V1.8, Section 6.4.4.3.1.3 Product VDO, Table 6-38 Product VDO
    pub const PRODUCT_VDO: &[VdoField] = &[
        VdoField { index: 0, mask: 0x0000ffff, description: "bcdDevice" },
        VdoField { index: 16, mask: 0xffff0000, description: "USB Product ID" },
    ];

    /// Contains information about a USB-PD Upstream Facing Port (UFP).
    ///
    /// ref: USB Power Delivery R3.1 V1.8, Section 6.4.4.3.1.4 UFP VDO, Table 6-39 UFP VDO
    pub const UFP_VDO: &[VdoField] = &[
        VdoField { index: 0, mask: 0x00000007, description: "USB Highest Speed" },
        VdoField { index: 3, mask: 0x00000038, description: "Alternate Modes" },
        VdoField { index: 6, mask: 0x00000040, description: "Vbus Required" },
        VdoField { index: 7, mask: 0x00000080, description: "Vconn Required" },
        VdoField { index: 8, mask: 0x00000700, description: "Vconn Power" },
        VdoField { index: 11, mask: 0x003ff800, description: "Reserved" },
        VdoField { index: 22, mask: 0x00c00000, description: "Connector Type (Legacy)" },
        VdoField { index: 24, mask: 0x0f000000, description: "Device Capability" },
        VdoField { index: 28, mask: 0x10000000, description: "Reserved" },
        VdoField { index: 29, mask: 0xe0000000, description: "UFP VDO Version" },
    ];

    /// Contains information about a USB-PD Downstream Facing Port (DFP).
    ///
    /// ref: USB Power Delivery R3.1 V1.8, Section 6.4.4.3.1.5 DFP VDO, Table 6-40 DFP VDO
    pub const DFP_VDO: &[VdoField] = &[
        VdoField { index: 0, mask: 0x0000001f, description: "Port Number" },
        VdoField { index: 5, mask: 0x003fffe0, description: "Reserved" },
        VdoField { index: 22, mask: 0x00c00000, description: "Connector Type (Legacy)" },
        VdoField { index: 24, mask: 0x07000000, description: "Host Capability" },
        VdoField { index: 27, mask: 0x18000000, description: "Reserved" },
        VdoField { index: 29, mask: 0xe0000000, description: "DFP VDO Version" },
    ];

    /// Contains information about a passive cable.
    ///
    /// ref: USB Power Delivery R3.1 V1.8, Section 6.4.4.3.1.6 Passive Cable VDO, Table 6-41 Passive Cable VDO
    pub const PASSIVE_VDO: &[VdoField] = &[
        VdoField { index: 0, mask: 0x00000007, description: "USB Speed" },
        VdoField { index: 3, mask: 0x00000018, description: "Reserved" },
        VdoField { index: 5, mask: 0x00000060, description: "Vbus Current Handling" },
        VdoField { index: 7, mask: 0x00000180, description: "Reserved" },
        VdoField { index: 9, mask: 0x00000600, description: "Maximum Vbus Voltage" },
        VdoField { index: 11, mask: 0x00001800, description: "Cable Termination Type" },
        VdoField { index: 13, mask: 0x0001e000, description: "Cable Latency" },
        VdoField { index: 17, mask: 0x00020000, description: "EPR Mode Cable" },
        VdoField { index: 18, mask: 0x000c0000, description: "USB Type-C Plug to USB Type" },
        VdoField { index: 20, mask: 0x00100000, description: "Reserved" },
        VdoField { index: 21, mask: 0x00e00000, description: "VDO Version" },
        VdoField { index: 24, mask: 0x0f000000, description: "Firmware Version" },
        VdoField { index: 28, mask: 0xf0000000, description: "HW Version" },
    ];

    /// Contains information about an active cable.
    ///
    /// ref: USB Power Delivery R3.1 V1.8, Section 6.4.4.3.1.7 Active Cable VDOs, Table 6-42 Active Cable VDO 1
    pub const ACTIVE_VDO1: &[VdoField] = &[
        VdoField { index: 0, mask: 0x00000007, description: "USB Speed" },
        VdoField { index: 3, mask: 0x00000008, description: "SOP'' Controller Present" },
        VdoField { index: 4, mask: 0x00000010, description: "Vbus Through Cable" },
        VdoField { index: 5, mask: 0x00000060, description: "Vbus Current Handling" },
        VdoField { index: 7, mask: 0x00000080, description: "SBU Type" },
        VdoField { index: 8, mask: 0x00000100, description: "SBU Supported" },
        VdoField { index: 9, mask: 0x00000600, description: "Maximum Vbus Voltage" },
        VdoField { index: 11, mask: 0x00001800, description: "Cable Termination Type" },
        VdoField { index: 13, mask: 0x0001e000, description: "Cable Latency" },
        VdoField { index: 17, mask: 0x00020000, description: "EPR Mode Cable" },
        VdoField { index: 18, mask: 0x000c0000, description: "USB Type-C Plug to USB Type" },
        VdoField { index: 20, mask: 0x00100000, description: "Reserved" },
        VdoField { index: 21, mask: 0x00e00000, description: "VDO Version" },
        VdoField { index: 24, mask: 0x0f000000, description: "Firmware Version" },
        VdoField { index: 28, mask: 0xf0000000, description: "HW Version" },
    ];

    /// Contains information about an active cable.
    ///
    /// ref: USB Power Delivery R3.1 V1.8, Section 6.4.4.3.1.7 Active Cable VDOs, Table 6-43 Active Cable VDO 2
    pub const ACTIVE_VDO2: &[VdoField] = &[
        VdoField { index: 0, mask: 0x00000001, description: "USB Gen" },
        VdoField { index: 1, mask: 0x00000002, description: "Reserved" },
        VdoField { index: 2, mask: 0x00000004, description: "Optically Insulated Active Cable" },
        VdoField { index: 3, mask: 0x00000008, description: "USB Lanes Supported" },
        VdoField { index: 4, mask: 0x00000010, description: "USB 3.2 Supported" },
        VdoField { index: 5, mask: 0x00000020, description: "USB 2.0 Supported" },
        VdoField { index: 6, mask: 0x000000c0, description: "USB 2.0 Hub Hops Command" }, // Corrected mask
        VdoField { index: 8, mask: 0x00000100, description: "USB4 Supported" },
        VdoField { index: 9, mask: 0x00000200, description: "Active Element" },
        VdoField { index: 10, mask: 0x00000400, description: "Physical Connection" },
        VdoField { index: 11, mask: 0x00000800, description: "U3 to U0 Transition Mode" },
        VdoField { index: 12, mask: 0x00007000, description: "U3/CLd Power" },
        VdoField { index: 15, mask: 0x00008000, description: "Reserved" },
        VdoField { index: 16, mask: 0x00ff0000, description: "Shutdown Tempurature" },
        VdoField { index: 24, mask: 0xff000000, description: "Max Operating Tempurature" },
    ];

    /// Contains information about a VCONN Powered USB Device (VPD).
    ///
    /// ref: USB Power Delivery R3.1 V1.8, Section 6.4.4.3.1.9 VCONN Powered USB Device VDO, Table 6-44 VPD VDO
    pub const VPD_VDO: &[VdoField] = &[
        VdoField { index: 0, mask: 0x00000001, description: "Charge Through Support" },
        VdoField { index: 1, mask: 0x0000007e, description: "Ground Impedance" },
        VdoField { index: 7, mask: 0x00001f80, description: "Vbus Impedance" },
        VdoField { index: 13, mask: 0x00002000, description: "Reserved" },
        VdoField { index: 14, mask: 0x00004000, description: "Charge Through Current Support" },
        VdoField { index: 15, mask: 0x00018000, description: "Maximum Vbus Voltage" },
        VdoField { index: 17, mask: 0x001e0000, description: "Reserved" },
        VdoField { index: 21, mask: 0x00e00000, description: "VDO Version" },
        VdoField { index: 24, mask: 0x0f000000, description: "Firmware Version" },
        VdoField { index: 28, mask: 0xf0000000, description: "HW Version" },
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
