//! SecuredDataTransmission (`0x84`): request framing (ISO 14229-1:2013, 9.8).
//!
//! SecuredDataTransmission carries a message produced by the Security Sub-Layer (an extended
//! data-link security document outside ISO 14229-1). The request is simply
//! `0x84 <securityDataRequestRecord>`, where the record is opaque to this crate. [`build_request`]
//! frames the envelope; it does **not** secure, encrypt or sign anything — the record is supplied
//! already processed, consistent with this crate's no-key-material boundary.

use crate::error::{Error, Result};

/// SecuredDataTransmission service id.
pub const SECURED_DATA_TRANSMISSION: u8 = 0x84;

/// Build a SecuredDataTransmission request PDU (`0x84 <securityDataRequestRecord>`).
///
/// # Errors
/// Returns an error when `security_data_request_record` is empty (the record is mandatory).
pub fn build_request(security_data_request_record: &[u8]) -> Result<Vec<u8>> {
    if security_data_request_record.is_empty() {
        return Err(Error::new(
            "SecuredDataTransmission needs a non-empty securityDataRequestRecord",
        ));
    }
    let mut pdu = vec![SECURED_DATA_TRANSMISSION];
    pdu.extend_from_slice(security_data_request_record);
    Ok(pdu)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_request_frames_the_record() {
        assert_eq!(
            build_request(&[0x01, 0x02, 0x03]).unwrap(),
            vec![0x84, 0x01, 0x02, 0x03]
        );
        assert!(build_request(&[]).is_err());
    }
}
