//! ControlDTCSetting (`0x85`): the DTCSettingType sub-functions and request building
//! (ISO 14229-1:2013, 9.9).
//!
//! ControlDTCSetting tells the server to stop or resume updating DTC status bits. The request is
//! `0x85 <DTCSettingType> [DTCSettingControlOptionRecord...]`; the option record is optional and
//! vehicle-manufacturer-specific. [`setting_types`] / [`describe`] name the DTCSettingType values
//! (Table 87) — `0x01` on, `0x02` off — and [`build_request`] frames the PDU.
//!
//! The sub-function byte can carry the `suppressPosRspMsgIndicationBit` (bit 7); that general
//! mechanism is a separate obligation, so this builder frames the setting type in bits 6-0 only.

use crate::error::{Error, Result};

/// ControlDTCSetting service id.
pub const CONTROL_DTC_SETTING: u8 = 0x85;

/// on (`0x01`): resume updating DTC status bits.
pub const DTC_SETTING_ON: u8 = 0x01;
/// off (`0x02`): stop updating DTC status bits.
pub const DTC_SETTING_OFF: u8 = 0x02;

/// One defined DTCSettingType (ISO 14229-1:2013, Table 87).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingType {
    /// The sub-function value.
    pub value: u8,
    /// The setting name.
    pub name: &'static str,
    /// The setting mnemonic.
    pub mnemonic: &'static str,
}

const fn setting(value: u8, name: &'static str, mnemonic: &'static str) -> SettingType {
    SettingType {
        value,
        name,
        mnemonic,
    }
}

/// The DTCSettingType values defined by ISO 14229-1:2013 (Table 87).
pub const SETTING_TYPES: &[SettingType] = &[
    setting(DTC_SETTING_ON, "on", "ON"),
    setting(DTC_SETTING_OFF, "off", "OFF"),
];

/// Every defined DTCSettingType.
pub const fn setting_types() -> &'static [SettingType] {
    SETTING_TYPES
}

/// The defined setting for a value, or `None` for a reserved/range value.
pub fn get(value: u8) -> Option<SettingType> {
    SETTING_TYPES.iter().copied().find(|s| s.value == value)
}

/// A label for any DTCSettingType value, naming the range for an undefined one.
pub fn describe(value: u8) -> &'static str {
    if let Some(setting) = get(value) {
        return setting.name;
    }
    match value {
        0x40..=0x5F => "vehicleManufacturerSpecific",
        0x60..=0x7E => "systemSupplierSpecific",
        _ => "ISOSAEReserved",
    }
}

/// Build a ControlDTCSetting request PDU
/// (`0x85 <DTCSettingType> [DTCSettingControlOptionRecord...]`).
///
/// # Errors
/// Returns an error when `setting_type` is not a usable sub-function value `0x01`-`0x7E` (`0x00`
/// and `0x7F` are reserved, and bit 7 is the separate suppressPosRspMsgIndicationBit).
pub fn build_request(setting_type: u8, option_record: &[u8]) -> Result<Vec<u8>> {
    if !(0x01..=0x7E).contains(&setting_type) {
        return Err(Error::new(format!(
            "not a DTCSettingType (0x01-0x7E): {setting_type:#04x}"
        )));
    }
    let mut pdu = vec![CONTROL_DTC_SETTING, setting_type];
    pdu.extend_from_slice(option_record);
    Ok(pdu)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_request_frames_the_setting_type() {
        assert_eq!(
            build_request(DTC_SETTING_OFF, &[]).unwrap(),
            vec![0x85, 0x02]
        );
        assert_eq!(
            build_request(DTC_SETTING_ON, &[0xAB]).unwrap(),
            vec![0x85, 0x01, 0xAB]
        );
        assert!(build_request(0x00, &[]).is_err()); // reserved
        assert!(build_request(0x7F, &[]).is_err()); // reserved
        assert!(build_request(0x80, &[]).is_err()); // bit 7 is suppressPosRspMsgIndicationBit
    }

    #[test]
    fn setting_types_match_table_87() {
        assert_eq!(setting_types().len(), 2);
        assert_eq!(get(DTC_SETTING_ON).unwrap().mnemonic, "ON");
        assert_eq!(get(0x02).unwrap().name, "off");
        assert!(get(0x03).is_none());
        assert_eq!(describe(0x01), "on");
        assert_eq!(describe(0x50), "vehicleManufacturerSpecific");
        assert_eq!(describe(0x70), "systemSupplierSpecific");
        assert_eq!(describe(0x03), "ISOSAEReserved");
    }
}
