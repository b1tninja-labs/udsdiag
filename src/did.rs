//! The Data Identifier (DID) catalog.
//!
//! A DID is the 2-byte identifier ReadDataByIdentifier (`0x22`) reads and WriteDataByIdentifier
//! (`0x2E`) writes. This catalog is the set a third-party Chevrolet Volt app (Voltage 2.3.0)
//! references, recovered from its process memory in demo mode (115 `0x22` DIDs, 2 `0x2E`
//! targets); see `docs/sources.md`. The catalog records which identifiers exist and the range
//! each sits in — not the GM-proprietary field semantics of the enhanced blocks, which are not
//! published and would need an owner-authorised vehicle to confirm.
//!
//! [`classify`] labels a DID by its high byte: `0x90xx` is the identification range
//! (VIN-adjacent, the enhanced counterpart of generic OBD Mode $09), `0x41xx`/`0x42xx` are the
//! dense GM manufacturer-enhanced blocks, and the rest are other GM-enhanced identifiers.
//! [`parse_read_response`] splits a positive `0x62` response into DID and value bytes.

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

/// Voltage 2.3.0 — the 115 ReadDataByIdentifier (`0x22`) DIDs, grouped by high byte
/// (`docs/sources.md`). The `0x62` demo pair 22002F -> 62002F6C confirmed 002F.
const READ_DIDS: &[u16] = &[
    0x002F, 0x40D4, 0x4184, 0x4185, 0x4186, 0x4187, 0x4188, 0x4189, 0x4190, 0x4191, 0x4192, 0x4196,
    0x4197, 0x4198, 0x4199, 0x419A, 0x419B, 0x419C, 0x419D, 0x419E, 0x41A6, 0x41AD, 0x41C5, 0x41C6,
    0x4202, 0x4203, 0x4204, 0x4205, 0x4206, 0x4207, 0x420B, 0x420C, 0x420D, 0x420E, 0x420F, 0x4210,
    0x4211, 0x4212, 0x4213, 0x4214, 0x4215, 0x4216, 0x4217, 0x4218, 0x4219, 0x421A, 0x421B, 0x421C,
    0x4220, 0x4221, 0x4222, 0x4223, 0x4224, 0x4225, 0x4226, 0x4227, 0x4228, 0x4229, 0x422A, 0x422B,
    0x422C, 0x422D, 0x422E, 0x4232, 0x4233, 0x4234, 0x4235, 0x4236, 0x4237, 0x4238, 0x4239, 0x423A,
    0x423B, 0x423C, 0x423D, 0x423E, 0x423F, 0x4240, 0x4241, 0x4242, 0x4243, 0x4244, 0x4245, 0x4246,
    0x4247, 0x4248, 0x4249, 0x424A, 0x424B, 0x424C, 0x424D, 0x424E, 0x424F, 0x4250, 0x4368, 0x4369,
    0x436B, 0x436C, 0x43AE, 0x43AF, 0x4470, 0x4531, 0x5005, 0x5006, 0x5007, 0x5008, 0x5247, 0x5248,
    0x82A3, 0x8334, 0x83E1, 0x83E2, 0x90B1, 0x90F8, 0x90FB,
];

/// Voltage 2.3.0 — the `0x2E` WriteDataByIdentifier target DIDs (coding writes).
const WRITE_DIDS: &[u16] = &[0xD21B, 0xD28E];

/// The catalogued ReadDataByIdentifier (`0x22`) DIDs, classified.
pub fn read_dids() -> Vec<Did> {
    READ_DIDS
        .iter()
        .map(|&value| Did {
            value,
            category: classify(value),
        })
        .collect()
}

/// The catalogued WriteDataByIdentifier (`0x2E`) target DIDs.
pub fn write_dids() -> Vec<Did> {
    WRITE_DIDS
        .iter()
        .map(|&value| Did {
            value,
            category: classify(value),
        })
        .collect()
}

/// How many read DIDs fall in each category.
pub fn count_by_category() -> std::collections::BTreeMap<&'static str, usize> {
    let mut counts = std::collections::BTreeMap::new();
    for &value in READ_DIDS {
        *counts.entry(classify(value)).or_insert(0) += 1;
    }
    counts
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
    fn the_catalog_has_115_read_dids() {
        let dids = read_dids();
        assert_eq!(dids.len(), 115);
        let values: Vec<u16> = dids.iter().map(|d| d.value).collect();
        let unique: std::collections::BTreeSet<u16> = values.iter().copied().collect();
        assert_eq!(values.len(), unique.len());
        assert!(values.contains(&0x002F));
        assert!(values.contains(&0x90B1));
        let write: std::collections::BTreeSet<u16> = write_dids().iter().map(|d| d.value).collect();
        assert_eq!(write, std::collections::BTreeSet::from([0xD21B, 0xD28E]));
    }

    #[test]
    fn classify_names_the_ranges() {
        assert_eq!(classify(0x90B1), "identification");
        assert_eq!(classify(0x41AD), "gm-enhanced-dense");
        assert_eq!(classify(0x4202), "gm-enhanced-dense");
        assert_eq!(classify(0x8334), "gm-enhanced");
        let counts = count_by_category();
        assert_eq!(counts["gm-enhanced-dense"], 92); // 22 in 41xx + 70 in 42xx
        assert_eq!(counts["identification"], 3);
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
