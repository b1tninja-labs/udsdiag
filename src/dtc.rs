//! The Diagnostic Trouble Code (DTC) model and reference catalog.
//!
//! A DTC is read with ReadDTCInformation (`0x19`) and cleared with ClearDiagnosticInformation
//! (`0x14`) (ISO 14229-1:2013). Its on-wire form is two bytes; its display form is a letter and
//! four hex digits (e.g. `P0A7F`). The letter comes from the top two bits of the first byte:
//! `P` powertrain, `C` chassis, `B` body, `U` network. That two-byte packing is the conventional
//! SAE J2012 / ISO 15031-6 format; neither document is in `primarysource` (see `docs/iso-map.md`),
//! so [`encode`] / [`decode`] implement the convention and are marked unverified against the
//! normative source.
//!
//! The DTC status byte (`statusOfDTC` and `DTCStatusAvailabilityMask`) is decoded by
//! [`decode_status`] against the ISO 14229-1:2013 Annex D.2 layout (see [`STATUS_BITS`]).
//!
//! The reference catalog is a representative subset of the cross-reference between a third-party
//! Volt app's DTC-name table (841 codes in its `classes.dex`) and GM's own `17OBDG01 Hybrid
//! Diagnostics` monitor-certification document for the Volt (OBD Group 1A); see `docs/sources.md`.
//! The per-prefix totals are recorded in [`APP_DTC_COUNTS`].

use crate::error::{Error, Result};

/// ReadDTCInformation service id.
pub const READ_DTC_INFORMATION: u8 = 0x19;
/// ClearDiagnosticInformation service id.
pub const CLEAR_DIAGNOSTIC_INFORMATION: u8 = 0x14;

/// App DTC-name table totals, per prefix, from the Voltage 2.3.0 `classes.dex` extraction.
pub const APP_DTC_COUNTS: &[(char, u32)] = &[('P', 309), ('B', 139), ('C', 63), ('U', 330)];
/// The total DTC-name count across prefixes.
pub const APP_DTC_TOTAL: u32 = 841;

const fn category_for_letter(letter: char) -> Option<&'static str> {
    match letter {
        'P' => Some("powertrain"),
        'C' => Some("chassis"),
        'B' => Some("body"),
        'U' => Some("network"),
        _ => None,
    }
}

const fn letter_to_bits(letter: char) -> Option<u8> {
    match letter {
        'P' => Some(0b00),
        'C' => Some(0b01),
        'B' => Some(0b10),
        'U' => Some(0b11),
        _ => None,
    }
}

const fn bits_to_letter(bits: u8) -> char {
    match bits & 0b11 {
        0b00 => 'P',
        0b01 => 'C',
        0b10 => 'B',
        _ => 'U',
    }
}

fn hex_nibble(c: char) -> Option<u8> {
    c.to_digit(16).and_then(|d| u8::try_from(d).ok())
}

/// Validate a display code (`[PCBU]` then four hex digits) and return `(letter, nibbles)`.
fn parse_display(code: &str) -> Option<(char, [u8; 4])> {
    let upper = code.to_ascii_uppercase();
    let chars: Vec<char> = upper.chars().collect();
    if chars.len() != 5 {
        return None;
    }
    let letter = chars[0];
    category_for_letter(letter)?;
    let mut nibbles = [0u8; 4];
    for (i, &c) in chars[1..].iter().enumerate() {
        nibbles[i] = hex_nibble(c)?;
    }
    Some((letter, nibbles))
}

/// `powertrain` / `chassis` / `body` / `network`, from a display code's leading letter.
///
/// # Errors
/// Returns an error when the leading letter is not one of `P`, `C`, `B`, `U`.
pub fn category_of(code: &str) -> Result<&'static str> {
    code.chars()
        .next()
        .and_then(|c| category_for_letter(c.to_ascii_uppercase()))
        .ok_or_else(|| Error::new(format!("not a DTC display code: {code:?}")))
}

/// Pack a display code (e.g. `P0A7F`) into its two on-wire bytes.
///
/// Conventional SAE J2012 / ISO 15031-6 packing (unverified against the normative source; see
/// the module docs).
///
/// # Errors
/// Returns an error when `code` is not a letter followed by four hex digits.
pub fn encode(code: &str) -> Result<[u8; 2]> {
    let (letter, [d1, d2, d3, d4]) = parse_display(code)
        .ok_or_else(|| Error::new(format!("not a DTC display code: {code:?}")))?;
    let bits = letter_to_bits(letter).unwrap_or(0);
    let byte0 = (bits << 6) | (d1 << 4) | d2;
    let byte1 = (d3 << 4) | d4;
    Ok([byte0, byte1])
}

/// Unpack two on-wire bytes into a display code (the inverse of [`encode`]).
///
/// # Errors
/// Returns an error when `raw` is not exactly two bytes.
pub fn decode(raw: &[u8]) -> Result<String> {
    if raw.len() != 2 {
        return Err(Error::new(format!("a DTC is two bytes, got {}", raw.len())));
    }
    let byte0 = raw[0];
    let byte1 = raw[1];
    let letter = bits_to_letter((byte0 >> 6) & 0b11);
    let d1 = (byte0 >> 4) & 0b11;
    let d2 = byte0 & 0x0F;
    Ok(format!("{letter}{d1:X}{d2:X}{byte1:02X}"))
}

// --- DTC status byte (statusOfDTC / DTCStatusAvailabilityMask) --------------------------------
//
// The statusOfDTC byte reported by ReadDTCInformation (0x19) and the DTCStatusAvailabilityMask
// the server advertises share one 8-bit layout, bit 0 first. The bit names, mnemonics and
// availability (M/U) are ISO 14229-1:2013, Annex D.2 (Tables D.3-D.11, DTC status bit
// definitions), confirmed against the 0x19 message-flow examples in 11.3.5 (Tables 287-289),
// both read through the primarysource iso_* tools; see docs/iso-map.md.

/// One bit of the 8-bit DTC status byte, with its meaning when set to 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusBit {
    /// The bit position, 0 (least significant) through 7.
    pub bit: u8,
    /// The ISO bit name.
    pub name: &'static str,
    /// The ISO mnemonic.
    pub mnemonic: &'static str,
    /// Availability: `"M"` mandatory or `"U"` user-optional (ISO 14229-1:2013 Annex D.2).
    pub availability: &'static str,
    /// What the bit means when set to 1.
    pub meaning: &'static str,
}

const fn status_bit(
    bit: u8,
    name: &'static str,
    mnemonic: &'static str,
    availability: &'static str,
    meaning: &'static str,
) -> StatusBit {
    StatusBit {
        bit,
        name,
        mnemonic,
        availability,
        meaning,
    }
}

/// The eight DTC status bits in bit order (ISO 14229-1:2013 Annex D.2, Tables D.3-D.11).
pub const STATUS_BITS: &[StatusBit] = &[
    status_bit(
        0,
        "testFailed",
        "TF",
        "M",
        "the most recently performed test failed",
    ),
    status_bit(
        1,
        "testFailedThisOperationCycle",
        "TFTOC",
        "U",
        "a test reported a testFailed result at any time during the current operation cycle",
    ),
    status_bit(
        2,
        "pendingDTC",
        "PDTC",
        "U",
        "the DTC failed on the current or a previous operation cycle but is not yet confirmed",
    ),
    status_bit(
        3,
        "confirmedDTC",
        "CDTC",
        "M",
        "a malfunction was detected enough times to warrant the DTC being stored",
    ),
    status_bit(
        4,
        "testNotCompletedSinceLastClear",
        "TNCSLC",
        "U",
        "a test has not completed since the last ClearDiagnosticInformation",
    ),
    status_bit(
        5,
        "testFailedSinceLastClear",
        "TFSLC",
        "U",
        "a test failed at least once since the last ClearDiagnosticInformation",
    ),
    status_bit(
        6,
        "testNotCompletedThisOperationCycle",
        "TNCTOC",
        "U",
        "a test has not completed this operation cycle",
    ),
    status_bit(
        7,
        "warningIndicatorRequested",
        "WIR",
        "U",
        "the server is requesting the warning indicator to be active",
    ),
];

/// The eight DTC status bits in bit order (bit 0 first).
pub const fn status_bits() -> &'static [StatusBit] {
    STATUS_BITS
}

/// The status bits set in an 8-bit DTC status byte, bit 0 first.
///
/// Applies to both a `statusOfDTC` byte and a `DTCStatusAvailabilityMask`: they share the Annex
/// D.2 layout, so the mask's result names the status bits the server supports. The byte is a
/// `u8`, so unlike the Python original there is no out-of-range case to reject.
pub fn decode_status(value: u8) -> Vec<StatusBit> {
    STATUS_BITS
        .iter()
        .copied()
        .filter(|b| value & (1 << b.bit) != 0)
        .collect()
}

/// One diagnostic trouble code with its app-side name and GM-document coverage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dtc {
    /// The display code, e.g. `P0A7F`.
    pub code: &'static str,
    /// The app-side DTC name.
    pub name: &'static str,
    /// Whether GM's 17OBDG01 Hybrid Diagnostics carries the code in its powertrain cert set.
    pub in_17obdg01: bool,
    /// A short note on the code, or the empty string.
    pub note: &'static str,
}

impl Dtc {
    /// `powertrain` / `chassis` / `body` / `network`.
    pub fn category(&self) -> &'static str {
        self.code
            .chars()
            .next()
            .and_then(category_for_letter)
            .unwrap_or("unknown")
    }
}

const fn dtc(code: &'static str, name: &'static str, in_17obdg01: bool, note: &'static str) -> Dtc {
    Dtc {
        code,
        name,
        in_17obdg01,
        note,
    }
}

// Representative cross-reference subset (docs/sources.md). in_17obdg01 marks codes GM's
// 17OBDG01 Hybrid Diagnostics (Volt, OBD Group 1A) carries in its powertrain cert set.
const CATALOG: &[Dtc] = &[
    // High-voltage battery & HV system
    dtc(
        "P0A0D",
        "High Voltage System Interlock Circuit Voltage",
        true,
        "HVIL safety interlock",
    ),
    dtc("P0A7E", "Hybrid/EV Battery Pack High Temperature", true, ""),
    dtc(
        "P0A7F",
        "Hybrid/EV Battery Pack",
        true,
        "GM scopes it to pack capability / end-of-life",
    ),
    dtc(
        "P0AE3",
        "Hybrid/EV Battery Precharge Contactor Stuck Open",
        true,
        "",
    ),
    dtc(
        "P0AF8",
        "Hybrid System Voltage",
        true,
        "pack vs TPIM/charger bus correlation > 16 V",
    ),
    dtc(
        "P0AFB",
        "Hybrid/EV Battery System Voltage",
        true,
        "pack or cell over-voltage",
    ),
    dtc(
        "P0B13",
        "Hybrid/EV Battery Current Sensor",
        true,
        "fine vs coarse current > 10 A",
    ),
    dtc(
        "P0C76",
        "Hybrid/EV Battery System High Voltage Present",
        true,
        "HV discharge time too long",
    ),
    dtc(
        "P0D5E",
        "Battery Charger System High Voltage Present",
        true,
        "",
    ),
    // Drive motor & inverter (TPIM)
    dtc("P0A1C", "Drive Motor Control Module Performance", true, ""),
    dtc(
        "P0A79",
        "Drive Motor Inverter Performance",
        true,
        "IGBT desaturation",
    ),
    dtc("P0C04", "Drive Motor High Current", true, ""),
    dtc(
        "P0C5C",
        "Drive Motor 1 Position Sensor",
        true,
        "resolver circuit low",
    ),
    // On-board charger & charge cord
    dtc(
        "P0CF6",
        "Control Pilot Circuit",
        true,
        "J1772 pilot out-of-range high",
    ),
    dtc(
        "P0D26",
        "Battery Charger System Precharge Time Too Long",
        true,
        "",
    ),
    dtc(
        "P0D59",
        "Proximity Detection Circuit Low/High Voltage",
        true,
        "charge-cord latch",
    ),
    // Engine / range-extender (ECM)
    dtc("P1101", "Intake Air Flow System Performance", true, ""),
    dtc("P2097", "Post Catalyst Fuel Trim System", true, ""),
    dtc("P2138", "Accelerator Pedal Position (APP) Sensor", true, ""),
    // Communication — in 17OBDG01 powertrain set
    dtc("U0073", "Control Module Communication Bus A Off", true, ""),
    dtc(
        "U0100",
        "Lost Communication With Engine Control Module",
        true,
        "",
    ),
    dtc(
        "U0111",
        "Lost Communication with Battery Energy Control Module",
        true,
        "",
    ),
    dtc(
        "U0140",
        "Lost Communication With Body Control Module",
        true,
        "",
    ),
    // Communication — NOT in 17OBDG01 (module the app decodes)
    dtc(
        "U0121",
        "Lost Communication With Electronic Brake Control Module",
        false,
        "module: EBCM",
    ),
    dtc(
        "U0146",
        "Lost Communication with Serial Data Gateway Module",
        false,
        "",
    ),
    dtc(
        "U0155",
        "Lost Communication With Instrument Cluster",
        false,
        "module: IPC",
    ),
    dtc(
        "U0184",
        "Lost Communication With Radio",
        false,
        "module: Radio (head unit)",
    ),
    dtc(
        "U0256",
        "Lost Communication (faceplate)",
        false,
        "named in OBD-CONFIG.md",
    ),
    // Chassis / brake (BSC section of 17OBDG01)
    dtc("C0075", "Brake Pedal Position Sensor Performance", true, ""),
    dtc("C0591", "Brake Booster Motor", true, ""),
    dtc(
        "C10EA",
        "Powertrain Control Module Indicated Requested Torque",
        true,
        "",
    ),
];

/// The representative DTC reference subset.
pub const fn catalog() -> &'static [Dtc] {
    CATALOG
}

/// The reference entry for a display code, or `None` if not in the subset.
pub fn get(code: &str) -> Option<Dtc> {
    let upper = code.to_ascii_uppercase();
    CATALOG.iter().copied().find(|entry| entry.code == upper)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_round_trips_a_code() {
        assert_eq!(encode("P0A7F").unwrap(), [0x0A, 0x7F]);
        assert_eq!(decode(&[0x0A, 0x7F]).unwrap(), "P0A7F");
        assert_eq!(decode(&[0xC0, 0x73]).unwrap(), "U0073"); // U => top two bits 11
        for code in ["P0A7F", "U0073", "C0075", "B1234"] {
            assert_eq!(decode(&encode(code).unwrap()).unwrap(), code);
        }
        assert!(encode("Z0000").is_err());
        assert!(decode(&[0x00]).is_err());
    }

    #[test]
    fn category_comes_from_the_letter() {
        assert_eq!(category_of("P0A7F").unwrap(), "powertrain");
        assert_eq!(category_of("C0075").unwrap(), "chassis");
        assert_eq!(category_of("B1234").unwrap(), "body");
        assert_eq!(category_of("U0073").unwrap(), "network");
        assert_eq!(get("P0A7F").unwrap().category(), "powertrain");
    }

    #[test]
    fn status_byte_decodes_against_iso_examples() {
        // ISO 14229-1:2013, 11.3.5 Example #1: statusOfDTC 0x24 -> pendingDTC + testFailedSinceLastClear.
        let names_24: std::collections::BTreeSet<&str> =
            decode_status(0x24).iter().map(|b| b.name).collect();
        assert_eq!(
            names_24,
            std::collections::BTreeSet::from(["pendingDTC", "testFailedSinceLastClear"])
        );
        // statusOfDTC 0x2F (Table 289): testFailed, testFailedThisOperationCycle, pendingDTC,
        // confirmedDTC, testFailedSinceLastClear.
        let bits_2f: std::collections::BTreeSet<u8> =
            decode_status(0x2F).iter().map(|b| b.bit).collect();
        assert_eq!(bits_2f, std::collections::BTreeSet::from([0, 1, 2, 3, 5]));
        assert_eq!(decode_status(0x00).len(), 0);
        assert_eq!(decode_status(0xFF).len(), 8);
        let order: Vec<u8> = status_bits().iter().map(|b| b.bit).collect();
        assert_eq!(order, (0..8).collect::<Vec<u8>>());
        assert_eq!(status_bits()[3].mnemonic, "CDTC");
    }

    #[test]
    fn the_reference_subset_and_counts() {
        assert_eq!(APP_DTC_TOTAL, 841);
        assert_eq!(APP_DTC_COUNTS.iter().map(|&(_, n)| n).sum::<u32>(), 841);
        assert!(!get("U0184").unwrap().in_17obdg01);
        assert!(get("P0A0D").unwrap().in_17obdg01);
        assert!(get("ZZZZZ").is_none());
        assert!(catalog().len() >= 20);
    }
}
