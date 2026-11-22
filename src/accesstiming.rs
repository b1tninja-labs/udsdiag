//! AccessTimingParameter (`0x83`): the timingParameterAccessType sub-functions and request
//! building (ISO 14229-1:2013, 9.7).
//!
//! AccessTimingParameter reads or sets the server's communication timing parameters. The request
//! is `0x83 <timingParameterAccessType> [TimingParameterRequestRecord...]`; the record is present
//! only for `setTimingParametersToGivenValues` (`0x04`) and is data-link specific.
//! [`access_types`] / [`describe`] name the four access types (Table 74) and [`build_request`]
//! frames the PDU.
//!
//! The sub-function byte can carry the `suppressPosRspMsgIndicationBit` (bit 7); that general
//! mechanism is a separate obligation. This module frames the access type in bits 6-0 only.

use crate::error::{Error, Result};

/// AccessTimingParameter service id.
pub const ACCESS_TIMING_PARAMETER: u8 = 0x83;

/// readExtendedTimingParameterSet (`0x01`).
pub const READ_EXTENDED_TIMING_PARAMETER_SET: u8 = 0x01;
/// setTimingParametersToDefaultValues (`0x02`).
pub const SET_TIMING_PARAMETERS_TO_DEFAULT_VALUES: u8 = 0x02;
/// readCurrentlyActiveTimingParameters (`0x03`).
pub const READ_CURRENTLY_ACTIVE_TIMING_PARAMETERS: u8 = 0x03;
/// setTimingParametersToGivenValues (`0x04`).
pub const SET_TIMING_PARAMETERS_TO_GIVEN_VALUES: u8 = 0x04;

/// One defined timingParameterAccessType (ISO 14229-1:2013, Table 74).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccessType {
    /// The sub-function value.
    pub value: u8,
    /// The access-type name.
    pub name: &'static str,
    /// The access-type mnemonic.
    pub mnemonic: &'static str,
}

const fn access(value: u8, name: &'static str, mnemonic: &'static str) -> AccessType {
    AccessType {
        value,
        name,
        mnemonic,
    }
}

/// The timingParameterAccessType values defined by ISO 14229-1:2013 (Table 74).
pub const ACCESS_TYPES: &[AccessType] = &[
    access(
        READ_EXTENDED_TIMING_PARAMETER_SET,
        "readExtendedTimingParameterSet",
        "RETPS",
    ),
    access(
        SET_TIMING_PARAMETERS_TO_DEFAULT_VALUES,
        "setTimingParametersToDefaultValues",
        "STPTDV",
    ),
    access(
        READ_CURRENTLY_ACTIVE_TIMING_PARAMETERS,
        "readCurrentlyActiveTimingParameters",
        "RCATP",
    ),
    access(
        SET_TIMING_PARAMETERS_TO_GIVEN_VALUES,
        "setTimingParametersToGivenValues",
        "STPTGV",
    ),
];

/// Every defined timingParameterAccessType.
pub const fn access_types() -> &'static [AccessType] {
    ACCESS_TYPES
}

/// The defined access type for a value, or `None` when it is reserved.
pub fn get(value: u8) -> Option<AccessType> {
    ACCESS_TYPES.iter().copied().find(|a| a.value == value)
}

/// A label for a timingParameterAccessType value: its name, or `ISOSAEReserved` when undefined.
pub fn describe(value: u8) -> &'static str {
    get(value).map_or("ISOSAEReserved", |a| a.name)
}

/// Build an AccessTimingParameter request PDU
/// (`0x83 <timingParameterAccessType> [TimingParameterRequestRecord...]`).
///
/// # Errors
/// Returns an error when `access_type` is not one of `0x01`-`0x04`, when a `timing_record` is
/// supplied for a type other than `setTimingParametersToGivenValues`, or when
/// `setTimingParametersToGivenValues` is missing its record.
pub fn build_request(access_type: u8, timing_record: &[u8]) -> Result<Vec<u8>> {
    if get(access_type).is_none() {
        return Err(Error::new(format!(
            "not a timingParameterAccessType (0x01-0x04): {access_type:#04x}"
        )));
    }
    let given = access_type == SET_TIMING_PARAMETERS_TO_GIVEN_VALUES;
    match (given, timing_record.is_empty()) {
        (true, true) => {
            return Err(Error::new(
                "setTimingParametersToGivenValues needs a TimingParameterRequestRecord",
            ));
        }
        (false, false) => {
            return Err(Error::new(
                "a TimingParameterRequestRecord is only valid for setTimingParametersToGivenValues",
            ));
        }
        _ => {}
    }
    let mut pdu = vec![ACCESS_TIMING_PARAMETER, access_type];
    pdu.extend_from_slice(timing_record);
    Ok(pdu)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn access_types_match_table_74() {
        assert_eq!(access_types().len(), 4);
        assert_eq!(
            get(READ_EXTENDED_TIMING_PARAMETER_SET).unwrap().mnemonic,
            "RETPS"
        );
        assert_eq!(get(0x04).unwrap().name, "setTimingParametersToGivenValues");
        assert!(get(0x05).is_none());
        assert_eq!(describe(0x03), "readCurrentlyActiveTimingParameters");
        assert_eq!(describe(0x00), "ISOSAEReserved");
    }

    #[test]
    fn build_request_frames_access_type_and_record() {
        assert_eq!(
            build_request(READ_CURRENTLY_ACTIVE_TIMING_PARAMETERS, &[]).unwrap(),
            vec![0x83, 0x03]
        );
        // setTimingParametersToGivenValues carries the (data-link specific) record.
        assert_eq!(
            build_request(SET_TIMING_PARAMETERS_TO_GIVEN_VALUES, &[0x12, 0x34]).unwrap(),
            vec![0x83, 0x04, 0x12, 0x34]
        );
        assert!(build_request(0x04, &[]).is_err()); // given-values needs a record
        assert!(build_request(0x03, &[0x01]).is_err()); // record only for given-values
        assert!(build_request(0x05, &[]).is_err()); // reserved
    }
}
