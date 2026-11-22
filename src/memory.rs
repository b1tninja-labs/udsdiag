//! ReadMemoryByAddress (`0x23`) and WriteMemoryByAddress (`0x3D`): the addressAndLengthFormat-
//! Identifier and request building (ISO 14229-1:2013, 10.3 / 10.8; Annex H).
//!
//! Both services address memory by a raw `memoryAddress` and `memorySize`, each a big-endian byte
//! string whose length is carried in the one-byte `addressAndLengthFormatIdentifier` (ALFID): the
//! high nibble is the length of `memorySize`, the low nibble the length of `memoryAddress`.
//!
//! - ReadMemoryByAddress: `0x23 <ALFID> <memoryAddress> <memorySize>` ([`build_read_request`]).
//! - WriteMemoryByAddress: `0x3D <ALFID> <memoryAddress> <memorySize> <dataRecord>`
//!   ([`build_write_request`]).
//!
//! The builders compute the ALFID from the slice lengths. [`address_and_length_format_identifier`]
//! composes one directly, and [`memory_size_length`] / [`memory_address_length`] decode it.

use crate::error::{Error, Result};

/// ReadMemoryByAddress service id.
pub const READ_MEMORY_BY_ADDRESS: u8 = 0x23;
/// WriteMemoryByAddress service id.
pub const WRITE_MEMORY_BY_ADDRESS: u8 = 0x3D;

/// Compose an addressAndLengthFormatIdentifier from the byte lengths of the memoryAddress and
/// memorySize parameters (high nibble = `size_len`, low nibble = `address_len`).
///
/// # Errors
/// Returns an error when either length is outside `1`-`15` (a single nibble, at least one byte).
pub fn address_and_length_format_identifier(address_len: u8, size_len: u8) -> Result<u8> {
    if !(1..=15).contains(&address_len) || !(1..=15).contains(&size_len) {
        return Err(Error::new(format!(
            "address/size length must be 1-15 bytes: address={address_len}, size={size_len}"
        )));
    }
    Ok((size_len << 4) | address_len)
}

/// The memorySize length (bytes) encoded in an addressAndLengthFormatIdentifier (high nibble).
#[must_use]
pub fn memory_size_length(alfid: u8) -> u8 {
    alfid >> 4
}

/// The memoryAddress length (bytes) encoded in an addressAndLengthFormatIdentifier (low nibble).
#[must_use]
pub fn memory_address_length(alfid: u8) -> u8 {
    alfid & 0x0F
}

fn nibble_len(bytes: &[u8], what: &str) -> Result<u8> {
    u8::try_from(bytes.len())
        .ok()
        .filter(|&n| (1..=15).contains(&n))
        .ok_or_else(|| Error::new(format!("{what} must be 1-15 bytes, got {}", bytes.len())))
}

/// Build a ReadMemoryByAddress request PDU (`0x23 <ALFID> <memoryAddress> <memorySize>`), computing
/// the ALFID from the slice lengths. Both parameters are big-endian (MSB first).
///
/// # Errors
/// Returns an error when `memory_address` or `memory_size` is empty or longer than 15 bytes.
pub fn build_read_request(memory_address: &[u8], memory_size: &[u8]) -> Result<Vec<u8>> {
    let addr_len = nibble_len(memory_address, "memoryAddress")?;
    let size_len = nibble_len(memory_size, "memorySize")?;
    let alfid = (size_len << 4) | addr_len;
    let mut pdu = vec![READ_MEMORY_BY_ADDRESS, alfid];
    pdu.extend_from_slice(memory_address);
    pdu.extend_from_slice(memory_size);
    Ok(pdu)
}

/// Build a WriteMemoryByAddress request PDU
/// (`0x3D <ALFID> <memoryAddress> <memorySize> <dataRecord>`), computing the ALFID from the slice
/// lengths. The address and size are big-endian (MSB first).
///
/// # Errors
/// Returns an error when `memory_address`/`memory_size` is empty or longer than 15 bytes, or when
/// `data_record` is empty.
pub fn build_write_request(
    memory_address: &[u8],
    memory_size: &[u8],
    data_record: &[u8],
) -> Result<Vec<u8>> {
    let addr_len = nibble_len(memory_address, "memoryAddress")?;
    let size_len = nibble_len(memory_size, "memorySize")?;
    if data_record.is_empty() {
        return Err(Error::new(
            "WriteMemoryByAddress needs a non-empty dataRecord",
        ));
    }
    let alfid = (size_len << 4) | addr_len;
    let mut pdu = vec![WRITE_MEMORY_BY_ADDRESS, alfid];
    pdu.extend_from_slice(memory_address);
    pdu.extend_from_slice(memory_size);
    pdu.extend_from_slice(data_record);
    Ok(pdu)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alfid_composes_and_decodes() {
        // Annex H example: memorySize 2 bytes, memoryAddress 4 bytes -> 0x24.
        let alfid = address_and_length_format_identifier(4, 2).unwrap();
        assert_eq!(alfid, 0x24);
        assert_eq!(memory_address_length(alfid), 4);
        assert_eq!(memory_size_length(alfid), 2);
        assert!(address_and_length_format_identifier(0, 2).is_err());
        assert!(address_and_length_format_identifier(4, 16).is_err());
    }

    #[test]
    fn build_read_request_frames_address_and_size() {
        // 4-byte address, 2-byte size -> ALFID 0x24.
        assert_eq!(
            build_read_request(&[0x00, 0x20, 0x48, 0x13], &[0x01, 0x9A]).unwrap(),
            vec![0x23, 0x24, 0x00, 0x20, 0x48, 0x13, 0x01, 0x9A]
        );
        assert!(build_read_request(&[], &[0x01]).is_err());
        assert!(build_read_request(&[0x01], &[]).is_err());
    }

    #[test]
    fn build_write_request_frames_address_size_and_data() {
        // 2-byte address, 1-byte size -> ALFID 0x12, then the data record.
        assert_eq!(
            build_write_request(&[0x48, 0x13], &[0x05], &[0xAA, 0xBB]).unwrap(),
            vec![0x3D, 0x12, 0x48, 0x13, 0x05, 0xAA, 0xBB]
        );
        assert!(build_write_request(&[0x48, 0x13], &[0x05], &[]).is_err());
    }
}
