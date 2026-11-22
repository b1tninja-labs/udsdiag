//! The Upload/Download functional unit (`0x34`-`0x38`): request framing (ISO 14229-1:2013, 14).
//!
//! These services move a block of data to or from the server. This module frames the *requests*
//! only — it performs no transfer and writes nothing:
//!
//! - RequestDownload (`0x34`) / RequestUpload (`0x35`): `<SID> <dataFormatIdentifier> <ALFID>
//!   <memoryAddress> <memorySize>` ([`build_request_download`] / [`build_request_upload`]).
//! - TransferData (`0x36`): `0x36 <blockSequenceCounter> [<transferRequestParameterRecord>]`
//!   ([`build_transfer_data`]).
//! - RequestTransferExit (`0x37`): `0x37 [<transferRequestParameterRecord>]`
//!   ([`build_request_transfer_exit`]).
//! - RequestFileTransfer (`0x38`): `0x38 <modeOfOperation> <filePathAndNameLength> <filePathAndName>
//!   [<mode-dependent trailer>]` ([`build_file_transfer`]).
//!
//! The actual flashing/programming flow is out of scope (the sibling `spskit` drives it).

use crate::error::{Error, Result};

/// RequestDownload service id.
pub const REQUEST_DOWNLOAD: u8 = 0x34;
/// RequestUpload service id.
pub const REQUEST_UPLOAD: u8 = 0x35;
/// TransferData service id.
pub const TRANSFER_DATA: u8 = 0x36;
/// RequestTransferExit service id.
pub const REQUEST_TRANSFER_EXIT: u8 = 0x37;
/// RequestFileTransfer service id.
pub const REQUEST_FILE_TRANSFER: u8 = 0x38;

/// A dataFormatIdentifier with neither compression nor encryption (`0x00`).
pub const NO_COMPRESSION_NO_ENCRYPTION: u8 = 0x00;

/// Compose a dataFormatIdentifier: high nibble `compression`, low nibble `encryption` (`0x00` =
/// neither; other values are vehicle-manufacturer specific).
#[must_use]
pub fn data_format_identifier(compression: u8, encryption: u8) -> u8 {
    ((compression & 0x0F) << 4) | (encryption & 0x0F)
}

fn nibble_len(bytes: &[u8], what: &str) -> Result<u8> {
    u8::try_from(bytes.len())
        .ok()
        .filter(|&n| (1..=15).contains(&n))
        .ok_or_else(|| Error::new(format!("{what} must be 1-15 bytes, got {}", bytes.len())))
}

fn build_download_upload(
    sid: u8,
    data_format_identifier: u8,
    memory_address: &[u8],
    memory_size: &[u8],
) -> Result<Vec<u8>> {
    let addr_len = nibble_len(memory_address, "memoryAddress")?;
    let size_len = nibble_len(memory_size, "memorySize")?;
    let alfid = (size_len << 4) | addr_len;
    let mut pdu = vec![sid, data_format_identifier, alfid];
    pdu.extend_from_slice(memory_address);
    pdu.extend_from_slice(memory_size);
    Ok(pdu)
}

/// Build a RequestDownload request
/// (`0x34 <dataFormatIdentifier> <ALFID> <memoryAddress> <memorySize>`, Table 393). The ALFID is
/// derived from the slice lengths.
///
/// # Errors
/// Returns an error when `memory_address` or `memory_size` is empty or longer than 15 bytes.
pub fn build_request_download(
    data_format_identifier: u8,
    memory_address: &[u8],
    memory_size: &[u8],
) -> Result<Vec<u8>> {
    build_download_upload(
        REQUEST_DOWNLOAD,
        data_format_identifier,
        memory_address,
        memory_size,
    )
}

/// Build a RequestUpload request
/// (`0x35 <dataFormatIdentifier> <ALFID> <memoryAddress> <memorySize>`, same shape as download).
///
/// # Errors
/// Returns an error when `memory_address` or `memory_size` is empty or longer than 15 bytes.
pub fn build_request_upload(
    data_format_identifier: u8,
    memory_address: &[u8],
    memory_size: &[u8],
) -> Result<Vec<u8>> {
    build_download_upload(
        REQUEST_UPLOAD,
        data_format_identifier,
        memory_address,
        memory_size,
    )
}

/// Build a TransferData request (`0x36 <blockSequenceCounter> [<transferRequestParameterRecord>]`).
/// The first TransferData after a RequestDownload/Upload uses a blockSequenceCounter of 1.
#[must_use]
pub fn build_transfer_data(block_sequence_counter: u8, data: &[u8]) -> Vec<u8> {
    let mut pdu = vec![TRANSFER_DATA, block_sequence_counter];
    pdu.extend_from_slice(data);
    pdu
}

/// Build a RequestTransferExit request (`0x37 [<transferRequestParameterRecord>]`).
#[must_use]
pub fn build_request_transfer_exit(parameter_record: &[u8]) -> Vec<u8> {
    let mut pdu = vec![REQUEST_TRANSFER_EXIT];
    pdu.extend_from_slice(parameter_record);
    pdu
}

/// AddFile (`0x01`).
pub const ADD_FILE: u8 = 0x01;
/// DeleteFile (`0x02`).
pub const DELETE_FILE: u8 = 0x02;
/// ReplaceFile (`0x03`).
pub const REPLACE_FILE: u8 = 0x03;
/// ReadFile (`0x04`).
pub const READ_FILE: u8 = 0x04;
/// ReadDir (`0x05`).
pub const READ_DIR: u8 = 0x05;

/// One defined RequestFileTransfer modeOfOperation (ISO 14229-1:2013, Annex G, Table G.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModeOfOperation {
    /// The modeOfOperation value.
    pub value: u8,
    /// The mode name.
    pub name: &'static str,
}

const fn mode(value: u8, name: &'static str) -> ModeOfOperation {
    ModeOfOperation { value, name }
}

/// The modeOfOperation values defined by ISO 14229-1:2013 (Annex G, Table G.1).
pub const MODES_OF_OPERATION: &[ModeOfOperation] = &[
    mode(ADD_FILE, "AddFile"),
    mode(DELETE_FILE, "DeleteFile"),
    mode(REPLACE_FILE, "ReplaceFile"),
    mode(READ_FILE, "ReadFile"),
    mode(READ_DIR, "ReadDir"),
];

/// Every defined modeOfOperation.
pub const fn modes_of_operation() -> &'static [ModeOfOperation] {
    MODES_OF_OPERATION
}

/// The defined modeOfOperation for a value, or `None` when it is reserved.
pub fn mode_of_operation(value: u8) -> Option<ModeOfOperation> {
    MODES_OF_OPERATION
        .iter()
        .copied()
        .find(|m| m.value == value)
}

/// Build a RequestFileTransfer request
/// (`0x38 <modeOfOperation> <filePathAndNameLength (2 bytes)> <filePathAndName> [<trailer>]`).
///
/// `trailer` carries the mode-dependent parameters (for AddFile / ReplaceFile: dataFormatIdentifier,
/// fileSizeParameterLength and the file sizes); it is empty for DeleteFile / ReadFile / ReadDir.
///
/// # Errors
/// Returns an error when `mode_of_operation` is not `0x01`-`0x05`, or when `file_path` is empty or
/// longer than 65535 bytes.
pub fn build_file_transfer(
    mode_of_operation: u8,
    file_path: &[u8],
    trailer: &[u8],
) -> Result<Vec<u8>> {
    if mode_of_operation_is_reserved(mode_of_operation) {
        return Err(Error::new(format!(
            "not a modeOfOperation (0x01-0x05): {mode_of_operation:#04x}"
        )));
    }
    let path_len = u16::try_from(file_path.len())
        .ok()
        .filter(|&n| n >= 1)
        .ok_or_else(|| Error::new("filePathAndName must be 1-65535 bytes"))?;
    let mut pdu = vec![REQUEST_FILE_TRANSFER, mode_of_operation];
    pdu.extend_from_slice(&path_len.to_be_bytes());
    pdu.extend_from_slice(file_path);
    pdu.extend_from_slice(trailer);
    Ok(pdu)
}

fn mode_of_operation_is_reserved(value: u8) -> bool {
    mode_of_operation(value).is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_and_upload_frame_dfi_alfid_and_memory() {
        // dataFormatIdentifier 0x00, 4-byte address, 2-byte size -> ALFID 0x24.
        assert_eq!(
            build_request_download(0x00, &[0x00, 0x20, 0x48, 0x13], &[0x01, 0x9A]).unwrap(),
            vec![0x34, 0x00, 0x24, 0x00, 0x20, 0x48, 0x13, 0x01, 0x9A]
        );
        assert_eq!(
            build_request_upload(0x00, &[0x48, 0x13], &[0x05]).unwrap()[0],
            0x35
        );
        assert_eq!(data_format_identifier(0x1, 0x2), 0x12);
        assert!(build_request_download(0x00, &[], &[0x01]).is_err());
    }

    #[test]
    fn transfer_data_and_exit_frame_their_records() {
        assert_eq!(
            build_transfer_data(0x01, &[0xAA, 0xBB]),
            vec![0x36, 0x01, 0xAA, 0xBB]
        );
        assert_eq!(build_transfer_data(0x05, &[]), vec![0x36, 0x05]);
        assert_eq!(build_request_transfer_exit(&[]), vec![0x37]);
        assert_eq!(build_request_transfer_exit(&[0x01]), vec![0x37, 0x01]);
    }

    #[test]
    fn file_transfer_frames_mode_path_and_trailer() {
        // ReadFile: 0x38 04 <len=0x0003> "abc", no trailer.
        assert_eq!(
            build_file_transfer(READ_FILE, b"abc", &[]).unwrap(),
            vec![0x38, 0x04, 0x00, 0x03, b'a', b'b', b'c']
        );
        assert_eq!(modes_of_operation().len(), 5);
        assert_eq!(mode_of_operation(0x01).unwrap().name, "AddFile");
        assert!(mode_of_operation(0x06).is_none());
        assert!(build_file_transfer(0x06, b"x", &[]).is_err()); // reserved mode
        assert!(build_file_transfer(READ_FILE, b"", &[]).is_err()); // empty path
    }
}
