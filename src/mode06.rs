//! Mode $06 / Mode $09 service identifiers and the OBD reference-document type.
//!
//! Mode $06 (On-Board Monitoring Test Results) and Mode $09 (Vehicle Information) are the generic
//! OBD-II services of SAE J1979 / ISO 15031-5. This module holds the service ids and the
//! [`Document`] type; the catalog of GM's own published Mode $06 reference PDFs is GM-specific and
//! lives in the `gm` vendor layer (`gm::mode06_documents`).

/// Mode $06 On-Board Monitoring Test Results service id.
pub const MODE_06_ON_BOARD_MONITORING: u8 = 0x06;
/// Mode $09 Vehicle Information service id.
pub const MODE_09_VEHICLE_INFORMATION: u8 = 0x09;

/// One GM OBD reference PDF.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Document {
    /// The PDF file name.
    pub filename: &'static str,
    /// The OBD groups the file applies to; empty marks a generic (all-vehicle) file.
    pub obd_groups: &'static [&'static str],
    /// What the file contains.
    pub what: &'static str,
}
