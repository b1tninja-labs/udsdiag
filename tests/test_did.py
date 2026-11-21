from __future__ import annotations

from udsdiag import did


def test_the_catalog_has_115_read_dids():
    dids = did.read_dids()
    assert len(dids) == 115
    values = [d.value for d in dids]
    assert len(values) == len(set(values))  # no duplicates
    assert 0x002F in values
    assert 0x90B1 in values
    # the write targets are separate
    assert {d.value for d in did.write_dids()} == {0xD21B, 0xD28E}


def test_classify_names_the_identification_range():
    assert did.classify(0x90B1) == "identification"
    assert did.classify(0x41AD) == "gm-enhanced-dense"
    assert did.classify(0x4202) == "gm-enhanced-dense"
    assert did.classify(0x8334) == "gm-enhanced"
    counts = did.count_by_category()
    assert counts["gm-enhanced-dense"] == 92  # 22 in 41xx + 70 in 42xx
    assert counts["identification"] == 3


def test_parse_read_response_splits_did_and_value():
    assert did.parse_read_response(b"\x62\x00\x2f\x6c") == (0x002F, b"\x6c")
    assert did.parse_read_response(b"\x62\x41\xad\x00\x00") == (0x41AD, b"\x00\x00")
