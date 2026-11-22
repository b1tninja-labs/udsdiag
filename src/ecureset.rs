//! ECUReset (`0x11`): reset types and request building (ISO 14229-1:2013, 9.3).
//!
//! ECUReset asks the server to perform a reset selected by a 1-byte `resetType` sub-function. The
//! request is just `0x11 <resetType>` — the service has no request data parameters (9.3.2.3).
//! [`build_request`] frames it and [`reset_types`] / [`describe`] name the values from Table 34.
//!
//! As with the other sub-function services, the byte can carry the `suppressPosRspMsgIndicationBit`
//! (bit 7); that general mechanism is a separate obligation, so this builder frames the reset type
//! in bits 6-0 only.

use crate::error::{Error, Result};

/// ECUReset service id.
pub const ECU_RESET: u8 = 0x11;

/// hardReset (`0x01`).
pub const HARD_RESET: u8 = 0x01;
/// keyOffOnReset (`0x02`).
pub const KEY_OFF_ON_RESET: u8 = 0x02;
/// softReset (`0x03`).
pub const SOFT_RESET: u8 = 0x03;
/// enableRapidPowerShutDown (`0x04`).
pub const ENABLE_RAPID_POWER_SHUT_DOWN: u8 = 0x04;
/// disableRapidPowerShutDown (`0x05`).
pub const DISABLE_RAPID_POWER_SHUT_DOWN: u8 = 0x05;

/// One defined resetType (ISO 14229-1:2013, Table 34).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResetType {
    /// The sub-function value.
    pub value: u8,
    /// The reset name.
    pub name: &'static str,
    /// The reset mnemonic.
    pub mnemonic: &'static str,
}

const fn reset(value: u8, name: &'static str, mnemonic: &'static str) -> ResetType {
    ResetType {
        value,
        name,
        mnemonic,
    }
}

/// The resetType values defined by ISO 14229-1:2013 (Table 34).
pub const RESET_TYPES: &[ResetType] = &[
    reset(HARD_RESET, "hardReset", "HR"),
    reset(KEY_OFF_ON_RESET, "keyOffOnReset", "KOFFONR"),
    reset(SOFT_RESET, "softReset", "SR"),
    reset(
        ENABLE_RAPID_POWER_SHUT_DOWN,
        "enableRapidPowerShutDown",
        "ERPSD",
    ),
    reset(
        DISABLE_RAPID_POWER_SHUT_DOWN,
        "disableRapidPowerShutDown",
        "DRPSD",
    ),
];

/// Every defined resetType.
pub const fn reset_types() -> &'static [ResetType] {
    RESET_TYPES
}

/// The defined reset for a value, or `None` for a reserved/range value.
pub fn get(value: u8) -> Option<ResetType> {
    RESET_TYPES.iter().copied().find(|r| r.value == value)
}

/// A label for any resetType value, naming the range for an undefined one.
pub fn describe(value: u8) -> &'static str {
    if let Some(reset) = get(value) {
        return reset.name;
    }
    match value {
        0x40..=0x5F => "vehicleManufacturerSpecific",
        0x60..=0x7E => "systemSupplierSpecific",
        _ => "ISOSAEReserved",
    }
}

/// Build an ECUReset request PDU (`0x11 <resetType>`).
///
/// # Errors
/// Returns an error when `reset_type` is not a usable sub-function value `0x01`-`0x7E` (`0x00` and
/// `0x7F` are reserved, and bit 7 is the separate suppressPosRspMsgIndicationBit).
pub fn build_request(reset_type: u8) -> Result<Vec<u8>> {
    if !(0x01..=0x7E).contains(&reset_type) {
        return Err(Error::new(format!(
            "not a resetType (0x01-0x7E): {reset_type:#04x}"
        )));
    }
    Ok(vec![ECU_RESET, reset_type])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_request_frames_the_reset_type() {
        assert_eq!(build_request(HARD_RESET).unwrap(), vec![0x11, 0x01]);
        assert_eq!(build_request(SOFT_RESET).unwrap(), vec![0x11, 0x03]);
        assert!(build_request(0x00).is_err()); // reserved
        assert!(build_request(0x7F).is_err()); // reserved
        assert!(build_request(0x80).is_err()); // bit 7 is suppressPosRspMsgIndicationBit
    }

    #[test]
    fn reset_types_match_table_34() {
        assert_eq!(reset_types().len(), 5);
        assert_eq!(get(HARD_RESET).unwrap().mnemonic, "HR");
        assert_eq!(get(0x04).unwrap().name, "enableRapidPowerShutDown");
        assert_eq!(get(0x05).unwrap().mnemonic, "DRPSD");
        assert!(get(0x06).is_none());
        assert_eq!(describe(0x02), "keyOffOnReset");
        assert_eq!(describe(0x50), "vehicleManufacturerSpecific");
        assert_eq!(describe(0x70), "systemSupplierSpecific");
        assert_eq!(describe(0x06), "ISOSAEReserved");
    }
}
