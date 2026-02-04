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
    pub fn display(&self, vdo: u32) -> String {
        let field_val = self.decode_vdo(vdo);
        [self.description, &format!(": 0x{field_val:x}")].concat()
    }

    /// Decodes the field value from a VDO.
    pub fn decode_vdo(&self, vdo: u32) -> u32 {
        (vdo & self.mask) >> self.index
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
#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub enum PdRev {
    /// Represents an unknown or unspecified PD revision.
    #[default]
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

/// Represents the USB Speed
///
/// refs:
/// * USB Power Delivery R3.0 V2.0 - Table 6.35
/// * ref: USB Power Delivery R3.2 V1.1 - Table 6.41
/// * USB Power Delivery R2.0 V1.3 - Table 6.28
#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub enum UsbSpeed {
    /// USB 2.0 only speed - up to 480 Mbps.
    Usb2Only,
    /// USB 3.2 Gen 1 speed, up to 5 Gbps.
    ///
    /// Aliases (some outdated):
    /// * USB 3.0 - SuperSpeed
    /// * USB 3.1 Gen 1.
    /// * USB 3.2 Gen 1x1
    Speed5Gbps,
    /// USB 3.2 Gen 2 speed, up to 10 Gbps.
    ///
    /// Aliases (some outdated):
    /// * USB 3.1 Gen 2
    /// * USB 3.2 Gen 2x1 - SuperSpeedPlus
    Speed10Gbps,
    /// USB 3.2 Gen 2x2 speed or USB 4 Gen 2 speed, up to 20 Gbps.
    ///
    /// Aliases:
    /// * USB 4 Gen 2x2
    /// * USB 4 Gen 3x1
    Speed20Gbps,
    /// USB 4 Gen 3 speed, up to 40 Gbps.
    ///
    /// Aliases:
    /// * USB 4 Gen 3x2
    Speed40Gbps,
    /// USB 4 Gen 4 speed, up to 80 Gbps.
    ///
    /// Aliases:
    /// * USB 4 Gen 4x2
    Speed80Gbps,
    /// Represents an unknown or unspecified USB speed.
    #[default]
    Unknown,
}

/// Determines the cable's product type based on the id_header VDO value,
/// USB PD revision and product mask.
/// # Arguments
/// * `id_header` - The value of the "id_header" VDO.
/// * `pd` - The USB PD revision of the cable.
/// # Returns
/// * `ProductType` - The determined product type of the cable.
pub fn decode_cable_product_type(id_header: u32, pd: PdRev) -> ProductType {
    let prod_mask = id_header & UFP_PRODUCT_TYPE_MASK;

    // All USB PD 3.1/3.0/2.0 support passive cables.
    if pd == PdRev::Pd20 && prod_mask == pd20_data::PASSIVE_CABLE_COMP
        || pd == PdRev::Pd30 && prod_mask == pd30_data::PASSIVE_CABLE_COMP
        || pd == PdRev::Pd31 && prod_mask == pd31_data::PASSIVE_CABLE_COMP
    {
        return ProductType::Cable((CableType::Passive, pd));
    }

    // All USB PD 3.1/3.0/2.0 support active cables.
    if pd == PdRev::Pd20 && prod_mask == pd20_data::ACTIVE_CABLE_COMP
        || pd == PdRev::Pd30 && prod_mask == pd30_data::ACTIVE_CABLE_COMP
        || pd == PdRev::Pd31 && prod_mask == pd31_data::ACTIVE_CABLE_COMP
    {
        return ProductType::Cable((CableType::Active, pd));
    }

    // Since USB PD 3.1 Vconn Powered Devices (VPD) are also supported.
    if pd == PdRev::Pd31 && prod_mask == pd31_data::VPD_COMP {
        return ProductType::Cable((CableType::Vpd, pd));
    }

    ProductType::Other
}

/// Decodes the maximum theoretical speed supported by a cable from its VDOs.
#[allow(dead_code)]
pub fn decode_cable_highest_speed(
    pd_rev: PdRev,
    id_header_vdo: u32,
    product_type_vdo1: u32,
) -> UsbSpeed {
    let ProductType::Cable(cable_type) = decode_cable_product_type(id_header_vdo, pd_rev) else {
        return UsbSpeed::Unknown;
    };

    // The USB Speed field (bits 2:0) is in the same position for all relevant PD revisions
    // starting from PD 2.0 in the first Product Type VDO, so we can decode it once.
    let speed_vdo_field_decoded = pd20_data::PASSIVE_VDO[0].decode_vdo(product_type_vdo1);

    match cable_type {
        // No cable speed information in VPD_VDO
        (CableType::Vpd, _) => UsbSpeed::Unknown,
        (_, PdRev::Pd20) => match speed_vdo_field_decoded {
            0b000 => UsbSpeed::Usb2Only,
            0b001 => UsbSpeed::Speed5Gbps,
            0b010 => UsbSpeed::Speed10Gbps,
            _ => UsbSpeed::Unknown,
        },
        (_, PdRev::Pd30) => match speed_vdo_field_decoded {
            // Note: Assumes at least PD3.0 V2.0 minor version where the USB speed
            // is decoded the same way for passive and active cables.
            // In PD 3.0 V1.2 there we need to use ACTIVE_VDO2 for active cables.
            0b000 => UsbSpeed::Usb2Only,
            0b001 => UsbSpeed::Speed5Gbps,
            0b010 => UsbSpeed::Speed20Gbps,
            0b011 => UsbSpeed::Speed40Gbps,
            _ => UsbSpeed::Unknown,
        },
        (_, PdRev::Pd31) => match speed_vdo_field_decoded {
            0b000 => UsbSpeed::Usb2Only,
            0b001 => UsbSpeed::Speed5Gbps,
            0b010 => UsbSpeed::Speed20Gbps,
            0b011 => UsbSpeed::Speed40Gbps,
            0b100 => UsbSpeed::Speed80Gbps,
            _ => UsbSpeed::Unknown,
        },
        _ => UsbSpeed::Unknown,
    }
}

/// Decodes the Vendor ID and Product ID from the ID Header VDO and Product VDO.
///
/// # Returns
/// * `(u32, u32)` - The Vendor ID and Product ID.
#[allow(dead_code)]
pub fn get_vid_pid(id_header_vdo: u32, product_vdo: u32) -> (u32, u32) {
    (
        pd20_data::ID_HEADER_VDO[0].decode_vdo(id_header_vdo),
        pd20_data::PRODUCT_VDO[1].decode_vdo(product_vdo),
    )
}

/// Determines if the partner supports USB.
///
/// # Arguments
/// * `id_header_vdo` - The value of the "id_header" VDO from the partner.
/// # Returns
/// * `bool` - True if the partner supports USB, false otherwise.
#[allow(dead_code)]
pub fn get_partner_supports_usb(id_header_vdo: u32) -> bool {
    let supports_device = pd20_data::ID_HEADER_VDO[4].decode_vdo(id_header_vdo) != 0;
    let supports_host = pd20_data::ID_HEADER_VDO[5].decode_vdo(id_header_vdo) != 0;
    supports_device || supports_host
}

/// Determines if the partner supports USB4, based on the USB PD 3.1 specification.
///
/// # Arguments
/// * `id_header_vdo` - The value of the partner's "id_header" VDO.
/// * `product_type_vdo1` - The value of the first product type VDO from the partner.
/// # Returns
/// * `bool` - True if the partner supports USB4, false otherwise.
#[allow(dead_code)]
pub fn get_partner_supports_usb4(id_header_vdo: u32, product_type_vdo1: u32) -> bool {
    // A partner can be a Downstream Facing Port (DFP) or an Upstream Facing Port (UFP).
    // We check its product type from the ID Header VDO to determine which it is,
    // and then check for USB4 capability in its respective first Product Type VDO.
    let dfp_product_type = pd31_data::ID_HEADER_VDO[3].decode_vdo(id_header_vdo);
    let ufp_product_type = pd31_data::ID_HEADER_VDO[5].decode_vdo(id_header_vdo);

    // Check UFP first. If UFP type is set (even if DRD), VDO1 is the UFP VDO [5].
    // ref: USB Power Delivery R3.1 V2.0, section 6.4.4.3.1 Discover Identity, Paragraph 11 (Page 155).
    if ufp_product_type != 0 {
        // Partner is UFP.
        let device_capability_value = pd31_data::UFP_VDO[7].decode_vdo(product_type_vdo1);
        device_capability_value == pd31_data::UFP_USB4_CAPABILITY_VALUE
    } else if dfp_product_type != 0 {
        // Partner is DFP.
        let host_capability_value = pd31_data::DFP_VDO[3].decode_vdo(product_type_vdo1);
        host_capability_value == pd31_data::DFP_USB4_CAPABILITY_VALUE
    } else {
        // Not a recognized UFP or DFP product type.
        false
    }
}

/// Determines if the cable is an active cable.
///
/// # Arguments
/// * `id_header_vdo` - The value of the cable's "id_header" VDO.
/// # Returns
/// * `bool` - True if the cable is active, false otherwise.
#[allow(dead_code)]
pub fn is_active_cable(id_header_vdo: u32, pd_rev: PdRev) -> bool {
    matches!(
        decode_cable_product_type(id_header_vdo, pd_rev),
        ProductType::Cable((CableType::Active, _))
    )
}

/// Determines if the cable is captive based on the USB PD 3.1 specification.
///
/// # Arguments
/// * `product_type_vdo1` - The value of the cable's first product type VDO.
/// # Returns
/// * `bool` - True if the cable is captive, false otherwise.
#[allow(dead_code)]
pub fn is_captive_cable(product_type_vdo1: u32) -> bool {
    // According to the USB PD 3.1 spec, Table 6-41 (Passive Cable VDO) and Table 6-42 (Active Cable VDO 1),
    // bits 19..18 indicate the connector type. A value of 3 (0b11) signifies a captive cable.
    // This field is identical for both passive and active cables, so we can use either VDO definition.
    pd31_data::PASSIVE_VDO[8].decode_vdo(product_type_vdo1) == 3
}

/// Determines if the cable is a TBT3 only cable based on USB Type-C Spec revision 2.4
///
/// # Arguments
/// * `tbt_vdo` - The value of the TBT VDO.
/// # Returns
/// * `bool` - True if the cable is a TBT3 only cable, false otherwise.
#[allow(dead_code)]
pub fn is_tbt3_only_cable(cable_speed: UsbSpeed, tbt_vdo: u32) -> bool {
    // TBT only cables supporting 10Gbps or 20Gbps speeds are TBT3 Active Re-timer cables
    // that only support USB2.0 and TBT.
    //
    // We identify them by checking the TBT3 discovery VDO (from USB Type-C Spec R2.4, Table F-11).
    if cable_speed != UsbSpeed::Usb2Only {
        // TBT-only cables report a Cable highest speed of USB 2.0.
        return false;
    }
    let cable_speed = typec24_data::TBT3_SOP_PRIME_VDO[1].decode_vdo(tbt_vdo);
    cable_speed == typec24_data::TBT3_ONLY_10G_20G_CABLE_SPEED_VALUE
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

    /// Indicates a UFP partner is USB4 capable
    ///
    /// ref: USB Power Delivery R3.0 V2.0, Section 6.4.4.3.1.4 UFP VDOs, Table 6-35 UFP VDO 1, Row 27..24
    #[allow(dead_code)]
    pub const USB4_CAPABILITY_MASK: u32 = 0x8;

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

    /// Indicates a UFP partner is USB4 device capable.
    ///
    /// ref: USB Power Delivery R3.1 V1.8, Section 6.4.4.3.1.4 UFP VDO, Table 6-39 UFP VDO, Row: B27..24
    pub const UFP_USB4_CAPABILITY_VALUE: u32 = 3;

    /// Indicates a DFP partner is USB4 host capable.
    ///
    /// ref: USB Power Delivery R3.1 V1.8, Section 6.4.4.3.1.5 DFP VDO, Table 6-40 DFP VDO, Row: B26..24.
    pub const DFP_USB4_CAPABILITY_VALUE: u32 = 2;

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
        VdoField {
            index: 18,
            mask: 0x000c0000,
            description: "USB Type-C Plug to USB Type-C/Captive",
        },
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

/// Constants specific to USB Type-C specification revision 3.1.
///
/// ref: USB Type-C Specification Release 2.4
pub mod typec24_data {
    use super::VdoField;
    /// Indicates that the cable speed is 10 Gbps and 20 Gbps (TBT 3rd Gen active cables and 20 Gbps passive cables)
    ///
    /// ref: USB Type-C Specification Release 2.4, Section F.2.6 TBT3 Cable Discover Modes Responses, Table F-11 TBT3 Cable Discover Mode VDO Responses, Row B18..16
    pub const TBT3_ONLY_10G_20G_CABLE_SPEED_VALUE: u32 = 0b011;

    /// Contains information helping to decode a TBT3 SOP' VDO.
    ///
    /// ref: USB Type-C Specification Release 2.4, Section F.2.6 TBT3 Cable Discover Modes Responses, Table F-11 TBT3 Cable Discover Mode VDO Responses
    pub const TBT3_SOP_PRIME_VDO: &[VdoField] = &[
        VdoField { index: 0, mask: 0x0000ffff, description: "TBT Alternate Mode" },
        VdoField { index: 16, mask: 0x00070000, description: "Cable Speed" },
        VdoField { index: 19, mask: 0x00180000, description: "TBT_Rounded_Support" },
        VdoField { index: 21, mask: 0x00200000, description: "Cable Type" },
        VdoField { index: 22, mask: 0x00400000, description: "Re-timer" },
        VdoField { index: 23, mask: 0x00800000, description: "Active Cable Plug Link Training" },
        VdoField { index: 25, mask: 0x02000000, description: "Active_Passive" },
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_FIELD: VdoField = VdoField {
        index: 4,
        mask: 0x000000f0,
        description: "Value of the 2nd least significant byte",
    };

    #[test]
    fn test_decode_vdo() {
        let vdo: u32 = 0x12341234;
        assert_eq!(TEST_FIELD.decode_vdo(vdo), 0x3);
    }

    #[test]
    fn test_display_vdo() {
        let vdo: u32 = 0x12341234;
        assert_eq!(TEST_FIELD.display(vdo), "Value of the 2nd least significant byte: 0x3");
    }

    #[test]
    fn test_decode_cable_product_type() {
        let passive_id_header = 0x18001234;
        let active_id_header = 0x20005678;
        let vpd_id_header = 0x3000ABCD;
        let other_id_header = 0x00000000;

        // PD 2.0
        assert_eq!(
            decode_cable_product_type(passive_id_header, PdRev::Pd20),
            ProductType::Cable((CableType::Passive, PdRev::Pd20))
        );
        assert_eq!(
            decode_cable_product_type(active_id_header, PdRev::Pd20),
            ProductType::Cable((CableType::Active, PdRev::Pd20))
        );
        assert_eq!(decode_cable_product_type(other_id_header, PdRev::Pd20), ProductType::Other);

        // PD 3.0
        assert_eq!(
            decode_cable_product_type(passive_id_header, PdRev::Pd30),
            ProductType::Cable((CableType::Passive, PdRev::Pd30))
        );
        assert_eq!(
            decode_cable_product_type(active_id_header, PdRev::Pd30),
            ProductType::Cable((CableType::Active, PdRev::Pd30))
        );
        assert_eq!(decode_cable_product_type(other_id_header, PdRev::Pd30), ProductType::Other);

        // PD 3.1
        assert_eq!(
            decode_cable_product_type(passive_id_header, PdRev::Pd31),
            ProductType::Cable((CableType::Passive, PdRev::Pd31))
        );
        assert_eq!(
            decode_cable_product_type(active_id_header, PdRev::Pd31),
            ProductType::Cable((CableType::Active, PdRev::Pd31))
        );
        assert_eq!(
            decode_cable_product_type(vpd_id_header, PdRev::Pd31),
            ProductType::Cable((CableType::Vpd, PdRev::Pd31))
        );
        assert_eq!(decode_cable_product_type(other_id_header, PdRev::Pd31), ProductType::Other);

        // Unhandled PD revision
        assert_eq!(decode_cable_product_type(passive_id_header, PdRev::None), ProductType::Other);
    }

    #[test]
    fn test_decode_cable_highest_speed() {
        let passive_id_header = 0x18001234;
        let active_id_header = 0x20005678;
        let vpd_id_header = 0x3000ABCD;
        let other_id_header = 0x00000000;

        // Non-cable product type
        assert_eq!(
            decode_cable_highest_speed(PdRev::Pd30, other_id_header, 0b001),
            UsbSpeed::Unknown
        );

        // VPD cable type
        assert_eq!(
            decode_cable_highest_speed(PdRev::Pd31, vpd_id_header, 0b001),
            UsbSpeed::Unknown
        );

        // PD 2.0 passive cable
        assert_eq!(
            decode_cable_highest_speed(PdRev::Pd20, passive_id_header, 0b000),
            UsbSpeed::Usb2Only
        );
        assert_eq!(
            decode_cable_highest_speed(PdRev::Pd20, passive_id_header, 0b001),
            UsbSpeed::Speed5Gbps
        );

        // PD 3.0 passive cable
        assert_eq!(
            decode_cable_highest_speed(PdRev::Pd30, passive_id_header, 0b010),
            UsbSpeed::Speed20Gbps
        );

        // PD 3.1 passive cable
        assert_eq!(
            decode_cable_highest_speed(PdRev::Pd31, passive_id_header, 0b011),
            UsbSpeed::Speed40Gbps
        );

        // PD 3.1 active cable
        assert_eq!(
            decode_cable_highest_speed(PdRev::Pd31, active_id_header, 0b100),
            UsbSpeed::Speed80Gbps
        );
    }

    #[test]
    fn test_get_vid_pid() {
        let id_header_vdo = 0x12345678;
        let product_vdo = 0xABCDDCBA;
        assert_eq!(get_vid_pid(id_header_vdo, product_vdo), (0x5678, 0xABCD));
    }

    #[test]
    fn test_get_partner_supports_usb() {
        assert!(get_partner_supports_usb(0x40000000)); // Device only
        assert!(get_partner_supports_usb(0x80000000)); // Host only
        assert!(get_partner_supports_usb(0xC0000000)); // Both
        assert!(!get_partner_supports_usb(0x3FFFFFFF)); // Neither
    }

    #[test]
    fn test_get_partner_supports_usb4() {
        // Test values based on pd31_data definitions.
        const UFP_ID_HEADER: u32 = 1 << 27;
        const DFP_ID_HEADER: u32 = 1 << 23;
        const DRD_ID_HEADER: u32 = UFP_ID_HEADER | DFP_ID_HEADER;

        const UFP_VDO1_SUPPORTS_USB4: u32 = pd31_data::UFP_USB4_CAPABILITY_VALUE << 24;
        const UFP_VDO1_NO_USB4: u32 = (pd31_data::UFP_USB4_CAPABILITY_VALUE - 1) << 24;

        const DFP_VDO1_SUPPORTS_USB4: u32 = pd31_data::DFP_USB4_CAPABILITY_VALUE << 24;
        const DFP_VDO1_NO_USB4: u32 = (pd31_data::DFP_USB4_CAPABILITY_VALUE - 1) << 24;

        // UFP partner that supports USB4
        assert!(get_partner_supports_usb4(UFP_ID_HEADER, UFP_VDO1_SUPPORTS_USB4));

        // UFP partner that does not support USB4
        assert!(!get_partner_supports_usb4(UFP_ID_HEADER, UFP_VDO1_NO_USB4));

        // DFP partner that supports USB4
        assert!(get_partner_supports_usb4(DFP_ID_HEADER, DFP_VDO1_SUPPORTS_USB4));

        // DFP partner that does not support USB4
        assert!(!get_partner_supports_usb4(DFP_ID_HEADER, DFP_VDO1_NO_USB4));

        // DRD partner that supports USB4 (acts as UFP)
        assert!(get_partner_supports_usb4(DRD_ID_HEADER, UFP_VDO1_SUPPORTS_USB4));

        // DRD partner that does not support USB4
        assert!(!get_partner_supports_usb4(DRD_ID_HEADER, UFP_VDO1_NO_USB4));

        // Not a UFP or DFP partner
        assert!(!get_partner_supports_usb4(0, 0));
    }

    #[test]
    fn test_is_active_cable() {
        let active_id_header = 0x20005678;
        let passive_id_header = 0x18001234;
        let other_id_header = 0x00000000;

        assert!(is_active_cable(active_id_header, PdRev::Pd20));
        assert!(is_active_cable(active_id_header, PdRev::Pd31));
        assert!(!is_active_cable(passive_id_header, PdRev::Pd30));
        assert!(!is_active_cable(other_id_header, PdRev::Pd31));
    }

    #[test]
    fn test_is_captive_cable() {
        // Bits 19:18 are 3 (0b11) for captive
        let captive_vdo = 3 << 18;
        let not_captive_vdo = 2 << 18;

        assert!(is_captive_cable(captive_vdo));
        assert!(!is_captive_cable(not_captive_vdo));
        assert!(!is_captive_cable(0));
    }

    #[test]
    fn test_is_tbt3_only_cable() {
        // TBT3 only cable reports USB2.0 speed and has a specific TBT VDO value.
        let tbt3_vdo = typec24_data::TBT3_ONLY_10G_20G_CABLE_SPEED_VALUE << 16;
        let non_tbt3_vdo = 0;

        // Not a USB 2.0 cable, so can't be a TBT3 only cable.
        assert!(!is_tbt3_only_cable(UsbSpeed::Speed5Gbps, tbt3_vdo));

        // USB 2.0 cable with correct TBT VDO.
        assert!(is_tbt3_only_cable(UsbSpeed::Usb2Only, tbt3_vdo));

        // USB 2.0 cable with incorrect TBT VDO.
        assert!(!is_tbt3_only_cable(UsbSpeed::Usb2Only, non_tbt3_vdo));
    }
}
