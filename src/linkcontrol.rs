//! LinkControl (`0x87`): linkControlType sub-functions, the linkControlModeIdentifier baud-rate
//! set and request building (ISO 14229-1:2013, 9.11; Annex B.3).
//!
//! LinkControl switches the data-link mode (e.g. baud rate) in a verify-then-transition sequence:
//!
//! - `0x01` verifyModeTransitionWithFixedParameter: `0x87 0x01 <linkControlModeIdentifier>`
//!   ([`build_verify_fixed`]) — the mode is a fixed entry from Annex B.3 ([`mode_identifiers`]).
//! - `0x02` verifyModeTransitionWithSpecificParameter: `0x87 0x02 <linkRecord (3-byte baud rate)>`
//!   ([`build_verify_specific`]).
//! - `0x03` transitionMode: `0x87 0x03` ([`build_transition`]).
//!
//! The sub-function byte can carry the `suppressPosRspMsgIndicationBit` (bit 7); that general
//! mechanism is a separate obligation.

use crate::error::{Error, Result};

/// LinkControl service id.
pub const LINK_CONTROL: u8 = 0x87;

/// verifyModeTransitionWithFixedParameter (`0x01`).
pub const VERIFY_MODE_TRANSITION_WITH_FIXED_PARAMETER: u8 = 0x01;
/// verifyModeTransitionWithSpecificParameter (`0x02`).
pub const VERIFY_MODE_TRANSITION_WITH_SPECIFIC_PARAMETER: u8 = 0x02;
/// transitionMode (`0x03`).
pub const TRANSITION_MODE: u8 = 0x03;

/// One defined linkControlType (ISO 14229-1:2013, Table 127).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinkControlType {
    /// The sub-function value.
    pub value: u8,
    /// The linkControlType name.
    pub name: &'static str,
    /// The linkControlType mnemonic.
    pub mnemonic: &'static str,
}

const fn control(value: u8, name: &'static str, mnemonic: &'static str) -> LinkControlType {
    LinkControlType {
        value,
        name,
        mnemonic,
    }
}

/// The linkControlType values defined by ISO 14229-1:2013 (Table 127).
pub const LINK_CONTROL_TYPES: &[LinkControlType] = &[
    control(
        VERIFY_MODE_TRANSITION_WITH_FIXED_PARAMETER,
        "verifyModeTransitionWithFixedParameter",
        "VMTWFP",
    ),
    control(
        VERIFY_MODE_TRANSITION_WITH_SPECIFIC_PARAMETER,
        "verifyModeTransitionWithSpecificParameter",
        "VMTWSP",
    ),
    control(TRANSITION_MODE, "transitionMode", "TM"),
];

/// Every defined linkControlType.
pub const fn link_control_types() -> &'static [LinkControlType] {
    LINK_CONTROL_TYPES
}

/// The defined linkControlType for a value, or `None` when it is reserved/range.
pub fn link_control_type(value: u8) -> Option<LinkControlType> {
    LINK_CONTROL_TYPES
        .iter()
        .copied()
        .find(|c| c.value == value)
}

/// A label for a linkControlType value, naming the range for an undefined one.
pub fn describe_type(value: u8) -> &'static str {
    if let Some(control) = link_control_type(value) {
        return control.name;
    }
    match value {
        0x40..=0x5F => "vehicleManufacturerSpecific",
        0x60..=0x7E => "systemSupplierSpecific",
        _ => "ISOSAEReserved",
    }
}

/// One defined linkControlModeIdentifier (ISO 14229-1:2013, Annex B.3, Table B.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModeIdentifier {
    /// The mode identifier value.
    pub value: u8,
    /// The mode name.
    pub name: &'static str,
    /// The mode mnemonic.
    pub mnemonic: &'static str,
}

const fn mode(value: u8, name: &'static str, mnemonic: &'static str) -> ModeIdentifier {
    ModeIdentifier {
        value,
        name,
        mnemonic,
    }
}

/// The linkControlModeIdentifier values defined by ISO 14229-1:2013 (Annex B.3, Table B.3).
pub const MODE_IDENTIFIERS: &[ModeIdentifier] = &[
    mode(0x01, "PC9600Baud", "PC9600"),
    mode(0x02, "PC19200Baud", "PC19200"),
    mode(0x03, "PC38400Baud", "PC38400"),
    mode(0x04, "PC57600Baud", "PC57600"),
    mode(0x05, "PC115200Baud", "PC115200"),
    mode(0x10, "CAN125000Baud", "CAN125000"),
    mode(0x11, "CAN250000Baud", "CAN250000"),
    mode(0x12, "CAN500000Baud", "CAN500000"),
    mode(0x13, "CAN1000000Baud", "CAN1000000"),
    mode(0x20, "ProgrammingSetup", "PROGSU"),
];

/// Every defined linkControlModeIdentifier.
pub const fn mode_identifiers() -> &'static [ModeIdentifier] {
    MODE_IDENTIFIERS
}

/// The defined linkControlModeIdentifier for a value, or `None` when it is reserved.
pub fn mode_identifier(value: u8) -> Option<ModeIdentifier> {
    MODE_IDENTIFIERS.iter().copied().find(|m| m.value == value)
}

/// Build a verifyModeTransitionWithFixedParameter request
/// (`0x87 0x01 <linkControlModeIdentifier>`).
#[must_use]
pub fn build_verify_fixed(mode_identifier: u8) -> Vec<u8> {
    vec![
        LINK_CONTROL,
        VERIFY_MODE_TRANSITION_WITH_FIXED_PARAMETER,
        mode_identifier,
    ]
}

/// Build a verifyModeTransitionWithSpecificParameter request
/// (`0x87 0x02 <linkRecord>`), where `baud_rate` is the 3-byte modeParameter baud rate.
///
/// # Errors
/// Returns an error when `baud_rate` does not fit the 3-byte linkRecord field (> `0xFFFFFF`).
pub fn build_verify_specific(baud_rate: u32) -> Result<Vec<u8>> {
    if baud_rate > 0x00FF_FFFF {
        return Err(Error::new(format!(
            "linkRecord baud rate is a 3-byte value (<= 0xFFFFFF): {baud_rate:#x}"
        )));
    }
    let [_, hi, mid, lo] = baud_rate.to_be_bytes();
    Ok(vec![
        LINK_CONTROL,
        VERIFY_MODE_TRANSITION_WITH_SPECIFIC_PARAMETER,
        hi,
        mid,
        lo,
    ])
}

/// Build a transitionMode request (`0x87 0x03`), applying the previously verified mode.
#[must_use]
pub fn build_transition() -> Vec<u8> {
    vec![LINK_CONTROL, TRANSITION_MODE]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogs_match_the_tables() {
        assert_eq!(link_control_types().len(), 3);
        assert_eq!(link_control_type(0x03).unwrap().mnemonic, "TM");
        assert_eq!(describe_type(0x50), "vehicleManufacturerSpecific");
        assert_eq!(describe_type(0x04), "ISOSAEReserved");
        assert_eq!(mode_identifiers().len(), 10);
        assert_eq!(mode_identifier(0x05).unwrap().name, "PC115200Baud");
        assert_eq!(mode_identifier(0x10).unwrap().mnemonic, "CAN125000");
        assert!(mode_identifier(0x06).is_none());
    }

    #[test]
    fn builders_frame_each_request() {
        assert_eq!(build_verify_fixed(0x05), vec![0x87, 0x01, 0x05]);
        // 115200 baud = 0x01C200.
        assert_eq!(
            build_verify_specific(0x0001_C200).unwrap(),
            vec![0x87, 0x02, 0x01, 0xC2, 0x00]
        );
        assert_eq!(build_transition(), vec![0x87, 0x03]);
        assert!(build_verify_specific(0x0100_0000).is_err());
    }
}
