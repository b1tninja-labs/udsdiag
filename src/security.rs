//! SecurityAccess (`0x27`): the level-pairing model and two mechanism descriptors.
//!
//! SecurityAccess unlocks protected services. It is a two-step exchange: the client sends
//! requestSeed (an odd sub-function), the server returns a seed, the client sends sendKey (the
//! next even sub-function) with a key derived from the seed, and the server unlocks or answers
//! NRC `0x35` invalidKey (ISO 14229-1:2013, 9.4). A fixed relationship binds each requestSeed to
//! its sendKey: requestSeed `0x01` to sendKey `0x02`, `0x03` to `0x04`, and so on.
//!
//! # Scope boundary
//! This module models the *exchange shape* only. It contains no seed->key algorithm, no key
//! material, and no key-derivation method, and it will not compute a key. That is out of scope by
//! design (`docs/security-access.md`). [`MECHANISMS`] describes the two mechanisms the assessment
//! distinguished — the classic GM seed/key (a per-module secret held by GM) and the GMG3 head
//! unit's ADIT ALD, an RSA-2048 signed challenge/response — as reference descriptors, not as
//! anything runnable.

use crate::error::{Error, Result};

/// SecurityAccess service id.
pub const SECURITY_ACCESS: u8 = 0x27;

fn check_level(level: u8) -> Result<()> {
    if (0x01..=0x7E).contains(&level) {
        Ok(())
    } else {
        Err(Error::new(format!(
            "security-access level out of range: {level:#04x}"
        )))
    }
}

/// Whether a sub-function byte is a requestSeed (odd, ISO 14229-1:2013, 9.4.2.2).
///
/// # Errors
/// Returns an error when `level` is outside `0x01`-`0x7E`.
pub fn is_request_seed(level: u8) -> Result<bool> {
    check_level(level)?;
    Ok(level % 2 == 1)
}

/// Whether a sub-function byte is a sendKey (even, the partner of a requestSeed).
///
/// # Errors
/// Returns an error when `level` is outside `0x01`-`0x7E`.
pub fn is_send_key(level: u8) -> Result<bool> {
    check_level(level)?;
    Ok(level % 2 == 0)
}

/// The sendKey sub-function paired with a requestSeed sub-function.
///
/// # Errors
/// Returns an error when `level` is not a requestSeed (odd, in range).
pub fn send_key_for(level: u8) -> Result<u8> {
    if is_request_seed(level)? {
        Ok(level + 1)
    } else {
        Err(Error::new(format!("not a requestSeed level: {level:#04x}")))
    }
}

/// The requestSeed sub-function paired with a sendKey sub-function.
///
/// # Errors
/// Returns an error when `level` is not a sendKey (even, in range).
pub fn request_seed_for(level: u8) -> Result<u8> {
    if is_send_key(level)? {
        Ok(level - 1)
    } else {
        Err(Error::new(format!("not a sendKey level: {level:#04x}")))
    }
}

/// Build a SecurityAccess requestSeed PDU (`0x27 <level>`).
///
/// This is the only PDU builder in the module. There is deliberately no builder that computes or
/// sends a key.
///
/// # Errors
/// Returns an error when `level` is not a requestSeed level.
pub fn build_request_seed(level: u8) -> Result<Vec<u8>> {
    if is_request_seed(level)? {
        Ok(vec![SECURITY_ACCESS, level])
    } else {
        Err(Error::new(format!("not a requestSeed level: {level:#04x}")))
    }
}

/// A description of how a server verifies a SecurityAccess key. Reference only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mechanism {
    /// The mechanism name.
    pub name: &'static str,
    /// The mechanism family: `"seed-key"` or `"challenge-response"`.
    pub family: &'static str,
    /// A prose summary of the mechanism. Carries no key material.
    pub summary: &'static str,
    /// The NRC a bad key draws (`0x35` invalidKey).
    pub invalid_key_nrc: u8,
}

/// The lookup key for the classic GM seed/key mechanism.
pub const CLASSIC_GM_SEED_KEY: &str = "classic-gm-seed-key";
/// The lookup key for the ADIT ALD mechanism.
pub const ADIT_ALD: &str = "adit-ald";

/// Reference descriptors (`docs/security-access.md`). Neither carries key material or an algorithm.
pub const MECHANISMS: &[(&str, Mechanism)] = &[
    (
        CLASSIC_GM_SEED_KEY,
        Mechanism {
            name: "Classic GM seed/key",
            family: "seed-key",
            summary: "A module returns a short seed; the tester returns a key computed from it by a \
                      per-module secret algorithm held by GM. The algorithm and keys are not in scope.",
            invalid_key_nrc: 0x35,
        },
    ),
    (
        ADIT_ALD,
        Mechanism {
            name: "ADIT ALD (GMG3 head unit)",
            family: "challenge-response",
            summary: "SecurityAccess 0x27 is a front-end over the ADIT Access-Level Daemon. The server \
                      returns a 96-byte device-bound challenge (16-byte rand() nonce, 8-byte timestamp, \
                      8-byte i.MX6 serial, 16-byte caller ECU_ID). The tester returns an RSA-2048 signed \
                      block; the daemon verifies the signature against the per-level public key (level.pub) \
                      and memcmp's the response against the live challenge. A bad key is NRC 0x35. The \
                      per-level private keys are held by GM/Bosch and are out of scope.",
            invalid_key_nrc: 0x35,
        },
    ),
];

/// Look up a mechanism descriptor by key.
///
/// # Errors
/// Returns an error when `key` names no catalogued mechanism.
pub fn mechanism(key: &str) -> Result<Mechanism> {
    MECHANISMS
        .iter()
        .find(|&&(k, _)| k == key)
        .map(|&(_, m)| m)
        .ok_or_else(|| Error::new(format!("unknown security-access mechanism: {key:?}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requestseed_pairs_with_sendkey() {
        assert!(is_request_seed(0x01).unwrap());
        assert_eq!(send_key_for(0x01).unwrap(), 0x02);
        assert_eq!(send_key_for(0x03).unwrap(), 0x04);
        assert_eq!(request_seed_for(0x02).unwrap(), 0x01);
        assert!(is_send_key(0x02).unwrap());
        assert!(!is_request_seed(0x02).unwrap());
        assert_eq!(build_request_seed(0x01).unwrap(), vec![0x27, 0x01]);
        assert!(send_key_for(0x02).is_err()); // even is a sendKey, not a requestSeed
    }

    #[test]
    fn mechanisms_cover_classic_and_ald() {
        let classic = mechanism(CLASSIC_GM_SEED_KEY).unwrap();
        let ald = mechanism(ADIT_ALD).unwrap();
        assert_eq!(classic.family, "seed-key");
        assert_eq!(ald.family, "challenge-response");
        assert_eq!(ald.invalid_key_nrc, 0x35);
        assert!(ald.summary.contains("RSA-2048"));
        assert!(mechanism("nope").is_err());
    }

    #[test]
    fn only_the_exchange_shape_is_modelled() {
        // The module builds requestSeed and nothing that derives or sends a key. This is a
        // compile-time boundary: no seed->key function exists to call.
        assert_eq!(build_request_seed(0x01).unwrap(), vec![0x27, 0x01]);
    }
}
