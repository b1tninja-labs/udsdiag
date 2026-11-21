from __future__ import annotations

from udsdiag import nrc


def test_table_a1_values_are_named():
    by_value = {code.value: code for code in nrc.all_codes()}
    assert by_value[0x31].name == "requestOutOfRange"
    assert by_value[0x35].name == "invalidKey"
    assert by_value[0x22].name == "conditionsNotCorrect"
    assert by_value[0x78].mnemonic == "RCRRP"
    assert nrc.get(0x11).name == "serviceNotSupported"


def test_describe_falls_back_to_the_range():
    assert nrc.describe(0x31) == "requestOutOfRange"
    assert nrc.describe(0x05) == "ISOSAEReserved"
    assert nrc.describe(0x40) == "reservedByExtendedDataLinkSecurityDocument"
    assert nrc.describe(0x90).startswith("conditionsNotCorrect-specific")
    assert nrc.get(0x05) is None


def test_response_pending_is_0x78():
    assert nrc.is_response_pending(0x78)
    assert not nrc.is_response_pending(0x31)
