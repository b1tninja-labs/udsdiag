"""The UDS/GMLAN request and response codec (application-layer PDUs, ISO 14229-1:2013, 7).

This is the client model: it builds a request PDU and reads a response PDU. A request is a
service SID followed by its data. A positive response is the request SID raised by ``0x40``
followed by the response data. A negative response is ``0x7F``, the echoed request SID, and
a single NRC byte (ISO 14229-1:2013, 7.5.6).

The codec works on the reassembled application PDU only. The transport that segments a PDU
larger than one CAN frame (ISO-TP / ISO 15765-2) is the concern of a bus layer this package
does not include; see ``docs/sources.md``.
"""

from __future__ import annotations

from dataclasses import dataclass, field

from udsdiag import nrc, services

NEGATIVE_RESPONSE_SID = services.NEGATIVE_RESPONSE_SID  # 0x7F


@dataclass(frozen=True, slots=True)
class Request:
    """A diagnostic request PDU: a service SID and its data bytes."""

    sid: int
    data: bytes = b""

    def __post_init__(self) -> None:
        if not 0x00 <= self.sid <= 0xFF:
            raise ValueError(f"sid out of range: {self.sid:#x}")

    def encode(self) -> bytes:
        """The on-wire request PDU."""
        return bytes([self.sid]) + self.data

    @property
    def expected_response_sid(self) -> int:
        """The positive-response SID this request expects."""
        return services.positive_response_sid(self.sid)


@dataclass(frozen=True, slots=True)
class PositiveResponse:
    """A positive response PDU: a response SID (request SID + 0x40) and its data."""

    response_sid: int
    data: bytes = b""
    positive: bool = field(default=True, init=False)

    @property
    def request_sid(self) -> int:
        """The request SID this response answers."""
        return services.request_sid_for_response(self.response_sid)

    def answers(self, request: Request) -> bool:
        """Whether this response positively answers the given request."""
        return self.response_sid == request.expected_response_sid


@dataclass(frozen=True, slots=True)
class NegativeResponse:
    """A negative response PDU: ``0x7F``, the request SID, and one NRC byte."""

    request_sid: int
    code: int
    positive: bool = field(default=False, init=False)

    def __post_init__(self) -> None:
        if not 0x00 <= self.code <= 0xFF:
            raise ValueError(f"nrc out of range: {self.code:#x}")

    @property
    def is_response_pending(self) -> bool:
        """Whether the NRC is 0x78 (keep waiting), not a final failure."""
        return nrc.is_response_pending(self.code)

    def describe(self) -> str:
        """``<service> rejected: <nrc name> (0x..)``."""
        return f"{services.name_of(self.request_sid)} rejected: {nrc.describe(self.code)} ({self.code:#04x})"


Response = PositiveResponse | NegativeResponse


def encode_request(sid: int, data: bytes = b"") -> bytes:
    """Frame a request PDU: the service SID followed by its data."""
    return Request(sid=sid, data=data).encode()


def decode_response(pdu: bytes) -> Response:
    """Decode a response PDU into a positive or negative response.

    Raises :class:`ValueError` on an empty PDU or a malformed negative response.
    """
    if not pdu:
        raise ValueError("empty response pdu")
    if pdu[0] == NEGATIVE_RESPONSE_SID:
        if len(pdu) != 3:
            raise ValueError(f"negative response must be 3 bytes, got {len(pdu)}")
        return NegativeResponse(request_sid=pdu[1], code=pdu[2])
    return PositiveResponse(response_sid=pdu[0], data=bytes(pdu[1:]))
