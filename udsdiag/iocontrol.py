"""InputOutputControlByIdentifier (``0x2F``, GM alias DeviceControl ``$AE``).

The service drives a server signal named by a 2-byte dataIdentifier. The first byte of the
controlOptionRecord is the inputOutputControlParameter: ``0x00`` returnControlToECU, ``0x01``
resetToDefault, ``0x02`` freezeCurrentState, ``0x03`` shortTermAdjustment (ISO 14229-1:2013,
E.1). :func:`build_request` assembles ``0x2F <DID MSB> <DID LSB> <parameter> [controlState]``.

:data:`SIGNAL_GROUPS` is the ``DIA_EN_IOCTRL_SIGTYPE_*`` signal enumeration extracted in full
from the GM GMG3 head-unit firmware (``docs/sources.md``), grouped by what each touches. The
standout is the ``IPSIM_*`` group — HMI input injection (simulated keypress / knob / volume),
driving the user interface as if a person pressed it — alongside speaker, microphone-power,
USB-power, camera and tuner actuators. Every signal is gated by SecurityAccess (``0x27`` ->
ALD); see :mod:`udsdiag.security`.
"""

from __future__ import annotations

INPUT_OUTPUT_CONTROL_BY_IDENTIFIER = 0x2F
GM_DEVICE_CONTROL = 0xAE

RETURN_CONTROL_TO_ECU = 0x00
RESET_TO_DEFAULT = 0x01
FREEZE_CURRENT_STATE = 0x02
SHORT_TERM_ADJUSTMENT = 0x03

_PARAMETERS: dict[int, str] = {
    RETURN_CONTROL_TO_ECU: "returnControlToECU",
    RESET_TO_DEFAULT: "resetToDefault",
    FREEZE_CURRENT_STATE: "freezeCurrentState",
    SHORT_TERM_ADJUSTMENT: "shortTermAdjustment",
}


def parameter_name(parameter: int) -> str:
    """The name of an inputOutputControlParameter byte."""
    try:
        return _PARAMETERS[parameter]
    except KeyError:
        raise ValueError(f"not an IOControl parameter: {parameter:#04x}") from None


def build_request(did: int, parameter: int, control_state: bytes = b"") -> bytes:
    """Build an InputOutputControlByIdentifier request PDU."""
    if not 0x0000 <= did <= 0xFFFF:
        raise ValueError(f"did out of range: {did:#x}")
    if parameter not in _PARAMETERS:
        raise ValueError(f"not an IOControl parameter: {parameter:#04x}")
    return bytes([INPUT_OUTPUT_CONTROL_BY_IDENTIFIER, did >> 8, did & 0xFF, parameter]) + control_state


# DIA_EN_IOCTRL_SIGTYPE_* signal enumeration, grouped (GMG3 firmware; docs/sources.md).
SIGNAL_GROUPS: dict[str, tuple[str, ...]] = {
    "hmi-input-injection": (
        "IPSIM_PRESSED_KEY",
        "IPSIM_KEY_CODE_SIMULATION",
        "IPSIM_CONTROL_KNOB_SIMULATION",
        "IPSIM_VOLUME_SIMULATION",
        "IPSIM_LOCK_INFO_CONTROL",
        "PRESSED_KEY",
    ),
    "speaker-audio": (
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
    ),
    "microphone-power": (
        "MICROPHONE_POWER_SUPPLY",
        "MICROPHONE_POWER_SUPPLY_2",
    ),
    "camera": (
        "RVC_CONTROLLINE",
        "ACTIVE_VIDEO_SOURCE",
        "VIDEO_SOURCE",
    ),
    "display": (
        "DISPLAY_TEST",
        "DISPLAY_TEST_PATTERN",
        "DISPLAY_BRIGHTNESS",
        "KEY_ILLUMINATION_PWM",
        "KEYILLUMINATION_BRIGHTNESS",
    ),
    "thermal-power": (
        "FAN",
        "USB_POWER_SUPPLY_1",
        "USB_POWER_SUPPLY_2",
        "USB_POWER_SUPPLY_3",
    ),
    "bluetooth": (
        "BLUETOOTH_MODULE_RESET",
        "BLUETOOTH_SEARCH_DEVICES",
        "BLUETOOTH_PAGING",
        "BLUETOOTH_MODE",
    ),
    "optical-drive": (
        "CDROM_EJECT",
        "CD_RESET_LINE",
        "CDDRIVE_TEST_QUICK",
        "CD_MODE_CONTROL",
        "PLAYED_TRACK",
    ),
    "tuner-rf": (
        "TUNERAMFM_ANT1_POWER_SUPPLY",
        "TUNERAMFM_ANT2_POWER_SUPPLY",
        "TUNERDAB_ANT1_POWER_SUPPLY",
        "TUNERDAB_ANT1_ENSEMBLE_FREQUENCY",
        "TUNERDAB_ANT1_SERVICE_ID",
        "TUNERXM_1_C",
        "TUNER_SEL_ACTIVE_TUNER_AMFM",
        "TUNER_SET_BAND_AMFM",
        "TUNER_SET_FREQUENCY_AMFM",
    ),
    "system-misc": (
        "SYSTEM_HMI_ON",
        "SYSTEM_REMOTE_CONTROL",
        "REMOTE_CONTROL_STATE",
        "GNSS_SYSTEM",
        "HMI_LANGUAGE",
        "NEONODE_RESETLINE",
        "DATASET_BLOCK_SIZE",
        "UNKNOWN",
    ),
}


def signal_groups() -> dict[str, tuple[str, ...]]:
    """The signal enumeration, grouped by what each signal touches."""
    return SIGNAL_GROUPS


def all_signals() -> tuple[str, ...]:
    """Every signal name across the groups, in group then declaration order."""
    return tuple(signal for group in SIGNAL_GROUPS.values() for signal in group)


def group_of(signal: str) -> str | None:
    """The group a signal name belongs to, or ``None`` when it is not catalogued."""
    for group, signals in SIGNAL_GROUPS.items():
        if signal in signals:
            return group
    return None
