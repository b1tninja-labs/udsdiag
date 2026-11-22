//! DiagnosticSessionControl (`0x10`): session types and request building (ISO 14229-1:2013, 9.2).
//!
//! DiagnosticSessionControl selects the server's active diagnostic session by a 1-byte
//! `diagnosticSessionType` sub-function. The request is just `0x10 <diagnosticSessionType>` — the
//! service has no request data parameters (9.2.2.3). [`build_request`] frames it and
//! [`session_types`] / [`describe`] name the values from Table 25.
//!
//! The sub-function byte can also carry the `suppressPosRspMsgIndicationBit` (bit 7); that general
//! mechanism is a separate obligation (see `ROADMAP.md`), so this builder frames the session type
//! in bits 6-0 only.

use crate::error::{Error, Result};

/// DiagnosticSessionControl service id.
pub const DIAGNOSTIC_SESSION_CONTROL: u8 = 0x10;

/// defaultSession (`0x01`).
pub const DEFAULT_SESSION: u8 = 0x01;
/// programmingSession (`0x02`).
pub const PROGRAMMING_SESSION: u8 = 0x02;
/// extendedDiagnosticSession (`0x03`).
pub const EXTENDED_DIAGNOSTIC_SESSION: u8 = 0x03;
/// safetySystemDiagnosticSession (`0x04`).
pub const SAFETY_SYSTEM_DIAGNOSTIC_SESSION: u8 = 0x04;

/// One defined diagnosticSessionType (ISO 14229-1:2013, Table 25).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionType {
    /// The sub-function value.
    pub value: u8,
    /// The session name.
    pub name: &'static str,
    /// The session mnemonic.
    pub mnemonic: &'static str,
}

const fn session(value: u8, name: &'static str, mnemonic: &'static str) -> SessionType {
    SessionType {
        value,
        name,
        mnemonic,
    }
}

/// The diagnosticSessionType values defined by ISO 14229-1:2013 (Table 25).
pub const SESSION_TYPES: &[SessionType] = &[
    session(DEFAULT_SESSION, "defaultSession", "DS"),
    session(PROGRAMMING_SESSION, "programmingSession", "PRGS"),
    session(
        EXTENDED_DIAGNOSTIC_SESSION,
        "extendedDiagnosticSession",
        "EXTDS",
    ),
    session(
        SAFETY_SYSTEM_DIAGNOSTIC_SESSION,
        "safetySystemDiagnosticSession",
        "SSDS",
    ),
];

/// Every defined diagnosticSessionType.
pub const fn session_types() -> &'static [SessionType] {
    SESSION_TYPES
}

/// The defined session for a value, or `None` for a reserved/range value.
pub fn get(value: u8) -> Option<SessionType> {
    SESSION_TYPES.iter().copied().find(|s| s.value == value)
}

/// A label for any diagnosticSessionType value, naming the range for an undefined one.
pub fn describe(value: u8) -> &'static str {
    if let Some(session) = get(value) {
        return session.name;
    }
    match value {
        0x40..=0x5F => "vehicleManufacturerSpecific",
        0x60..=0x7E => "systemSupplierSpecific",
        _ => "ISOSAEReserved",
    }
}

/// Build a DiagnosticSessionControl request PDU (`0x10 <diagnosticSessionType>`).
///
/// # Errors
/// Returns an error when `session_type` is not a usable sub-function value `0x01`-`0x7E` (`0x00`
/// and `0x7F` are reserved, and bit 7 is the separate suppressPosRspMsgIndicationBit).
pub fn build_request(session_type: u8) -> Result<Vec<u8>> {
    if !(0x01..=0x7E).contains(&session_type) {
        return Err(Error::new(format!(
            "not a diagnosticSessionType (0x01-0x7E): {session_type:#04x}"
        )));
    }
    Ok(vec![DIAGNOSTIC_SESSION_CONTROL, session_type])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_request_frames_the_session_type() {
        assert_eq!(build_request(DEFAULT_SESSION).unwrap(), vec![0x10, 0x01]);
        assert_eq!(
            build_request(EXTENDED_DIAGNOSTIC_SESSION).unwrap(),
            vec![0x10, 0x03]
        );
        assert!(build_request(0x00).is_err()); // reserved
        assert!(build_request(0x7F).is_err()); // reserved
        assert!(build_request(0x80).is_err()); // bit 7 is suppressPosRspMsgIndicationBit
    }

    #[test]
    fn session_types_match_table_25() {
        assert_eq!(session_types().len(), 4);
        assert_eq!(get(PROGRAMMING_SESSION).unwrap().mnemonic, "PRGS");
        assert_eq!(get(0x03).unwrap().name, "extendedDiagnosticSession");
        assert!(get(0x05).is_none());
        assert_eq!(describe(0x01), "defaultSession");
        assert_eq!(describe(0x50), "vehicleManufacturerSpecific");
        assert_eq!(describe(0x70), "systemSupplierSpecific");
        assert_eq!(describe(0x05), "ISOSAEReserved");
    }
}
