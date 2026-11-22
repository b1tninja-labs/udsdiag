//! InputOutputControlByIdentifier (`0x2F`, ISO 14229-1:2013, E.1).
//!
//! The service drives a server signal named by a 2-byte dataIdentifier. The first byte of the
//! controlOptionRecord is the inputOutputControlParameter: `0x00` returnControlToECU, `0x01`
//! resetToDefault, `0x02` freezeCurrentState, `0x03` shortTermAdjustment. [`build_request`]
//! assembles `0x2F <DID MSB> <DID LSB> <parameter> [controlState]`.
//!
//! The concrete signal enumeration extracted from the GM GMG3 firmware is GM-specific and lives in
//! the `gm` vendor layer (`gm::signal_groups`).

use crate::error::{Error, Result};

/// InputOutputControlByIdentifier service id.
pub const INPUT_OUTPUT_CONTROL_BY_IDENTIFIER: u8 = 0x2F;
/// GM DeviceControl alias service id.
pub const GM_DEVICE_CONTROL: u8 = 0xAE;

/// returnControlToECU parameter.
pub const RETURN_CONTROL_TO_ECU: u8 = 0x00;
/// resetToDefault parameter.
pub const RESET_TO_DEFAULT: u8 = 0x01;
/// freezeCurrentState parameter.
pub const FREEZE_CURRENT_STATE: u8 = 0x02;
/// shortTermAdjustment parameter.
pub const SHORT_TERM_ADJUSTMENT: u8 = 0x03;

const PARAMETERS: &[(u8, &str)] = &[
    (RETURN_CONTROL_TO_ECU, "returnControlToECU"),
    (RESET_TO_DEFAULT, "resetToDefault"),
    (FREEZE_CURRENT_STATE, "freezeCurrentState"),
    (SHORT_TERM_ADJUSTMENT, "shortTermAdjustment"),
];

/// The name of an inputOutputControlParameter byte.
///
/// # Errors
/// Returns an error when `parameter` is not one of `0x00`-`0x03`.
pub fn parameter_name(parameter: u8) -> Result<&'static str> {
    PARAMETERS
        .iter()
        .find(|&&(value, _)| value == parameter)
        .map(|&(_, name)| name)
        .ok_or_else(|| Error::new(format!("not an IOControl parameter: {parameter:#04x}")))
}

/// Build an InputOutputControlByIdentifier request PDU.
///
/// # Errors
/// Returns an error when `parameter` is not an inputOutputControlParameter.
pub fn build_request(did: u16, parameter: u8, control_state: &[u8]) -> Result<Vec<u8>> {
    if !PARAMETERS.iter().any(|&(value, _)| value == parameter) {
        return Err(Error::new(format!(
            "not an IOControl parameter: {parameter:#04x}"
        )));
    }
    let [hi, lo] = did.to_be_bytes();
    let mut pdu = vec![INPUT_OUTPUT_CONTROL_BY_IDENTIFIER, hi, lo, parameter];
    pdu.extend_from_slice(control_state);
    Ok(pdu)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_request_frames_the_signal() {
        assert_eq!(
            build_request(0x4531, SHORT_TERM_ADJUSTMENT, &[0x01]).unwrap(),
            vec![0x2F, 0x45, 0x31, 0x03, 0x01]
        );
        assert_eq!(
            build_request(0x4531, RETURN_CONTROL_TO_ECU, &[]).unwrap(),
            vec![0x2F, 0x45, 0x31, 0x00]
        );
        assert_eq!(parameter_name(0x03).unwrap(), "shortTermAdjustment");
        assert!(build_request(0x4531, 0x09, &[]).is_err());
    }
}
