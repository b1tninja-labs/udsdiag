//! Data Identifier (DID) model and classification (ISO 14229-1:2013).
//!
//! A DID is the 2-byte identifier ReadDataByIdentifier (`0x22`) reads and WriteDataByIdentifier
//! (`0x2E`) writes. This module holds the ISO-generic machinery: the [`Did`] type, [`classify`] by
//! high byte, and [`parse_read_response`] for a positive `0x62` response. The concrete DID
//! *catalog* recovered from the Volt app is GM-specific and lives in the `gm` vendor layer
//! (`gm::read_dids` / `gm::write_dids`).

use crate::error::{Error, Result};

/// ReadDataByIdentifier service id.
pub const READ_DATA_BY_IDENTIFIER: u8 = 0x22;
/// ReadDataByIdentifier positive-response SID.
pub const READ_POSITIVE_RESPONSE: u8 = 0x62;
/// WriteDataByIdentifier service id.
pub const WRITE_DATA_BY_IDENTIFIER: u8 = 0x2E;

/// One Data Identifier and the category its high byte places it in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Did {
    /// The 2-byte identifier value.
    pub value: u16,
    /// The category label (see [`classify`]).
    pub category: &'static str,
}

impl Did {
    /// The 4-hex-digit identifier, e.g. `"41AD"`.
    pub fn hex(&self) -> String {
        format!("{:04X}", self.value)
    }
}

/// The category label for a DID, by its high byte.
pub fn classify(did: u16) -> &'static str {
    let high = did >> 8;
    match high {
        // 0x90xx identification; 0xF1xx is the ISO 14229-1 identification range (if ever present).
        0x90 | 0xF1 => "identification",
        0x41 | 0x42 => "gm-enhanced-dense",
        _ => "gm-enhanced",
    }
}

/// Split a positive ReadDataByIdentifier response (`0x62` DID value...).
///
/// Returns `(did, value_bytes)`.
///
/// # Errors
/// Returns an error if the PDU is not a `0x62` response of at least three bytes.
pub fn parse_read_response(pdu: &[u8]) -> Result<(u16, Vec<u8>)> {
    if pdu.len() < 3 || pdu[0] != READ_POSITIVE_RESPONSE {
        return Err(Error::new(
            "not a 0x62 ReadDataByIdentifier positive response",
        ));
    }
    let did = u16::from_be_bytes([pdu[1], pdu[2]]);
    Ok((did, pdu[3..].to_vec()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_names_the_ranges() {
        assert_eq!(classify(0x90B1), "identification");
        assert_eq!(classify(0x41AD), "gm-enhanced-dense");
        assert_eq!(classify(0x4202), "gm-enhanced-dense");
        assert_eq!(classify(0x8334), "gm-enhanced");
    }

    #[test]
    fn parse_read_response_splits_did_and_value() {
        assert_eq!(
            parse_read_response(&[0x62, 0x00, 0x2F, 0x6C]).unwrap(),
            (0x002F, vec![0x6C])
        );
        assert_eq!(
            parse_read_response(&[0x62, 0x41, 0xAD, 0x00, 0x00]).unwrap(),
            (0x41AD, vec![0x00, 0x00])
        );
    }
}
