//! The diagnostic service-identifier (SID) catalog.
//!
//! Each [`Service`] names a request SID, its mnemonic and whether it is an ISO 14229-1 service
//! or a GM/GMLAN-specific one. A positive response echoes the request SID raised by `0x40`
//! (ISO 14229-1:2013, 7.2); [`positive_response_sid`] applies that offset and [`is_response_sid`]
//! recognises it.
//!
//! The ISO rows are the services defined in ISO 14229-1:2013. The GM rows (`0x1A`, `0x3B`,
//! `0xA2`, `0xA5`, `0xA9`, `0xAA`, `0xAE`) are the GMLAN services named in the GM GMG3 head-unit
//! assessment (`docs/sources.md`); GMLAN predates and parallels UDS, so its SIDs overlap the UDS
//! range but carry GM semantics. A SID outside the catalog is not an error here, only unknown.

use crate::error::{Error, Result};

/// A positive-response SID is the request SID raised by this offset (ISO 14229-1:2013, 7.2).
pub const POSITIVE_RESPONSE_OFFSET: u8 = 0x40;
/// The leading byte of a negative-response PDU (ISO 14229-1:2013, 7.5.6).
pub const NEGATIVE_RESPONSE_SID: u8 = 0x7F;

/// Which standard defines a service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standard {
    /// Defined in ISO 14229-1:2013 (the edition this crate models and `primarysource` carries).
    Iso14229,
    /// Defined in a later ISO 14229-1 edition (2020), not the 2013 one: a standard UDS service,
    /// not a vendor extension. The 2020 edition is not loaded in `primarysource`, so the edition
    /// attribution is unverified against a primary source (see `docs/iso-map.md`).
    Iso14229Later,
    /// A GM GMLAN-specific service (see `docs/sources.md`).
    Gmlan,
}

impl Standard {
    /// The label shown in catalog output.
    pub const fn label(self) -> &'static str {
        match self {
            Standard::Iso14229 => "ISO 14229-1:2013",
            Standard::Iso14229Later => "ISO 14229-1:2020",
            Standard::Gmlan => "GMLAN",
        }
    }
}

/// One diagnostic service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Service {
    /// The request service identifier.
    pub sid: u8,
    /// The service name.
    pub name: &'static str,
    /// The service mnemonic.
    pub mnemonic: &'static str,
    /// Which standard defines the service.
    pub standard: Standard,
}

impl Service {
    /// The SID of this service's positive response message (request SID + `0x40`).
    pub const fn response_sid(&self) -> u8 {
        self.sid + POSITIVE_RESPONSE_OFFSET
    }
}

const fn iso(sid: u8, name: &'static str, mnemonic: &'static str) -> Service {
    Service {
        sid,
        name,
        mnemonic,
        standard: Standard::Iso14229,
    }
}

const fn gm(sid: u8, name: &'static str, mnemonic: &'static str) -> Service {
    Service {
        sid,
        name,
        mnemonic,
        standard: Standard::Gmlan,
    }
}

const SERVICES: &[Service] = &[
    // ISO 14229-1:2013 — diagnostic and communication management
    iso(0x10, "DiagnosticSessionControl", "DSC"),
    iso(0x11, "ECUReset", "ER"),
    iso(0x27, "SecurityAccess", "SA"),
    iso(0x28, "CommunicationControl", "CC"),
    // Authentication (0x29) is a standard UDS service, but from ISO 14229-1:2020, NOT the 2013
    // edition (absent from the 2013 Table 22 service list, confirmed via primarysource). It is not
    // a GM/GMLAN vendor extension — GM's own authentication is the ADIT ALD over SecurityAccess
    // (0x27), modelled in `security.rs`. The 2020 edition is not loaded in primarysource, so the
    // 2020 attribution is unverified; see docs/iso-map.md.
    Service {
        sid: 0x29,
        name: "Authentication",
        mnemonic: "AUTH",
        standard: Standard::Iso14229Later,
    },
    iso(0x3E, "TesterPresent", "TP"),
    iso(0x83, "AccessTimingParameter", "ATP"),
    iso(0x84, "SecuredDataTransmission", "SDT"),
    iso(0x85, "ControlDTCSetting", "CDTCS"),
    iso(0x86, "ResponseOnEvent", "ROE"),
    iso(0x87, "LinkControl", "LC"),
    // ISO 14229-1:2013 — data transmission
    iso(0x22, "ReadDataByIdentifier", "RDBI"),
    iso(0x23, "ReadMemoryByAddress", "RMBA"),
    iso(0x24, "ReadScalingDataByIdentifier", "RSDBI"),
    iso(0x2A, "ReadDataByPeriodicIdentifier", "RDBPI"),
    iso(0x2C, "DynamicallyDefineDataIdentifier", "DDDI"),
    iso(0x2E, "WriteDataByIdentifier", "WDBI"),
    iso(0x3D, "WriteMemoryByAddress", "WMBA"),
    // ISO 14229-1:2013 — stored data transmission (DTCs)
    iso(0x14, "ClearDiagnosticInformation", "CDTCI"),
    iso(0x19, "ReadDTCInformation", "RDTCI"),
    // ISO 14229-1:2013 — input/output control and routine
    iso(0x2F, "InputOutputControlByIdentifier", "IOCBI"),
    iso(0x31, "RoutineControl", "RC"),
    // ISO 14229-1:2013 — upload/download
    iso(0x34, "RequestDownload", "RD"),
    iso(0x35, "RequestUpload", "RU"),
    iso(0x36, "TransferData", "TD"),
    iso(0x37, "RequestTransferExit", "RTE"),
    iso(0x38, "RequestFileTransfer", "RFT"),
    // GMLAN (GM GMG3 head-unit assessment; see docs/sources.md)
    gm(0x1A, "ReadDataByIdentifier (GMLAN)", "GM-RDBI"),
    gm(0x3B, "WriteDataByIdentifier (GMLAN)", "GM-WDBI"),
    gm(0xA2, "ReportProgrammedState (GMLAN)", "GM-RPS"),
    gm(0xA5, "ProgrammingMode (GMLAN)", "GM-PM"),
    gm(0xA9, "ReadDiagnosticInformation (GMLAN)", "GM-RDI"),
    gm(0xAA, "ReadDataByPacketIdentifier (GMLAN)", "GM-RDBPI"),
    gm(0xAE, "DeviceControl (GMLAN)", "GM-DC"),
];

/// Every service in the catalog, in declaration order.
pub const fn all_services() -> &'static [Service] {
    SERVICES
}

/// The service for a request SID, or `None` when the SID is not catalogued.
pub fn get(sid: u8) -> Option<Service> {
    SERVICES.iter().copied().find(|service| service.sid == sid)
}

/// A human label for a request SID, falling back to the hex value.
pub fn name_of(sid: u8) -> String {
    match get(sid) {
        Some(service) => service.name.to_owned(),
        None => format!("UnknownService({sid:#04x})"),
    }
}

/// The positive-response SID for a request SID (request SID + `0x40`).
///
/// # Errors
/// Returns an error when `sid` is above `0xBF`, which has no positive-response SID in range.
pub fn positive_response_sid(sid: u8) -> Result<u8> {
    if sid > 0xBF {
        return Err(Error::new(format!(
            "request sid has no positive response sid: {sid:#x}"
        )));
    }
    Ok(sid + POSITIVE_RESPONSE_OFFSET)
}

/// Whether a leading byte is a positive-response SID of some catalogued service.
pub fn is_response_sid(byte: u8) -> bool {
    SERVICES
        .iter()
        .any(|service| service.response_sid() == byte)
}

/// The request SID a positive-response SID answers (response SID - `0x40`).
pub fn request_sid_for_response(response_sid: u8) -> u8 {
    response_sid.wrapping_sub(POSITIVE_RESPONSE_OFFSET)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_the_core_services() {
        for sid in [0x10, 0x11, 0x14, 0x19, 0x22, 0x27, 0x2E, 0x2F, 0x31, 0x3E] {
            assert!(get(sid).is_some(), "missing {sid:#x}");
        }
        assert_eq!(get(0x31).unwrap().name, "RoutineControl");
        // Authentication (0x29) is carried, but tagged as the 2020 edition, not 2013 (Table 22).
        let auth = get(0x29).expect("0x29 catalogued");
        assert_eq!(auth.name, "Authentication");
        assert_eq!(auth.standard, Standard::Iso14229Later);
        assert_ne!(auth.standard, Standard::Iso14229);
        // RequestFileTransfer (0x38) is a 2013 upload/download service (contents 14.6).
        assert_eq!(get(0x38).unwrap().name, "RequestFileTransfer");
    }

    #[test]
    fn positive_response_sid_adds_0x40() {
        assert_eq!(positive_response_sid(0x22).unwrap(), 0x62);
        assert_eq!(positive_response_sid(0x31).unwrap(), 0x71);
        assert!(is_response_sid(0x62));
        assert_eq!(request_sid_for_response(0x62), 0x22);
        assert!(positive_response_sid(0xFF).is_err());
    }

    #[test]
    fn get_returns_none_for_unknown_sid() {
        assert!(get(0x00).is_none());
        assert!(name_of(0x00).starts_with("UnknownService"));
    }

    #[test]
    fn gmlan_services_are_flagged() {
        let device_control = get(0xAE).expect("0xAE catalogued");
        assert_eq!(device_control.standard, Standard::Gmlan);
        assert_eq!(get(0x22).unwrap().standard, Standard::Iso14229);
    }
}
