//! DynamicallyDefineDataIdentifier (`0x2C`): definitionType sub-functions and request building
//! (ISO 14229-1:2013, 10.6).
//!
//! The client composes a dynamic dataIdentifier (`0xF2xx`/`0xF3xx`) out of other DIDs or memory
//! regions, or clears one. The definitionType sub-function selects the shape:
//!
//! - `0x01` defineByIdentifier: `0x2C 0x01 <DDDI> [<sourceDID><position><memorySize>]...`
//!   ([`build_define_by_identifier`]).
//! - `0x02` defineByMemoryAddress: `0x2C 0x02 <DDDI> <ALFID> [<memoryAddress><memorySize>]...`
//!   ([`build_define_by_memory_address`]).
//! - `0x03` clearDynamicallyDefinedDataIdentifier: `0x2C 0x03 [<DDDI>]` (no DDDI = clear all)
//!   ([`build_clear`]).
//!
//! The sub-function byte can carry the `suppressPosRspMsgIndicationBit` (bit 7); that general
//! mechanism is a separate obligation.

use crate::error::{Error, Result};

/// DynamicallyDefineDataIdentifier service id.
pub const DYNAMICALLY_DEFINE_DATA_IDENTIFIER: u8 = 0x2C;

/// defineByIdentifier (`0x01`).
pub const DEFINE_BY_IDENTIFIER: u8 = 0x01;
/// defineByMemoryAddress (`0x02`).
pub const DEFINE_BY_MEMORY_ADDRESS: u8 = 0x02;
/// clearDynamicallyDefinedDataIdentifier (`0x03`).
pub const CLEAR_DYNAMICALLY_DEFINED_DATA_IDENTIFIER: u8 = 0x03;

/// One defined definitionType (ISO 14229-1:2013, Tables 190-192).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DefinitionType {
    /// The sub-function value.
    pub value: u8,
    /// The definitionType name.
    pub name: &'static str,
    /// The definitionType mnemonic.
    pub mnemonic: &'static str,
}

const fn definition(value: u8, name: &'static str, mnemonic: &'static str) -> DefinitionType {
    DefinitionType {
        value,
        name,
        mnemonic,
    }
}

/// The definitionType values defined by ISO 14229-1:2013 (Tables 190-192).
pub const DEFINITION_TYPES: &[DefinitionType] = &[
    definition(DEFINE_BY_IDENTIFIER, "defineByIdentifier", "DBID"),
    definition(DEFINE_BY_MEMORY_ADDRESS, "defineByMemoryAddress", "DBMA"),
    definition(
        CLEAR_DYNAMICALLY_DEFINED_DATA_IDENTIFIER,
        "clearDynamicallyDefinedDataIdentifier",
        "CDDDID",
    ),
];

/// Every defined definitionType.
pub const fn definition_types() -> &'static [DefinitionType] {
    DEFINITION_TYPES
}

/// The defined definitionType for a value, or `None` when it is reserved.
pub fn get(value: u8) -> Option<DefinitionType> {
    DEFINITION_TYPES.iter().copied().find(|d| d.value == value)
}

/// A label for a definitionType value: its name, or `ISOSAEReserved` when undefined.
pub fn describe(value: u8) -> &'static str {
    get(value).map_or("ISOSAEReserved", |d| d.name)
}

/// One source for defineByIdentifier: a sourceDataIdentifier, its positionInSourceDataRecord
/// (`0x01`-based) and the number of bytes (`memorySize`) to copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceEntry {
    /// The sourceDataIdentifier the bytes are copied from.
    pub source_data_identifier: u16,
    /// The 1-based byte position within the source data record.
    pub position_in_source_data_record: u8,
    /// The number of bytes to copy.
    pub memory_size: u8,
}

/// Build a defineByIdentifier request
/// (`0x2C 0x01 <DDDI> [<sourceDID><position><memorySize>]...`).
///
/// # Errors
/// Returns an error when `sources` is empty.
pub fn build_define_by_identifier(dddi: u16, sources: &[SourceEntry]) -> Result<Vec<u8>> {
    if sources.is_empty() {
        return Err(Error::new(
            "defineByIdentifier needs at least one source entry",
        ));
    }
    let mut pdu = vec![DYNAMICALLY_DEFINE_DATA_IDENTIFIER, DEFINE_BY_IDENTIFIER];
    pdu.extend_from_slice(&dddi.to_be_bytes());
    for source in sources {
        pdu.extend_from_slice(&source.source_data_identifier.to_be_bytes());
        pdu.push(source.position_in_source_data_record);
        pdu.push(source.memory_size);
    }
    Ok(pdu)
}

fn nibble_len(bytes: &[u8], what: &str) -> Result<u8> {
    u8::try_from(bytes.len())
        .ok()
        .filter(|&n| (1..=15).contains(&n))
        .ok_or_else(|| Error::new(format!("{what} must be 1-15 bytes, got {}", bytes.len())))
}

/// Build a defineByMemoryAddress request
/// (`0x2C 0x02 <DDDI> <ALFID> [<memoryAddress><memorySize>]...`). The single ALFID is derived from
/// the first location's lengths; every location must share those lengths.
///
/// # Errors
/// Returns an error when `locations` is empty, a length is outside `1`-`15` bytes, or the locations
/// do not all share the first one's address and size lengths.
pub fn build_define_by_memory_address(dddi: u16, locations: &[(&[u8], &[u8])]) -> Result<Vec<u8>> {
    let Some((first_address, first_size)) = locations.first() else {
        return Err(Error::new(
            "defineByMemoryAddress needs at least one memory location",
        ));
    };
    let addr_len = nibble_len(first_address, "memoryAddress")?;
    let size_len = nibble_len(first_size, "memorySize")?;
    for (address, size) in locations {
        if address.len() != first_address.len() || size.len() != first_size.len() {
            return Err(Error::new(
                "all memory locations must share the first one's address and size lengths",
            ));
        }
    }
    let alfid = (size_len << 4) | addr_len;
    let mut pdu = vec![DYNAMICALLY_DEFINE_DATA_IDENTIFIER, DEFINE_BY_MEMORY_ADDRESS];
    pdu.extend_from_slice(&dddi.to_be_bytes());
    pdu.push(alfid);
    for (address, size) in locations {
        pdu.extend_from_slice(address);
        pdu.extend_from_slice(size);
    }
    Ok(pdu)
}

/// Build a clearDynamicallyDefinedDataIdentifier request (`0x2C 0x03 [<DDDI>]`). `None` clears all
/// dynamically defined identifiers in the server.
#[must_use]
pub fn build_clear(dddi: Option<u16>) -> Vec<u8> {
    let mut pdu = vec![
        DYNAMICALLY_DEFINE_DATA_IDENTIFIER,
        CLEAR_DYNAMICALLY_DEFINED_DATA_IDENTIFIER,
    ];
    if let Some(dddi) = dddi {
        pdu.extend_from_slice(&dddi.to_be_bytes());
    }
    pdu
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn definition_types_match_the_tables() {
        assert_eq!(definition_types().len(), 3);
        assert_eq!(get(DEFINE_BY_IDENTIFIER).unwrap().mnemonic, "DBID");
        assert_eq!(get(0x02).unwrap().name, "defineByMemoryAddress");
        assert!(get(0x04).is_none());
        assert_eq!(describe(0x03), "clearDynamicallyDefinedDataIdentifier");
        assert_eq!(describe(0x00), "ISOSAEReserved");
    }

    #[test]
    fn build_define_by_identifier_frames_sources() {
        let sources = [SourceEntry {
            source_data_identifier: 0x010A,
            position_in_source_data_record: 0x01,
            memory_size: 0x04,
        }];
        assert_eq!(
            build_define_by_identifier(0xF301, &sources).unwrap(),
            vec![0x2C, 0x01, 0xF3, 0x01, 0x01, 0x0A, 0x01, 0x04]
        );
        assert!(build_define_by_identifier(0xF301, &[]).is_err());
    }

    #[test]
    fn build_define_by_memory_address_frames_alfid_and_locations() {
        // 4-byte address, 2-byte size -> ALFID 0x24.
        assert_eq!(
            build_define_by_memory_address(0xF301, &[(&[0x00, 0x20, 0x48, 0x13], &[0x01, 0x9A])])
                .unwrap(),
            vec![
                0x2C, 0x02, 0xF3, 0x01, 0x24, 0x00, 0x20, 0x48, 0x13, 0x01, 0x9A
            ]
        );
        assert!(build_define_by_memory_address(0xF301, &[]).is_err());
        // Mismatched lengths across locations are rejected.
        assert!(
            build_define_by_memory_address(
                0xF301,
                &[(&[0x00, 0x20], &[0x01]), (&[0x00, 0x20, 0x48], &[0x01])]
            )
            .is_err()
        );
    }

    #[test]
    fn build_clear_frames_optional_dddi() {
        assert_eq!(build_clear(Some(0xF301)), vec![0x2C, 0x03, 0xF3, 0x01]);
        assert_eq!(build_clear(None), vec![0x2C, 0x03]);
    }
}
