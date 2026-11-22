//! ReadDTCInformation (`0x19`): the reportType sub-function catalog and request builders
//! (ISO 14229-1:2013, 11.3).
//!
//! ReadDTCInformation reads DTC status, snapshots, extended data, counts and lists, selected by a
//! 1-byte `reportType` sub-function. [`report_types`] / [`describe`] name the values (Table 269);
//! the builders frame the request by its parameter shape:
//!
//! - [`build_by_status_mask`] — `0x19 <reportType> <DTCStatusMask>` (reportTypes `0x01`, `0x02`,
//!   `0x0F`, `0x11`-`0x13`), pairing with the Annex D.2 status-byte decoder in [`crate::dtc`].
//! - [`build_by_dtc_and_record`] — `0x19 <reportType> <DTC high,mid,low> <recordNumber>`
//!   (snapshot / extended-data by DTC number, reportTypes `0x04`, `0x06`, `0x10`).
//! - [`build_request`] — the generic `0x19 <reportType> <parameters...>` for every other shape.
//!
//! The sub-function byte can carry the `suppressPosRspMsgIndicationBit` (bit 7); that general
//! mechanism is a separate obligation. This module frames the reportType in bits 6-0 only.

use crate::dtc::READ_DTC_INFORMATION;
use crate::error::{Error, Result};

/// reportType: reportNumberOfDTCByStatusMask (`0x01`).
pub const REPORT_NUMBER_OF_DTC_BY_STATUS_MASK: u8 = 0x01;
/// reportType: reportDTCByStatusMask (`0x02`).
pub const REPORT_DTC_BY_STATUS_MASK: u8 = 0x02;
/// reportType: reportDTCSnapshotRecordByDTCNumber (`0x04`).
pub const REPORT_DTC_SNAPSHOT_RECORD_BY_DTC_NUMBER: u8 = 0x04;
/// reportType: reportDTCExtDataRecordByDTCNumber (`0x06`).
pub const REPORT_DTC_EXT_DATA_RECORD_BY_DTC_NUMBER: u8 = 0x06;
/// reportType: reportSupportedDTC (`0x0A`).
pub const REPORT_SUPPORTED_DTC: u8 = 0x0A;
/// reportType: reportDTCWithPermanentStatus (`0x15`).
pub const REPORT_DTC_WITH_PERMANENT_STATUS: u8 = 0x15;

/// One defined reportType (ISO 14229-1:2013, Table 269).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReportType {
    /// The sub-function value.
    pub value: u8,
    /// The reportType name.
    pub name: &'static str,
    /// The reportType mnemonic.
    pub mnemonic: &'static str,
}

const fn report(value: u8, name: &'static str, mnemonic: &'static str) -> ReportType {
    ReportType {
        value,
        name,
        mnemonic,
    }
}

/// Every defined reportType (ISO 14229-1:2013, Table 269 and the request tables 256-268).
pub const REPORT_TYPES: &[ReportType] = &[
    report(0x01, "reportNumberOfDTCByStatusMask", "RNODTCBSM"),
    report(0x02, "reportDTCByStatusMask", "RDTCBSM"),
    report(0x03, "reportDTCSnapshotIdentification", "RDTCSSI"),
    report(0x04, "reportDTCSnapshotRecordByDTCNumber", "RDTCSSBDTC"),
    report(0x05, "reportDTCStoredDataByRecordNumber", "RDTCSDBRN"),
    report(0x06, "reportDTCExtDataRecordByDTCNumber", "RDTCEDRBDN"),
    report(0x07, "reportNumberOfDTCBySeverityMaskRecord", "RNODTCBSMR"),
    report(0x08, "reportDTCBySeverityMaskRecord", "RDTCBSMR"),
    report(0x09, "reportSeverityInformationOfDTC", "RSIODTC"),
    report(0x0A, "reportSupportedDTC", "RSUPDTC"),
    report(0x0B, "reportFirstTestFailedDTC", "RFTFDTC"),
    report(0x0C, "reportFirstConfirmedDTC", "RFCDTC"),
    report(0x0D, "reportMostRecentTestFailedDTC", "RMRTFDTC"),
    report(0x0E, "reportMostRecentConfirmedDTC", "RMRCDTC"),
    report(0x0F, "reportMirrorMemoryDTCByStatusMask", "RMMDTCBSM"),
    report(
        0x10,
        "reportMirrorMemoryDTCExtDataRecordByDTCNumber",
        "RMDEDRBDN",
    ),
    report(
        0x11,
        "reportNumberOfMirrorMemoryDTCByStatusMask",
        "RNOMMDTCBSM",
    ),
    report(
        0x12,
        "reportNumberOfEmissionsOBDDTCByStatusMask",
        "RNOOEBDDTCBSM",
    ),
    report(0x13, "reportEmissionsOBDDTCByStatusMask", "ROBDDTCBSM"),
    report(0x14, "reportDTCFaultDetectionCounter", "RDTCFDC"),
    report(0x15, "reportDTCWithPermanentStatus", "RDTCWPS"),
    report(0x16, "reportDTCExtDataRecordByRecordNumber", "RDTCEDRBR"),
    report(0x17, "reportUserDefMemoryDTCByStatusMask", "RUDMDTCBSM"),
    report(
        0x18,
        "reportUserDefMemoryDTCSnapshotRecordByDTCNumber",
        "RUDMDTCSSBDTC",
    ),
    report(
        0x19,
        "reportUserDefMemoryDTCExtDataRecordByDTCNumber",
        "RUDMDTCEDRBDN",
    ),
    report(0x42, "reportWWHOBDDTCByMaskRecord", "ROBDDTCBMR"),
    report(0x55, "reportWWHOBDDTCWithPermanentStatus", "RWWHOBDDTCWPS"),
];

/// Every defined reportType.
pub const fn report_types() -> &'static [ReportType] {
    REPORT_TYPES
}

/// The defined reportType for a value, or `None` when it is not defined.
pub fn get(value: u8) -> Option<ReportType> {
    REPORT_TYPES.iter().copied().find(|r| r.value == value)
}

/// A label for a reportType value: its name, or `ISOSAEReserved` when undefined.
pub fn describe(value: u8) -> &'static str {
    get(value).map_or("ISOSAEReserved", |r| r.name)
}

/// Build a generic ReadDTCInformation request PDU (`0x19 <reportType> <parameters...>`).
#[must_use]
pub fn build_request(report_type: u8, parameters: &[u8]) -> Vec<u8> {
    let mut pdu = vec![READ_DTC_INFORMATION, report_type];
    pdu.extend_from_slice(parameters);
    pdu
}

/// Build a status-mask ReadDTCInformation request (`0x19 <reportType> <DTCStatusMask>`), for the
/// reportTypes that take a single DTCStatusMask byte (`0x01`, `0x02`, `0x0F`, `0x11`-`0x13`).
#[must_use]
pub fn build_by_status_mask(report_type: u8, status_mask: u8) -> Vec<u8> {
    build_request(report_type, &[status_mask])
}

/// Build a by-DTC-number ReadDTCInformation request
/// (`0x19 <reportType> <DTC high,mid,low> <recordNumber>`), for the snapshot/extended-data
/// reportTypes (`0x04`, `0x06`, `0x10`).
///
/// # Errors
/// Returns an error when `dtc` does not fit the 3-byte DTCMaskRecord field (> `0xFFFFFF`).
pub fn build_by_dtc_and_record(report_type: u8, dtc: u32, record_number: u8) -> Result<Vec<u8>> {
    if dtc > 0x00FF_FFFF {
        return Err(Error::new(format!(
            "a DTC number is a 3-byte value (<= 0xFFFFFF): {dtc:#x}"
        )));
    }
    let [_, hi, mid, lo] = dtc.to_be_bytes();
    Ok(build_request(report_type, &[hi, mid, lo, record_number]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_types_match_table_269() {
        assert_eq!(report_types().len(), 27);
        assert_eq!(get(REPORT_DTC_BY_STATUS_MASK).unwrap().mnemonic, "RDTCBSM");
        assert_eq!(get(0x0A).unwrap().name, "reportSupportedDTC");
        assert_eq!(get(0x55).unwrap().mnemonic, "RWWHOBDDTCWPS");
        assert!(get(0x20).is_none());
        assert_eq!(describe(0x02), "reportDTCByStatusMask");
        assert_eq!(describe(0x20), "ISOSAEReserved");
    }

    #[test]
    fn builders_frame_each_request_shape() {
        // ISO 14229-1:2013 §11.3.5 Example: reportDTCByStatusMask with DTCStatusMask 0x08.
        assert_eq!(
            build_by_status_mask(REPORT_DTC_BY_STATUS_MASK, 0x08),
            vec![0x19, 0x02, 0x08]
        );
        // reportDTCSnapshotRecordByDTCNumber, DTC 0x0A0D00, record 0xFF (all records).
        assert_eq!(
            build_by_dtc_and_record(REPORT_DTC_SNAPSHOT_RECORD_BY_DTC_NUMBER, 0x000A_0D00, 0xFF)
                .unwrap(),
            vec![0x19, 0x04, 0x0A, 0x0D, 0x00, 0xFF]
        );
        // reportSupportedDTC takes no parameters.
        assert_eq!(build_request(REPORT_SUPPORTED_DTC, &[]), vec![0x19, 0x0A]);
        assert!(build_by_dtc_and_record(0x04, 0x0100_0000, 0x00).is_err());
    }
}
