//! The sub-function byte: the suppressPosRspMsgIndicationBit and the sub-function value
//! (ISO 14229-1:2013, 8.2.2).
//!
//! Every service that takes a sub-function packs two things into one byte (Table 11): bit 7 is the
//! `suppressPosRspMsgIndicationBit` (SPRMIB) — `1` = the server shall not send a positive response —
//! and bits 6-0 are the sub-function parameter value (`0x00`-`0x7F`). The byte is
//! `SPRMIB | value` (Table 14), and a positive response echoes the value bits with bit 7 cleared
//! (8.3.2). These helpers set, test and strip bit 7 on top of the per-service builders in the
//! other modules.

/// Bit 7 of a sub-function byte: the suppressPosRspMsgIndicationBit (SPRMIB).
pub const SUPPRESS_POSITIVE_RESPONSE: u8 = 0x80;

/// Set the suppressPosRspMsgIndicationBit (bit 7) on a sub-function value so the server does not
/// send a positive response. The sub-function value bits (6-0) are preserved.
#[must_use]
pub fn with_suppress_positive_response(sub_function: u8) -> u8 {
    (sub_function & 0x7F) | SUPPRESS_POSITIVE_RESPONSE
}

/// Whether the suppressPosRspMsgIndicationBit (bit 7) is set on a sub-function byte.
#[must_use]
pub fn is_suppress_positive_response(sub_function: u8) -> bool {
    sub_function & SUPPRESS_POSITIVE_RESPONSE != 0
}

/// The sub-function parameter value (bits 6-0), with bit 7 stripped — the value a positive response
/// echoes (ISO 14229-1:2013, 8.3.2).
#[must_use]
pub fn sub_function_value(sub_function: u8) -> u8 {
    sub_function & 0x7F
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suppress_bit_sets_strips_and_tests() {
        // Set bit 7, preserving the value bits (Table 14).
        assert_eq!(with_suppress_positive_response(0x03), 0x83);
        // Idempotent and value-preserving.
        assert_eq!(with_suppress_positive_response(0x83), 0x83);
        assert!(is_suppress_positive_response(0x83));
        assert!(!is_suppress_positive_response(0x03));
        // Strip bit 7 to the echoed value (8.3.2).
        assert_eq!(sub_function_value(0x83), 0x03);
        assert_eq!(sub_function_value(0x03), 0x03);
    }
}
