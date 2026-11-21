from __future__ import annotations

import pytest

from udsdiag import gmlan


def test_response_and_filter_offsets():
    assert gmlan.response_id(0x241) == 0x641
    assert gmlan.filter_id(0x241) == 0x541
    assert gmlan.response_id(0x24C) == 0x64C
    with pytest.raises(ValueError):
        gmlan.response_id(0x800)  # not 11-bit
    with pytest.raises(ValueError):
        gmlan.response_id(0x7FF)  # +0x400 would overflow 11 bits


def test_obd_physical_response_offset():
    assert gmlan.obd_response_id(0x7E0) == 0x7E8
    assert gmlan.obd_response_id(0x7E7) == 0x7EF
    with pytest.raises(ValueError):
        gmlan.obd_response_id(0x7DF)  # functional broadcast, not a physical request
