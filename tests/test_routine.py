from __future__ import annotations

import pytest

from udsdiag import routine


def test_build_request_frames_the_routine():
    # startRoutine, routine 0x3202 (StressTest)
    assert routine.build_request(routine.START_ROUTINE, 0x3202) == b"\x31\x01\x32\x02"
    assert routine.build_request(0x03, 0x5247, b"\x01") == b"\x31\x03\x52\x47\x01"
    with pytest.raises(ValueError):
        routine.build_request(0x09, 0x3202)  # not a sub-function
    with pytest.raises(ValueError):
        routine.build_request(0x01, 0x12345)  # routine id out of range


def test_the_catalog_names_the_sharp_edges():
    stress = routine.get(0x3202)
    assert stress is not None
    assert stress.name == "StressTest"
    assert stress.risk == "H"
    assert routine.get(0xD001).name == "SetSystemToDeliveryState"
    assert routine.get(0x0000) is None
    assert {r.risk for r in routine.catalog()} <= {"H", "M", "L"}


def test_subfunction_names():
    assert routine.subfunction_name(0x01) == "startRoutine"
    assert routine.subfunction_name(0x02) == "stopRoutine"
    assert routine.subfunction_name(0x03) == "requestRoutineResults"
    with pytest.raises(ValueError):
        routine.subfunction_name(0x04)
