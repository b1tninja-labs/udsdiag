//! TesterPresent (`0x3E`): the keep-alive request (ISO 14229-1:2013, 9.6).
//!
//! TesterPresent indicates to the server that the client is still present, keeping a non-default
//! diagnostic session alive. It has exactly one sub-function value, `0x00` zeroSubFunction
//! (Table 66); `0x01`-`0x7F` are reserved, and the service has no request data parameters
//! (9.6.2.3). [`build_request`] frames the fixed `0x3E 0x00` request.
//!
//! A functionally addressed keep-alive usually sets the `suppressPosRspMsgIndicationBit` (bit 7,
//! giving `0x3E 0x80`); that general mechanism is a separate obligation (see `ROADMAP.md`), so this
//! builder frames the plain request.

/// TesterPresent service id.
pub const TESTER_PRESENT: u8 = 0x3E;
/// The only sub-function value: zeroSubFunction (`0x00`).
pub const ZERO_SUB_FUNCTION: u8 = 0x00;

/// Build a TesterPresent request PDU (`0x3E 0x00`).
#[must_use]
pub fn build_request() -> Vec<u8> {
    vec![TESTER_PRESENT, ZERO_SUB_FUNCTION]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_request_is_the_zero_sub_function() {
        assert_eq!(build_request(), vec![0x3E, 0x00]);
    }
}
