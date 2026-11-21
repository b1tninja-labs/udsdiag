//! RoutineControl (`0x31`): sub-functions, request building, and the routine catalog.
//!
//! RoutineControl runs a server-side routine named by a 2-byte routineIdentifier. The three
//! sub-functions are `0x01` startRoutine, `0x02` stopRoutine and `0x03` requestRoutineResults
//! (ISO 14229-1:2013, 13.2; startRoutine is mandatory, the other two user-optional).
//! [`build_request`] assembles `0x31 <sub-function> <routineId MSB> <routineId LSB> [data]`.
//!
//! The routine catalog is the set resolved in the GM GMG3 head-unit's `procdiagnosis` image
//! (`docs/sources.md`), each routine's on-wire ID and effect. These are GM-specific routines on
//! one module, not ISO-defined identifiers. `risk` flags the two sharpest ones: `0x3202`
//! StressTest (a fixed `execv("/usr/bin/stress", argv)` with an attacker-chosen argument vector
//! — resource-exhaustion DoS, not arbitrary command execution) and `0xD001`
//! SetSystemToDeliveryState (factory/shipping reset). All leaves are gated by SecurityAccess
//! (`0x27` -> ALD); see [`crate::security`].

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

/// One concrete routine on the assessed module.
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

const fn routine(
    routine_id: u16,
    name: &'static str,
    effect: &'static str,
    risk: &'static str,
) -> Routine {
    Routine {
        routine_id,
        name,
        effect,
        risk,
    }
}

// GMG3 head-unit procdiagnosis routines (docs/sources.md).
const CATALOG: &[Routine] = &[
    routine(
        0x3202,
        "StressTest",
        "fork()+execv(/usr/bin/stress, argv) with attacker-chosen args (DoS)",
        "H",
    ),
    routine(
        0xD001,
        "SetSystemToDeliveryState",
        "factory / shipping-state reset (de-personalise the unit)",
        "H",
    ),
    routine(0x3200, "RunIn", "end-of-line burn-in start", "M"),
    routine(0x3201, "RunInClearResults", "clear burn-in results", "M"),
    routine(
        0x3069,
        "GNSSCalcAndVerifyChecksum",
        "GNSS firmware CRC verify",
        "L",
    ),
    routine(0x3040, "IPODSelfTest", "iPod / USB self-test", "L"),
    routine(
        0x3204,
        "DisplayTestScreenSlideshow",
        "pushes display test patterns",
        "L",
    ),
];

/// The routine catalog.
pub const fn catalog() -> &'static [Routine] {
    CATALOG
}

/// The routine for an on-wire ID, or `None` when it is not catalogued.
pub fn get(routine_id: u16) -> Option<Routine> {
    CATALOG.iter().copied().find(|r| r.routine_id == routine_id)
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
    fn the_catalog_names_the_sharp_edges() {
        let stress = get(0x3202).expect("0x3202 catalogued");
        assert_eq!(stress.name, "StressTest");
        assert_eq!(stress.risk, "H");
        assert_eq!(get(0xD001).unwrap().name, "SetSystemToDeliveryState");
        assert!(get(0x0000).is_none());
        for r in catalog() {
            assert!(["H", "M", "L"].contains(&r.risk));
        }
    }

    #[test]
    fn subfunction_names() {
        assert_eq!(subfunction_name(0x01).unwrap(), "startRoutine");
        assert_eq!(subfunction_name(0x02).unwrap(), "stopRoutine");
        assert_eq!(subfunction_name(0x03).unwrap(), "requestRoutineResults");
        assert!(subfunction_name(0x04).is_err());
    }
}
