"""The negative-response-code (NRC) catalog.

Values and mnemonics are from ISO 14229-1:2013, Annex A (normative), Table A.1. A negative
response message is ``0x7F <requestSID> <NRC>`` (ISO 14229-1:2013, 7.5.6). ``0x78``
(requestCorrectlyReceived-ResponsePending) is the one a client must keep waiting on rather
than treat as a final failure; :func:`is_response_pending` recognises it.

Only the codes this catalog needs are enumerated by name; the reserved and data-link ranges
are summarised. :func:`describe` always returns a label, falling back to the range an unknown
value sits in.
"""

from __future__ import annotations

from dataclasses import dataclass

REQUEST_CORRECTLY_RECEIVED_RESPONSE_PENDING = 0x78


@dataclass(frozen=True, slots=True)
class NegativeResponseCode:
    """One negative response code."""

    value: int
    name: str
    mnemonic: str

    def __post_init__(self) -> None:
        if not 0x00 <= self.value <= 0xFF:
            raise ValueError(f"nrc out of range: {self.value:#x}")


# ISO 14229-1:2013 Table A.1 — the named, non-reserved codes used across services.
_CODES: tuple[NegativeResponseCode, ...] = (
    NegativeResponseCode(0x10, "generalReject", "GR"),
    NegativeResponseCode(0x11, "serviceNotSupported", "SNS"),
    NegativeResponseCode(0x12, "sub-functionNotSupported", "SFNS"),
    NegativeResponseCode(0x13, "incorrectMessageLengthOrInvalidFormat", "IMLOIF"),
    NegativeResponseCode(0x14, "responseTooLong", "RTL"),
    NegativeResponseCode(0x21, "busyRepeatRequest", "BRR"),
    NegativeResponseCode(0x22, "conditionsNotCorrect", "CNC"),
    NegativeResponseCode(0x24, "requestSequenceError", "RSE"),
    NegativeResponseCode(0x25, "noResponseFromSubnetComponent", "NRFSC"),
    NegativeResponseCode(0x26, "failurePreventsExecutionOfRequestedAction", "FPEORA"),
    NegativeResponseCode(0x31, "requestOutOfRange", "ROOR"),
    NegativeResponseCode(0x33, "securityAccessDenied", "SAD"),
    NegativeResponseCode(0x35, "invalidKey", "IK"),
    NegativeResponseCode(0x36, "exceedNumberOfAttempts", "ENOA"),
    NegativeResponseCode(0x37, "requiredTimeDelayNotExpired", "RTDNE"),
    NegativeResponseCode(0x70, "uploadDownloadNotAccepted", "UDNA"),
    NegativeResponseCode(0x71, "transferDataSuspended", "TDS"),
    NegativeResponseCode(0x72, "generalProgrammingFailure", "GPF"),
    NegativeResponseCode(0x73, "wrongBlockSequenceCounter", "WBSC"),
    NegativeResponseCode(0x78, "requestCorrectlyReceived-ResponsePending", "RCRRP"),
    NegativeResponseCode(0x7E, "sub-functionNotSupportedInActiveSession", "SFNSIAS"),
    NegativeResponseCode(0x7F, "serviceNotSupportedInActiveSession", "SNSIAS"),
)

_BY_VALUE: dict[int, NegativeResponseCode] = {code.value: code for code in _CODES}


def all_codes() -> tuple[NegativeResponseCode, ...]:
    """Every named code in the catalog, in value order."""
    return _CODES


def get(value: int) -> NegativeResponseCode | None:
    """The named code for a value, or ``None`` when the value is reserved or unnamed."""
    return _BY_VALUE.get(value)


def describe(value: int) -> str:
    """A label for any NRC value, naming the reserved range when the value is unnamed."""
    if not 0x00 <= value <= 0xFF:
        raise ValueError(f"nrc out of range: {value:#x}")
    code = _BY_VALUE.get(value)
    if code is not None:
        return code.name
    if value == 0x00:
        return "positiveResponse (reserved, not sent in a negative response)"
    if 0x38 <= value <= 0x4F:
        return "reservedByExtendedDataLinkSecurityDocument"
    if value >= 0x80:
        return "conditionsNotCorrect-specific (manufacturer, 0x80-0xFF)"
    return "ISOSAEReserved"


def is_response_pending(value: int) -> bool:
    """Whether the NRC is 0x78, which a client treats as "keep waiting", not a failure."""
    return value == REQUEST_CORRECTLY_RECEIVED_RESPONSE_PENDING
