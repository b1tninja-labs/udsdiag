from __future__ import annotations

import pytest

from udsdiag import security


def test_requestseed_pairs_with_sendkey():
    assert security.is_request_seed(0x01)
    assert security.send_key_for(0x01) == 0x02
    assert security.send_key_for(0x03) == 0x04
    assert security.request_seed_for(0x02) == 0x01
    assert security.is_send_key(0x02)
    assert not security.is_request_seed(0x02)
    assert security.build_request_seed(0x01) == b"\x27\x01"
    with pytest.raises(ValueError):
        security.send_key_for(0x02)  # even is a sendKey, not a requestSeed


def test_mechanisms_cover_classic_and_ald():
    classic = security.mechanism("classic-gm-seed-key")
    ald = security.mechanism("adit-ald")
    assert classic.family == "seed-key"
    assert ald.family == "challenge-response"
    assert ald.invalid_key_nrc == 0x35
    assert "RSA-2048" in ald.summary
    with pytest.raises(ValueError):
        security.mechanism("nope")


def test_no_key_derivation_is_exposed():
    # The module models the exchange shape only: it builds requestSeed but has no
    # callable that derives or sends a key.
    assert hasattr(security, "build_request_seed")
    names = dir(security)
    for forbidden in ("derive_key", "compute_key", "send_key", "build_send_key", "seed_to_key"):
        assert forbidden not in names
