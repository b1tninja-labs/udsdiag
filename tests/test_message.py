from __future__ import annotations

import pytest

from udsdiag import message


def test_a_request_encodes_sid_then_data():
    assert message.encode_request(0x22, b"\x00\x2f") == b"\x22\x00\x2f"
    request = message.Request(0x22, b"\x00\x2f")
    assert request.encode() == b"\x22\x00\x2f"
    assert request.expected_response_sid == 0x62


def test_a_positive_response_decodes_and_answers():
    # 62 002F 6C — ReadDataByIdentifier positive response (demo capture).
    response = message.decode_response(b"\x62\x00\x2f\x6c")
    assert isinstance(response, message.PositiveResponse)
    assert response.positive
    assert response.request_sid == 0x22
    assert response.data == b"\x00\x2f\x6c"
    assert response.answers(message.Request(0x22, b"\x00\x2f"))


def test_a_negative_response_decodes_with_its_nrc():
    # 7F 22 31 — ReadDataByIdentifier rejected, requestOutOfRange.
    response = message.decode_response(b"\x7f\x22\x31")
    assert isinstance(response, message.NegativeResponse)
    assert not response.positive
    assert response.request_sid == 0x22
    assert response.code == 0x31
    assert "requestOutOfRange" in response.describe()
    with pytest.raises(ValueError):
        message.decode_response(b"\x7f\x22")  # malformed: 2 bytes
    with pytest.raises(ValueError):
        message.decode_response(b"")


def test_response_pending_is_recognised():
    response = message.decode_response(b"\x7f\x1a\x78")
    assert isinstance(response, message.NegativeResponse)
    assert response.is_response_pending
