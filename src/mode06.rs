//! The GM Mode $06 document catalog.
//!
//! Mode $06 (On-Board Monitoring Test Results) and Mode $09 (Vehicle Information) are the generic
//! OBD-II services of SAE J1979 / ISO 15031-5. Manufacturers publish the per-engine-family
//! diagnostic-parameter files and the Mode $06 monitor/test-ID definitions for the independent
//! aftermarket, as the CARB/EPA OBD-II service-information rules require. This module catalogs
//! GM's own published 2017 OBD reference PDFs (`docs/sources.md`): these are GM documents, not
//! reverse-engineered material.
//!
//! They are the *standardized* OBD-II counterpart to the *enhanced* GMLAN DIDs/routines the rest
//! of this crate models — complementary, not overlapping. For the Chevrolet Volt, which is OBD
//! Group 1A, the applicable files are `17OBDG01 Hybrid Diagnostics` and the `GM CAN mode $06
//! data` reference; [`files_for_group`] and [`VOLT_OBD_GROUP`] encode that.

/// The base URL GM publishes the Mode $06 PDFs under.
pub const GM_MODE6_BASE: &str = "https://gsitlc.ext.gm.com/gmspo/mode6/pdf/";
/// The Chevrolet Volt's OBD group.
pub const VOLT_OBD_GROUP: &str = "1A";

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

const fn doc(
    filename: &'static str,
    obd_groups: &'static [&'static str],
    what: &'static str,
) -> Document {
    Document {
        filename,
        obd_groups,
        what,
    }
}

// GM 2017 OBD reference PDFs (docs/sources.md). An empty obd_groups marks a generic file.
const DOCS: &[Document] = &[
    doc(
        "2017_OBD_GROUP_SUMMARY.pdf",
        &[],
        "index: Engine/Transmission diagnostic parameter file matrix",
    ),
    doc(
        "17OBDG01 Hybrid Diagnostics.pdf",
        &["1A", "1B"],
        "Hybrid powertrain diagnostic parameters (Volt, ELR, CT6)",
    ),
    doc(
        "17OBDG02 Hybrid Diagnostics.pdf",
        &["2"],
        "Hybrid powertrain diagnostic parameters (Malibu 1.8L hybrid)",
    ),
    doc(
        "17OBDG03 Engine Diagnostics.pdf",
        &["3"],
        "Engine diagnostic parameters (conventional gas 1.4L-6.2L)",
    ),
    doc(
        "17OBDG03 Transmisson Diagnostics.pdf",
        &["3"],
        "Transmission diagnostic parameters (GM's spelling)",
    ),
    doc(
        "17OBDG04 Engine Diagnostics.pdf",
        &["4"],
        "Engine diagnostic parameters (diesels: LH7, LWN, L5P Duramax)",
    ),
    doc(
        "17OBDG04 Transmisson Diagnostics.pdf",
        &["4"],
        "Transmission diagnostic parameters (GM's spelling)",
    ),
    doc(
        "17OBDG05 Engine Diagnostics.pdf",
        &["5"],
        "Engine diagnostic parameters (6.0L L96 HD)",
    ),
    doc(
        "17OBDG05 Transmission Diagnostics.pdf",
        &["5"],
        "Transmission diagnostic parameters",
    ),
    doc(
        "17OBDG06 Engine Diagnostics.pdf",
        &["6"],
        "Engine diagnostic parameters (6.0L L96)",
    ),
    doc(
        "17OBDG06 Transmission Diagnostics.pdf",
        &["6"],
        "Transmission diagnostic parameters",
    ),
    doc(
        "GM CAN mode $06 data final_dm.pdf",
        &[],
        "Generic Mode $06 monitor/test-ID definitions (CAN/GMLAN, ISO 15031-5)",
    ),
    doc(
        "GM Class2 mode $06 data final_dm.pdf",
        &[],
        "Generic Mode $06 monitor/test-ID definitions (legacy Class 2)",
    ),
];

/// Every catalogued GM OBD reference PDF.
pub const fn documents() -> &'static [Document] {
    DOCS
}

/// The documents applicable to an OBD group: its group-specific files plus the generic ones.
pub fn files_for_group(group: &str) -> Vec<Document> {
    DOCS.iter()
        .copied()
        .filter(|d| d.obd_groups.is_empty() || d.obd_groups.contains(&group))
        .collect()
}

/// The documents applicable to the Chevrolet Volt (OBD Group 1A).
pub fn volt_files() -> Vec<Document> {
    files_for_group(VOLT_OBD_GROUP)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volt_is_group_1a() {
        assert_eq!(VOLT_OBD_GROUP, "1A");
        let names: std::collections::BTreeSet<&str> =
            volt_files().iter().map(|d| d.filename).collect();
        assert!(names.contains("17OBDG01 Hybrid Diagnostics.pdf"));
        assert!(names.contains("GM CAN mode $06 data final_dm.pdf"));
        assert!(!names.contains("17OBDG02 Hybrid Diagnostics.pdf"));
    }

    #[test]
    fn files_for_group_include_generic() {
        let names: std::collections::BTreeSet<&str> =
            files_for_group("3").iter().map(|d| d.filename).collect();
        assert!(names.contains("17OBDG03 Engine Diagnostics.pdf"));
        assert!(names.contains("GM CAN mode $06 data final_dm.pdf"));
        assert!(names.contains("2017_OBD_GROUP_SUMMARY.pdf"));
        assert!(documents().len() >= 10);
    }
}
