from __future__ import annotations

import pytest

from udsdiag import services


def test_catalog_has_the_core_services():
    sids = {service.sid for service in services.all_services()}
    for sid in (0x10, 0x11, 0x14, 0x19, 0x22, 0x27, 0x2E, 0x2F, 0x31, 0x3E):
        assert sid in sids
    assert services.get(0x31).name == "RoutineControl"


def test_positive_response_sid_adds_0x40():
    assert services.positive_response_sid(0x22) == 0x62
    assert services.positive_response_sid(0x31) == 0x71
    assert services.is_response_sid(0x62)
    assert services.request_sid_for_response(0x62) == 0x22
    with pytest.raises(ValueError):
        services.positive_response_sid(0xFF)


def test_get_returns_none_for_unknown_sid():
    assert services.get(0x00) is None
    assert services.name_of(0x00).startswith("UnknownService")


def test_gmlan_services_are_flagged():
    device_control = services.get(0xAE)
    assert device_control is not None
    assert device_control.standard == "GMLAN"
    assert services.get(0x22).standard == "ISO 14229-1:2013"
