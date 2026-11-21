from __future__ import annotations

import pytest

from udsdiag import iocontrol


def test_build_request_frames_the_signal():
    # shortTermAdjustment of DID 0x4531 with a control state byte
    assert iocontrol.build_request(0x4531, iocontrol.SHORT_TERM_ADJUSTMENT, b"\x01") == b"\x2f\x45\x31\x03\x01"
    assert iocontrol.build_request(0x4531, iocontrol.RETURN_CONTROL_TO_ECU) == b"\x2f\x45\x31\x00"
    assert iocontrol.parameter_name(0x03) == "shortTermAdjustment"
    with pytest.raises(ValueError):
        iocontrol.build_request(0x4531, 0x09)  # not a parameter
    with pytest.raises(ValueError):
        iocontrol.build_request(0x12345, 0x03)  # did out of range


def test_signal_groups_include_ipsim_injection():
    groups = iocontrol.signal_groups()
    assert "IPSIM_PRESSED_KEY" in groups["hmi-input-injection"]
    assert iocontrol.group_of("IPSIM_VOLUME_SIMULATION") == "hmi-input-injection"
    assert iocontrol.group_of("FAN") == "thermal-power"
    assert iocontrol.group_of("NO_SUCH_SIGNAL") is None


def test_all_signals_are_unique():
    signals = iocontrol.all_signals()
    assert len(signals) == len(set(signals))
    assert len(signals) >= 50
