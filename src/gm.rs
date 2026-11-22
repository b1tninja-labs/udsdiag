//! The GM / GMLAN vendor layer on top of the ISO 14229-1 core.
//!
//! GMLAN (GM single-wire / low-speed CAN) predates and parallels UDS, so its service identifiers
//! overlap the UDS range but carry GM semantics. These services and the `+0x400` / `+0x300`
//! addressing are GM-specific (recovered in the GM GMG3 head-unit assessment, `docs/sources.md`),
//! so they are modelled as a [`VendorProfile`] rather than mixed into the ISO catalog.
//!
//! The richer GM reference data the assessment recovered — the RoutineControl and IOControl
//! catalogs, the SecurityAccess mechanism descriptors, the Volt DID set and the Mode $06 document
//! list — is GM-sourced too and currently lives in its own modules (`routine`, `iocontrol`,
//! `security`, `did`, `mode06`). Routing it through this profile is tracked in `ROADMAP.md`; the
//! whole GM layer can then be lifted into its own crate behind this boundary without touching the
//! ISO core.

use crate::gmlan;
use crate::services::{Service, Standard};
use crate::vendor::VendorProfile;

const fn gm(sid: u8, name: &'static str, mnemonic: &'static str) -> Service {
    Service {
        sid,
        name,
        mnemonic,
        standard: Standard::Gmlan,
    }
}

/// The GMLAN services recovered in the GM GMG3 head-unit assessment (`docs/sources.md`).
pub const GM_SERVICES: &[Service] = &[
    gm(0x1A, "ReadDataByIdentifier (GMLAN)", "GM-RDBI"),
    gm(0x3B, "WriteDataByIdentifier (GMLAN)", "GM-WDBI"),
    gm(0xA2, "ReportProgrammedState (GMLAN)", "GM-RPS"),
    gm(0xA5, "ProgrammingMode (GMLAN)", "GM-PM"),
    gm(0xA9, "ReadDiagnosticInformation (GMLAN)", "GM-RDI"),
    gm(0xAA, "ReadDataByPacketIdentifier (GMLAN)", "GM-RDBPI"),
    gm(0xAE, "DeviceControl (GMLAN)", "GM-DC"),
];

/// The GM / GMLAN vendor layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gm;

/// The GM vendor profile.
pub const GM: Gm = Gm;

impl VendorProfile for Gm {
    fn name(&self) -> &'static str {
        "GM GMLAN"
    }

    fn services(&self) -> &'static [Service] {
        GM_SERVICES
    }

    fn response_id(&self, request_id: u16) -> Option<u16> {
        gmlan::response_id(request_id).ok()
    }

    fn filter_id(&self, request_id: u16) -> Option<u16> {
        gmlan::filter_id(request_id).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gm_services_are_flagged_and_addressed() {
        assert_eq!(GM.name(), "GM GMLAN");
        let device_control = GM.service(0xAE).expect("0xAE catalogued");
        assert_eq!(device_control.standard, Standard::Gmlan);
        assert_eq!(device_control.name, "DeviceControl (GMLAN)");
        assert_eq!(GM.services().len(), 7);
        assert_eq!(GM.response_id(0x241), Some(0x641));
        assert_eq!(GM.filter_id(0x241), Some(0x541));
        assert_eq!(GM.response_id(0x800), None); // not 11-bit
    }
}
