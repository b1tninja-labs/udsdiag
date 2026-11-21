//! InputOutputControlByIdentifier (`0x2F`, GM alias DeviceControl `$AE`).
//!
//! The service drives a server signal named by a 2-byte dataIdentifier. The first byte of the
//! controlOptionRecord is the inputOutputControlParameter: `0x00` returnControlToECU, `0x01`
//! resetToDefault, `0x02` freezeCurrentState, `0x03` shortTermAdjustment (ISO 14229-1:2013, E.1).
//! [`build_request`] assembles `0x2F <DID MSB> <DID LSB> <parameter> [controlState]`.
//!
//! [`SIGNAL_GROUPS`] is the `DIA_EN_IOCTRL_SIGTYPE_*` signal enumeration extracted in full from
//! the GM GMG3 head-unit firmware (`docs/sources.md`), grouped by what each touches. The standout
//! is the `IPSIM_*` group — HMI input injection (simulated keypress / knob / volume), driving the
//! user interface as if a person pressed it — alongside speaker, microphone-power, USB-power,
//! camera and tuner actuators. Every signal is gated by SecurityAccess (`0x27` -> ALD); see
//! [`crate::security`].

use crate::error::{Error, Result};

/// InputOutputControlByIdentifier service id.
pub const INPUT_OUTPUT_CONTROL_BY_IDENTIFIER: u8 = 0x2F;
/// GM DeviceControl alias service id.
pub const GM_DEVICE_CONTROL: u8 = 0xAE;

/// returnControlToECU parameter.
pub const RETURN_CONTROL_TO_ECU: u8 = 0x00;
/// resetToDefault parameter.
pub const RESET_TO_DEFAULT: u8 = 0x01;
/// freezeCurrentState parameter.
pub const FREEZE_CURRENT_STATE: u8 = 0x02;
/// shortTermAdjustment parameter.
pub const SHORT_TERM_ADJUSTMENT: u8 = 0x03;

const PARAMETERS: &[(u8, &str)] = &[
    (RETURN_CONTROL_TO_ECU, "returnControlToECU"),
    (RESET_TO_DEFAULT, "resetToDefault"),
    (FREEZE_CURRENT_STATE, "freezeCurrentState"),
    (SHORT_TERM_ADJUSTMENT, "shortTermAdjustment"),
];

/// The name of an inputOutputControlParameter byte.
///
/// # Errors
/// Returns an error when `parameter` is not one of `0x00`-`0x03`.
pub fn parameter_name(parameter: u8) -> Result<&'static str> {
    PARAMETERS
        .iter()
        .find(|&&(value, _)| value == parameter)
        .map(|&(_, name)| name)
        .ok_or_else(|| Error::new(format!("not an IOControl parameter: {parameter:#04x}")))
}

/// Build an InputOutputControlByIdentifier request PDU.
///
/// # Errors
/// Returns an error when `parameter` is not an inputOutputControlParameter.
pub fn build_request(did: u16, parameter: u8, control_state: &[u8]) -> Result<Vec<u8>> {
    if !PARAMETERS.iter().any(|&(value, _)| value == parameter) {
        return Err(Error::new(format!(
            "not an IOControl parameter: {parameter:#04x}"
        )));
    }
    let [hi, lo] = did.to_be_bytes();
    let mut pdu = vec![INPUT_OUTPUT_CONTROL_BY_IDENTIFIER, hi, lo, parameter];
    pdu.extend_from_slice(control_state);
    Ok(pdu)
}

/// `DIA_EN_IOCTRL_SIGTYPE_*` signal enumeration, grouped (GMG3 firmware; `docs/sources.md`).
pub const SIGNAL_GROUPS: &[(&str, &[&str])] = &[
    (
        "hmi-input-injection",
        &[
            "IPSIM_PRESSED_KEY",
            "IPSIM_KEY_CODE_SIMULATION",
            "IPSIM_CONTROL_KNOB_SIMULATION",
            "IPSIM_VOLUME_SIMULATION",
            "IPSIM_LOCK_INFO_CONTROL",
            "PRESSED_KEY",
        ],
    ),
    (
        "speaker-audio",
        &[
            "ACTUATOR_FRONT_LEFT",
            "ACTUATOR_FRONT_RIGHT",
            "ACTUATOR_REAR_LEFT",
            "ACTUATOR_REAR_RIGHT",
            "AUDIO_TESTTONE",
            "AUDIO_VOLUME",
            "AUDIO_SOURCE",
            "AUDIO_BALANCE",
            "AUDIO_FADER",
            "AUDIO_LINEARMODE",
            "AUDIO_EXT_AMPL_CONTLINE",
            "AUDIO_SELECT_COMPRESSED_AUDIO_TITLE",
        ],
    ),
    (
        "microphone-power",
        &["MICROPHONE_POWER_SUPPLY", "MICROPHONE_POWER_SUPPLY_2"],
    ),
    (
        "camera",
        &["RVC_CONTROLLINE", "ACTIVE_VIDEO_SOURCE", "VIDEO_SOURCE"],
    ),
    (
        "display",
        &[
            "DISPLAY_TEST",
            "DISPLAY_TEST_PATTERN",
            "DISPLAY_BRIGHTNESS",
            "KEY_ILLUMINATION_PWM",
            "KEYILLUMINATION_BRIGHTNESS",
        ],
    ),
    (
        "thermal-power",
        &[
            "FAN",
            "USB_POWER_SUPPLY_1",
            "USB_POWER_SUPPLY_2",
            "USB_POWER_SUPPLY_3",
        ],
    ),
    (
        "bluetooth",
        &[
            "BLUETOOTH_MODULE_RESET",
            "BLUETOOTH_SEARCH_DEVICES",
            "BLUETOOTH_PAGING",
            "BLUETOOTH_MODE",
        ],
    ),
    (
        "optical-drive",
        &[
            "CDROM_EJECT",
            "CD_RESET_LINE",
            "CDDRIVE_TEST_QUICK",
            "CD_MODE_CONTROL",
            "PLAYED_TRACK",
        ],
    ),
    (
        "tuner-rf",
        &[
            "TUNERAMFM_ANT1_POWER_SUPPLY",
            "TUNERAMFM_ANT2_POWER_SUPPLY",
            "TUNERDAB_ANT1_POWER_SUPPLY",
            "TUNERDAB_ANT1_ENSEMBLE_FREQUENCY",
            "TUNERDAB_ANT1_SERVICE_ID",
            "TUNERXM_1_C",
            "TUNER_SEL_ACTIVE_TUNER_AMFM",
            "TUNER_SET_BAND_AMFM",
            "TUNER_SET_FREQUENCY_AMFM",
        ],
    ),
    (
        "system-misc",
        &[
            "SYSTEM_HMI_ON",
            "SYSTEM_REMOTE_CONTROL",
            "REMOTE_CONTROL_STATE",
            "GNSS_SYSTEM",
            "HMI_LANGUAGE",
            "NEONODE_RESETLINE",
            "DATASET_BLOCK_SIZE",
            "UNKNOWN",
        ],
    ),
];

/// The signal enumeration, grouped by what each signal touches.
pub const fn signal_groups() -> &'static [(&'static str, &'static [&'static str])] {
    SIGNAL_GROUPS
}

/// Every signal name across the groups, in group then declaration order.
pub fn all_signals() -> Vec<&'static str> {
    SIGNAL_GROUPS
        .iter()
        .flat_map(|&(_, signals)| signals.iter().copied())
        .collect()
}

/// The group a signal name belongs to, or `None` when it is not catalogued.
pub fn group_of(signal: &str) -> Option<&'static str> {
    SIGNAL_GROUPS
        .iter()
        .find(|&&(_, signals)| signals.contains(&signal))
        .map(|&(group, _)| group)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_request_frames_the_signal() {
        assert_eq!(
            build_request(0x4531, SHORT_TERM_ADJUSTMENT, &[0x01]).unwrap(),
            vec![0x2F, 0x45, 0x31, 0x03, 0x01]
        );
        assert_eq!(
            build_request(0x4531, RETURN_CONTROL_TO_ECU, &[]).unwrap(),
            vec![0x2F, 0x45, 0x31, 0x00]
        );
        assert_eq!(parameter_name(0x03).unwrap(), "shortTermAdjustment");
        assert!(build_request(0x4531, 0x09, &[]).is_err());
    }

    #[test]
    fn signal_groups_include_ipsim_injection() {
        assert_eq!(group_of("IPSIM_PRESSED_KEY"), Some("hmi-input-injection"));
        assert_eq!(
            group_of("IPSIM_VOLUME_SIMULATION"),
            Some("hmi-input-injection")
        );
        assert_eq!(group_of("FAN"), Some("thermal-power"));
        assert_eq!(group_of("NO_SUCH_SIGNAL"), None);
    }

    #[test]
    fn all_signals_are_unique() {
        let signals = all_signals();
        let unique: std::collections::BTreeSet<&str> = signals.iter().copied().collect();
        assert_eq!(signals.len(), unique.len());
        assert!(signals.len() >= 50);
    }
}
