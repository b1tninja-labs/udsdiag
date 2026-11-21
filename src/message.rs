//! The UDS/GMLAN request and response codec (application-layer PDUs, ISO 14229-1:2013, 7).
//!
//! This is the client model: it builds a request PDU and reads a response PDU. A request is a
//! service SID followed by its data. A positive response is the request SID raised by `0x40`
//! followed by the response data. A negative response is `0x7F`, the echoed request SID, and a
//! single NRC byte (ISO 14229-1:2013, 7.5.6).
//!
//! The codec works on the reassembled application PDU only. The transport that segments a PDU
//! larger than one CAN frame (ISO-TP / ISO 15765-2) is the concern of a bus layer this crate
//! does not include; see `docs/sources.md`.

use crate::error::{Error, Result};
use crate::{nrc, services};

/// The leading byte of a negative-response PDU (`0x7F`).
pub const NEGATIVE_RESPONSE_SID: u8 = services::NEGATIVE_RESPONSE_SID;

/// A diagnostic request PDU: a service SID and its data bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// The service identifier.
    pub sid: u8,
    /// The request data bytes.
    pub data: Vec<u8>,
}

impl Request {
    /// A request with the given SID and data.
    pub fn new(sid: u8, data: impl Into<Vec<u8>>) -> Self {
        Self {
            sid,
            data: data.into(),
        }
    }

    /// The on-wire request PDU.
    pub fn encode(&self) -> Vec<u8> {
        let mut pdu = Vec::with_capacity(1 + self.data.len());
        pdu.push(self.sid);
        pdu.extend_from_slice(&self.data);
        pdu
    }

    /// The positive-response SID this request expects.
    ///
    /// # Errors
    /// Returns an error when the SID has no positive-response SID in range (see
    /// [`services::positive_response_sid`]).
    pub fn expected_response_sid(&self) -> Result<u8> {
        services::positive_response_sid(self.sid)
    }
}

/// A positive response PDU: a response SID (request SID + `0x40`) and its data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositiveResponse {
    /// The response SID (request SID + `0x40`).
    pub response_sid: u8,
    /// The response data bytes.
    pub data: Vec<u8>,
}

impl PositiveResponse {
    /// The request SID this response answers.
    pub fn request_sid(&self) -> u8 {
        services::request_sid_for_response(self.response_sid)
    }

    /// Whether this response positively answers the given request.
    pub fn answers(&self, request: &Request) -> bool {
        request
            .expected_response_sid()
            .is_ok_and(|expected| self.response_sid == expected)
    }
}

/// A negative response PDU: `0x7F`, the request SID, and one NRC byte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NegativeResponse {
    /// The echoed request SID.
    pub request_sid: u8,
    /// The negative-response code.
    pub code: u8,
}

impl NegativeResponse {
    /// Whether the NRC is `0x78` (keep waiting), not a final failure.
    pub fn is_response_pending(&self) -> bool {
        nrc::is_response_pending(self.code)
    }

    /// `<service> rejected: <nrc name> (0x..)`.
    pub fn describe(&self) -> String {
        format!(
            "{} rejected: {} ({:#04x})",
            services::name_of(self.request_sid),
            nrc::describe(self.code),
            self.code
        )
    }
}

/// A decoded response: positive or negative.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    /// A positive response.
    Positive(PositiveResponse),
    /// A negative response.
    Negative(NegativeResponse),
}

impl Response {
    /// Whether this is a positive response.
    pub fn is_positive(&self) -> bool {
        matches!(self, Response::Positive(_))
    }
}

/// Frame a request PDU: the service SID followed by its data.
pub fn encode_request(sid: u8, data: &[u8]) -> Vec<u8> {
    Request::new(sid, data).encode()
}

/// Decode a response PDU into a positive or negative response.
///
/// # Errors
/// Returns an error on an empty PDU or a negative response that is not exactly three bytes.
pub fn decode_response(pdu: &[u8]) -> Result<Response> {
    let Some(&first) = pdu.first() else {
        return Err(Error::new("empty response pdu"));
    };
    if first == NEGATIVE_RESPONSE_SID {
        if pdu.len() != 3 {
            return Err(Error::new(format!(
                "negative response must be 3 bytes, got {}",
                pdu.len()
            )));
        }
        return Ok(Response::Negative(NegativeResponse {
            request_sid: pdu[1],
            code: pdu[2],
        }));
    }
    Ok(Response::Positive(PositiveResponse {
        response_sid: first,
        data: pdu[1..].to_vec(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_request_encodes_sid_then_data() {
        assert_eq!(encode_request(0x22, &[0x00, 0x2F]), vec![0x22, 0x00, 0x2F]);
        let request = Request::new(0x22, [0x00, 0x2F]);
        assert_eq!(request.encode(), vec![0x22, 0x00, 0x2F]);
        assert_eq!(request.expected_response_sid().unwrap(), 0x62);
    }

    #[test]
    fn a_positive_response_decodes_and_answers() {
        // 62 002F 6C — ReadDataByIdentifier positive response (demo capture).
        let Response::Positive(response) = decode_response(&[0x62, 0x00, 0x2F, 0x6C]).unwrap()
        else {
            panic!("expected positive");
        };
        assert!(Response::Positive(response.clone()).is_positive());
        assert_eq!(response.request_sid(), 0x22);
        assert_eq!(response.data, vec![0x00, 0x2F, 0x6C]);
        assert!(response.answers(&Request::new(0x22, [0x00, 0x2F])));
    }

    #[test]
    fn a_negative_response_decodes_with_its_nrc() {
        // 7F 22 31 — ReadDataByIdentifier rejected, requestOutOfRange.
        let Response::Negative(response) = decode_response(&[0x7F, 0x22, 0x31]).unwrap() else {
            panic!("expected negative");
        };
        assert_eq!(response.request_sid, 0x22);
        assert_eq!(response.code, 0x31);
        assert!(response.describe().contains("requestOutOfRange"));
        assert!(decode_response(&[0x7F, 0x22]).is_err()); // malformed: 2 bytes
        assert!(decode_response(&[]).is_err());
    }

    #[test]
    fn response_pending_is_recognised() {
        let Response::Negative(response) = decode_response(&[0x7F, 0x1A, 0x78]).unwrap() else {
            panic!("expected negative");
        };
        assert!(response.is_response_pending());
    }
}
