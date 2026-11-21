//! The negative-response-code (NRC) catalog.
//!
//! Values and mnemonics are from ISO 14229-1:2013, Annex A (normative), Table A.1. A negative
//! response message is `0x7F <requestSID> <NRC>` (ISO 14229-1:2013, 7.5.6). `0x78`
//! (requestCorrectlyReceived-ResponsePending) is the one a client must keep waiting on rather
//! than treat as a final failure; [`is_response_pending`] recognises it.
//!
//! Only the codes this catalog needs are enumerated by name; the reserved and data-link ranges
//! are summarised. [`describe`] always returns a label, falling back to the range an unknown
//! value sits in.

/// The NRC a client treats as "keep waiting" rather than a final failure.
pub const REQUEST_CORRECTLY_RECEIVED_RESPONSE_PENDING: u8 = 0x78;

/// One negative response code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NegativeResponseCode {
    /// The code byte value.
    pub value: u8,
    /// The code name.
    pub name: &'static str,
    /// The code mnemonic.
    pub mnemonic: &'static str,
}

const fn code(value: u8, name: &'static str, mnemonic: &'static str) -> NegativeResponseCode {
    NegativeResponseCode {
        value,
        name,
        mnemonic,
    }
}

/// ISO 14229-1:2013 Table A.1 — the named, non-reserved codes used across services.
const CODES: &[NegativeResponseCode] = &[
    code(0x10, "generalReject", "GR"),
    code(0x11, "serviceNotSupported", "SNS"),
    code(0x12, "sub-functionNotSupported", "SFNS"),
    code(0x13, "incorrectMessageLengthOrInvalidFormat", "IMLOIF"),
    code(0x14, "responseTooLong", "RTL"),
    code(0x21, "busyRepeatRequest", "BRR"),
    code(0x22, "conditionsNotCorrect", "CNC"),
    code(0x24, "requestSequenceError", "RSE"),
    code(0x25, "noResponseFromSubnetComponent", "NRFSC"),
    code(0x26, "failurePreventsExecutionOfRequestedAction", "FPEORA"),
    code(0x31, "requestOutOfRange", "ROOR"),
    code(0x33, "securityAccessDenied", "SAD"),
    code(0x35, "invalidKey", "IK"),
    code(0x36, "exceedNumberOfAttempts", "ENOA"),
    code(0x37, "requiredTimeDelayNotExpired", "RTDNE"),
    code(0x70, "uploadDownloadNotAccepted", "UDNA"),
    code(0x71, "transferDataSuspended", "TDS"),
    code(0x72, "generalProgrammingFailure", "GPF"),
    code(0x73, "wrongBlockSequenceCounter", "WBSC"),
    code(0x78, "requestCorrectlyReceived-ResponsePending", "RCRRP"),
    code(0x7E, "sub-functionNotSupportedInActiveSession", "SFNSIAS"),
    code(0x7F, "serviceNotSupportedInActiveSession", "SNSIAS"),
];

/// Every named code in the catalog, in value order.
pub const fn all_codes() -> &'static [NegativeResponseCode] {
    CODES
}

/// The named code for a value, or `None` when the value is reserved or unnamed.
pub fn get(value: u8) -> Option<NegativeResponseCode> {
    CODES.iter().copied().find(|c| c.value == value)
}

/// A label for any NRC value, naming the reserved range when the value is unnamed.
pub fn describe(value: u8) -> String {
    if let Some(c) = get(value) {
        return c.name.to_owned();
    }
    match value {
        0x00 => "positiveResponse (reserved, not sent in a negative response)".to_owned(),
        0x38..=0x4F => "reservedByExtendedDataLinkSecurityDocument".to_owned(),
        0x80..=0xFF => "conditionsNotCorrect-specific (manufacturer, 0x80-0xFF)".to_owned(),
        _ => "ISOSAEReserved".to_owned(),
    }
}

/// Whether the NRC is `0x78`, which a client treats as "keep waiting", not a failure.
pub fn is_response_pending(value: u8) -> bool {
    value == REQUEST_CORRECTLY_RECEIVED_RESPONSE_PENDING
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_a1_values_are_named() {
        assert_eq!(get(0x31).unwrap().name, "requestOutOfRange");
        assert_eq!(get(0x35).unwrap().name, "invalidKey");
        assert_eq!(get(0x22).unwrap().name, "conditionsNotCorrect");
        assert_eq!(get(0x78).unwrap().mnemonic, "RCRRP");
        assert_eq!(get(0x11).unwrap().name, "serviceNotSupported");
    }

    #[test]
    fn describe_falls_back_to_the_range() {
        assert_eq!(describe(0x31), "requestOutOfRange");
        assert_eq!(describe(0x05), "ISOSAEReserved");
        assert_eq!(describe(0x40), "reservedByExtendedDataLinkSecurityDocument");
        assert!(describe(0x90).starts_with("conditionsNotCorrect-specific"));
        assert!(get(0x05).is_none());
    }

    #[test]
    fn response_pending_is_0x78() {
        assert!(is_response_pending(0x78));
        assert!(!is_response_pending(0x31));
    }
}
