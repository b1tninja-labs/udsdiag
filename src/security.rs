//! SecurityAccess (`0x27`): the level-pairing model (ISO 14229-1:2013, 9.4).
//!
//! SecurityAccess unlocks protected services. It is a two-step exchange: the client sends
//! requestSeed (an odd sub-function), the server returns a seed, the client sends sendKey (the
//! next even sub-function) with a key derived from the seed, and the server unlocks or answers
//! NRC `0x35` invalidKey. A fixed relationship binds each requestSeed to its sendKey: `0x01` to
//! `0x02`, `0x03` to `0x04`, and so on.
//!
//! # Scope boundary
//! This module models the *exchange shape* only — no seed->key algorithm, no key material, no
//! key-derivation method. The concrete mechanism *descriptors* the assessment distinguished (the
//! classic GM seed/key and the GMG3 ADIT ALD) are GM-specific and live in the `gm` vendor layer
//! (`gm::mechanisms`); the [`Mechanism`] type lives here so both share it. No layer carries key
//! material.

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

/// A description of how a server verifies a SecurityAccess key. Reference only; carries no key
/// material or algorithm.
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
    fn only_the_exchange_shape_is_modelled() {
        // The module builds requestSeed and nothing that derives or sends a key. This is a
        // compile-time boundary: no seed->key function exists to call.
        assert_eq!(build_request_seed(0x01).unwrap(), vec![0x27, 0x01]);
    }
}
