//! RoutineControl (`0x31`): sub-functions and request building (ISO 14229-1:2013, 13.2).
//!
//! RoutineControl runs a server-side routine named by a 2-byte routineIdentifier. The three
//! sub-functions are `0x01` startRoutine, `0x02` stopRoutine and `0x03` requestRoutineResults
//! (startRoutine is mandatory, the other two user-optional). [`build_request`] assembles
//! `0x31 <sub-function> <routineId MSB> <routineId LSB> [data]`.
//!
//! The concrete routine *catalog* resolved in the GM GMG3 head unit is GM-specific and lives in
//! the `gm` vendor layer (`gm::routines`). The [`Routine`] type lives here so both share it.

use crate::error::{Error, Result};

/// RoutineControl service id.
pub const ROUTINE_CONTROL: u8 = 0x31;

/// startRoutine sub-function.
pub const START_ROUTINE: u8 = 0x01;
/// stopRoutine sub-function.
pub const STOP_ROUTINE: u8 = 0x02;
/// requestRoutineResults sub-function.
pub const REQUEST_ROUTINE_RESULTS: u8 = 0x03;

const SUBFUNCTIONS: &[(u8, &str)] = &[
    (START_ROUTINE, "startRoutine"),
    (STOP_ROUTINE, "stopRoutine"),
    (REQUEST_ROUTINE_RESULTS, "requestRoutineResults"),
];

/// The name of a RoutineControl sub-function byte.
///
/// # Errors
/// Returns an error when `subfunction` is not one of `0x01`, `0x02`, `0x03`.
pub fn subfunction_name(subfunction: u8) -> Result<&'static str> {
    SUBFUNCTIONS
        .iter()
        .find(|&&(value, _)| value == subfunction)
        .map(|&(_, name)| name)
        .ok_or_else(|| {
            Error::new(format!(
                "not a RoutineControl sub-function: {subfunction:#04x}"
            ))
        })
}

/// Build a RoutineControl request PDU.
///
/// # Errors
/// Returns an error when `subfunction` is not a RoutineControl sub-function.
pub fn build_request(subfunction: u8, routine_id: u16, data: &[u8]) -> Result<Vec<u8>> {
    if !SUBFUNCTIONS.iter().any(|&(value, _)| value == subfunction) {
        return Err(Error::new(format!(
            "not a RoutineControl sub-function: {subfunction:#04x}"
        )));
    }
    let [hi, lo] = routine_id.to_be_bytes();
    let mut pdu = vec![ROUTINE_CONTROL, subfunction, hi, lo];
    pdu.extend_from_slice(data);
    Ok(pdu)
}

/// One concrete routine on an assessed module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Routine {
    /// The on-wire routine identifier.
    pub routine_id: u16,
    /// The routine name.
    pub name: &'static str,
    /// What running the routine does.
    pub effect: &'static str,
    /// Risk flag: `"H"`, `"M"` or `"L"`.
    pub risk: &'static str,
}

impl Routine {
    /// The 4-hex-digit routine identifier.
    pub fn hex(&self) -> String {
        format!("{:04X}", self.routine_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_request_frames_the_routine() {
        assert_eq!(
            build_request(START_ROUTINE, 0x3202, &[]).unwrap(),
            vec![0x31, 0x01, 0x32, 0x02]
        );
        assert_eq!(
            build_request(0x03, 0x5247, &[0x01]).unwrap(),
            vec![0x31, 0x03, 0x52, 0x47, 0x01]
        );
        assert!(build_request(0x09, 0x3202, &[]).is_err());
    }

    #[test]
    fn subfunction_names() {
        assert_eq!(subfunction_name(0x01).unwrap(), "startRoutine");
        assert_eq!(subfunction_name(0x02).unwrap(), "stopRoutine");
        assert_eq!(subfunction_name(0x03).unwrap(), "requestRoutineResults");
        assert!(subfunction_name(0x04).is_err());
    }
}
