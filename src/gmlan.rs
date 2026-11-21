//! GMLAN addressing conventions (GM single-wire / low-speed CAN).
//!
//! GM's GMLAN 11-bit addressing offsets a module's physical **response** CAN ID from its
//! **request** ID by `+0x400` and the flow/acceptance **filter** by `+0x300`: a request to
//! `0x241` is answered on `0x641` and accepted through filter `0x541` (`docs/sources.md`). The
//! generic ISO 15765-4 OBD-II layer is separate: the functional broadcast is `0x7DF` and a
//! physical request to `0x7E0..0x7E7` is answered on `0x7E8..0x7EF`.
//!
//! [`response_id`] and [`filter_id`] apply the GMLAN offsets; [`obd_response_id`] applies the
//! OBD-II physical offset. These are addressing helpers only — this crate does not open a CAN
//! interface.

use crate::error::{Error, Result};

/// The GMLAN physical response offset over a request ID.
pub const GMLAN_RESPONSE_OFFSET: u16 = 0x400;
/// The GMLAN flow/acceptance filter offset over a request ID.
pub const GMLAN_FILTER_OFFSET: u16 = 0x300;

/// The ISO 15765-4 OBD-II functional broadcast ID.
pub const OBD_FUNCTIONAL_BROADCAST: u16 = 0x7DF;
/// The lowest ISO 15765-4 OBD-II physical request ID.
pub const OBD_PHYSICAL_REQUEST_LOW: u16 = 0x7E0;
/// The highest ISO 15765-4 OBD-II physical request ID.
pub const OBD_PHYSICAL_REQUEST_HIGH: u16 = 0x7E7;
/// The ISO 15765-4 OBD-II physical response offset over a request ID.
pub const OBD_PHYSICAL_RESPONSE_OFFSET: u16 = 0x08;

fn check_11bit(can_id: u16) -> Result<()> {
    if can_id <= 0x7FF {
        Ok(())
    } else {
        Err(Error::new(format!("not an 11-bit CAN id: {can_id:#x}")))
    }
}

/// The GMLAN physical response ID for a request ID (request + `0x400`).
///
/// # Errors
/// Returns an error when the request ID, or the resulting response ID, is not 11-bit.
pub fn response_id(request_id: u16) -> Result<u16> {
    check_11bit(request_id)?;
    let result = request_id + GMLAN_RESPONSE_OFFSET;
    check_11bit(result)?;
    Ok(result)
}

/// The GMLAN flow/acceptance filter ID for a request ID (request + `0x300`).
///
/// # Errors
/// Returns an error when the request ID, or the resulting filter ID, is not 11-bit.
pub fn filter_id(request_id: u16) -> Result<u16> {
    check_11bit(request_id)?;
    let result = request_id + GMLAN_FILTER_OFFSET;
    check_11bit(result)?;
    Ok(result)
}

/// The ISO 15765-4 OBD-II physical response ID for a `0x7E0..0x7E7` request (request + 8).
///
/// # Errors
/// Returns an error when `request_id` is not an OBD-II physical request ID.
pub fn obd_response_id(request_id: u16) -> Result<u16> {
    if (OBD_PHYSICAL_REQUEST_LOW..=OBD_PHYSICAL_REQUEST_HIGH).contains(&request_id) {
        Ok(request_id + OBD_PHYSICAL_RESPONSE_OFFSET)
    } else {
        Err(Error::new(format!(
            "not an OBD-II physical request id: {request_id:#x}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_and_filter_offsets() {
        assert_eq!(response_id(0x241).unwrap(), 0x641);
        assert_eq!(filter_id(0x241).unwrap(), 0x541);
        assert_eq!(response_id(0x24C).unwrap(), 0x64C);
        assert!(response_id(0x800).is_err()); // not 11-bit
        assert!(response_id(0x7FF).is_err()); // +0x400 would overflow 11 bits
    }

    #[test]
    fn obd_physical_response_offset() {
        assert_eq!(obd_response_id(0x7E0).unwrap(), 0x7E8);
        assert_eq!(obd_response_id(0x7E7).unwrap(), 0x7EF);
        assert!(obd_response_id(0x7DF).is_err()); // functional broadcast, not a physical request
    }
}
