//! ReadDataByPeriodicIdentifier (`0x2A`): transmissionMode and request building
//! (ISO 14229-1:2013, 10.5; Annex C.4).
//!
//! The client asks the server to transmit one or more `periodicDataIdentifier`s (each a 1-byte id)
//! at a chosen rate, or to stop. The request is `0x2A <transmissionMode> <periodicDataIdentifier#1>
//! [...#m]`. For the three send rates the first periodicDataIdentifier is mandatory; for
//! `stopSending` the ids are optional (none = stop all). [`transmission_modes`] / [`describe`]
//! name the rates (Table C.10); [`build_request`] frames the PDU.

use crate::error::{Error, Result};

/// ReadDataByPeriodicIdentifier service id.
pub const READ_DATA_BY_PERIODIC_IDENTIFIER: u8 = 0x2A;

/// sendAtSlowRate (`0x01`).
pub const SEND_AT_SLOW_RATE: u8 = 0x01;
/// sendAtMediumRate (`0x02`).
pub const SEND_AT_MEDIUM_RATE: u8 = 0x02;
/// sendAtFastRate (`0x03`).
pub const SEND_AT_FAST_RATE: u8 = 0x03;
/// stopSending (`0x04`).
pub const STOP_SENDING: u8 = 0x04;

/// One defined transmissionMode (ISO 14229-1:2013, Annex C.4, Table C.10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransmissionMode {
    /// The transmissionMode value.
    pub value: u8,
    /// The transmissionMode name.
    pub name: &'static str,
    /// The transmissionMode mnemonic.
    pub mnemonic: &'static str,
}

const fn mode(value: u8, name: &'static str, mnemonic: &'static str) -> TransmissionMode {
    TransmissionMode {
        value,
        name,
        mnemonic,
    }
}

/// The transmissionMode values defined by ISO 14229-1:2013 (Annex C.4, Table C.10).
pub const TRANSMISSION_MODES: &[TransmissionMode] = &[
    mode(SEND_AT_SLOW_RATE, "sendAtSlowRate", "SASR"),
    mode(SEND_AT_MEDIUM_RATE, "sendAtMediumRate", "SAMR"),
    mode(SEND_AT_FAST_RATE, "sendAtFastRate", "SAFR"),
    mode(STOP_SENDING, "stopSending", "SS"),
];

/// Every defined transmissionMode.
pub const fn transmission_modes() -> &'static [TransmissionMode] {
    TRANSMISSION_MODES
}

/// The defined transmissionMode for a value, or `None` when it is reserved.
pub fn get(value: u8) -> Option<TransmissionMode> {
    TRANSMISSION_MODES
        .iter()
        .copied()
        .find(|m| m.value == value)
}

/// A label for a transmissionMode value: its name, or `ISOSAEReserved` when undefined.
pub fn describe(value: u8) -> &'static str {
    get(value).map_or("ISOSAEReserved", |m| m.name)
}

/// Build a ReadDataByPeriodicIdentifier request PDU
/// (`0x2A <transmissionMode> <periodicDataIdentifier...>`).
///
/// # Errors
/// Returns an error when `transmission_mode` is not `0x01`-`0x04`, or when it is a send rate
/// (`0x01`-`0x03`) with no `periodic_ids` (the first periodicDataIdentifier is mandatory).
pub fn build_request(transmission_mode: u8, periodic_ids: &[u8]) -> Result<Vec<u8>> {
    match transmission_mode {
        SEND_AT_SLOW_RATE | SEND_AT_MEDIUM_RATE | SEND_AT_FAST_RATE => {
            if periodic_ids.is_empty() {
                return Err(Error::new(
                    "the first periodicDataIdentifier is mandatory for a send rate",
                ));
            }
        }
        STOP_SENDING => {}
        _ => {
            return Err(Error::new(format!(
                "not a transmissionMode (0x01-0x04): {transmission_mode:#04x}"
            )));
        }
    }
    let mut pdu = vec![READ_DATA_BY_PERIODIC_IDENTIFIER, transmission_mode];
    pdu.extend_from_slice(periodic_ids);
    Ok(pdu)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transmission_modes_match_table_c10() {
        assert_eq!(transmission_modes().len(), 4);
        assert_eq!(get(SEND_AT_MEDIUM_RATE).unwrap().mnemonic, "SAMR");
        assert_eq!(get(0x04).unwrap().name, "stopSending");
        assert!(get(0x05).is_none());
        assert_eq!(describe(0x03), "sendAtFastRate");
        assert_eq!(describe(0x00), "ISOSAEReserved");
    }

    #[test]
    fn build_request_frames_mode_and_ids() {
        // ISO 14229-1:2013 §10.5 example: sendAtMediumRate, periodicDataIdentifiers 0xE3 and 0x24.
        assert_eq!(
            build_request(SEND_AT_MEDIUM_RATE, &[0xE3, 0x24]).unwrap(),
            vec![0x2A, 0x02, 0xE3, 0x24]
        );
        // stopSending with no ids stops all scheduled transmissions.
        assert_eq!(build_request(STOP_SENDING, &[]).unwrap(), vec![0x2A, 0x04]);
        assert!(build_request(SEND_AT_FAST_RATE, &[]).is_err()); // a send rate needs an id
        assert!(build_request(0x05, &[0x01]).is_err()); // reserved transmissionMode
    }
}
