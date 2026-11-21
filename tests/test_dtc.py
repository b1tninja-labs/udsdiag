from __future__ import annotations

import pytest

from udsdiag import dtc


def test_encode_decode_round_trips_a_code():
    assert dtc.encode("P0A7F") == b"\x0a\x7f"
    assert dtc.decode(b"\x0a\x7f") == "P0A7F"
    assert dtc.decode(b"\xc0\x73") == "U0073"  # U => top two bits 11
    for code in ("P0A7F", "U0073", "C0075", "B1234"):
        assert dtc.decode(dtc.encode(code)) == code
    with pytest.raises(ValueError):
        dtc.encode("Z0000")
    with pytest.raises(ValueError):
        dtc.decode(b"\x00")


def test_category_comes_from_the_letter():
    assert dtc.category_of("P0A7F") == "powertrain"
    assert dtc.category_of("C0075") == "chassis"
    assert dtc.category_of("B1234") == "body"
    assert dtc.category_of("U0073") == "network"
    assert dtc.get("P0A7F").category == "powertrain"


def test_the_reference_subset_and_counts():
    assert dtc.APP_DTC_TOTAL == 841
    assert sum(dtc.APP_DTC_COUNTS.values()) == 841
    assert dtc.get("U0184").in_17obdg01 is False
    assert dtc.get("P0A0D").in_17obdg01 is True
    assert dtc.get("ZZZZZ") is None
    assert len(dtc.catalog()) >= 20
