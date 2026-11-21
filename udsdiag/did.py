"""The Data Identifier (DID) catalog.

A DID is the 2-byte identifier ReadDataByIdentifier (``0x22``) reads and WriteDataByIdentifier
(``0x2E``) writes. This catalog is the set a third-party Chevrolet Volt app (Voltage 2.3.0)
references, recovered from its process memory in demo mode (115 ``0x22`` DIDs, 2 ``0x2E``
targets); see ``docs/sources.md``. The catalog records which identifiers exist and the range
each sits in — not the GM-proprietary field semantics of the enhanced blocks, which are not
published and would need an owner-authorised vehicle to confirm.

:func:`classify` labels a DID by its high byte: ``0x90xx`` is the identification range
(VIN-adjacent, the enhanced counterpart of generic OBD Mode $09), ``0x41xx``/``0x42xx`` are
the dense GM manufacturer-enhanced blocks, and the rest are other GM-enhanced identifiers.
:func:`parse_read_response` splits a positive ``0x62`` response into DID and value bytes.
"""

from __future__ import annotations

from collections.abc import Iterator
from dataclasses import dataclass

READ_DATA_BY_IDENTIFIER = 0x22
READ_POSITIVE_RESPONSE = 0x62
WRITE_DATA_BY_IDENTIFIER = 0x2E


@dataclass(frozen=True, slots=True)
class Did:
    """One Data Identifier and the category its high byte places it in."""

    value: int
    category: str

    def __post_init__(self) -> None:
        if not 0x0000 <= self.value <= 0xFFFF:
            raise ValueError(f"did out of range: {self.value:#x}")

    @property
    def hex(self) -> str:
        """The 4-hex-digit identifier, e.g. ``"41AD"``."""
        return f"{self.value:04X}"


def classify(did: int) -> str:
    """The category label for a DID, by its high byte."""
    if not 0x0000 <= did <= 0xFFFF:
        raise ValueError(f"did out of range: {did:#x}")
    high = did >> 8
    if high == 0x90:
        return "identification"
    if high in (0x41, 0x42):
        return "gm-enhanced-dense"
    if high in (0xF1, 0xF18, 0xF19):  # ISO 14229-1 identification DIDs, if ever present
        return "identification"
    return "gm-enhanced"


# Voltage 2.3.0 — the 115 ReadDataByIdentifier (0x22) DIDs, grouped by high byte
# (docs/sources.md). The 0x62 demo pair 22002F -> 62002F6C confirmed 002F.
_READ_DIDS: tuple[int, ...] = (
    0x002F,
    0x40D4,
    0x4184,
    0x4185,
    0x4186,
    0x4187,
    0x4188,
    0x4189,
    0x4190,
    0x4191,
    0x4192,
    0x4196,
    0x4197,
    0x4198,
    0x4199,
    0x419A,
    0x419B,
    0x419C,
    0x419D,
    0x419E,
    0x41A6,
    0x41AD,
    0x41C5,
    0x41C6,
    0x4202,
    0x4203,
    0x4204,
    0x4205,
    0x4206,
    0x4207,
    0x420B,
    0x420C,
    0x420D,
    0x420E,
    0x420F,
    0x4210,
    0x4211,
    0x4212,
    0x4213,
    0x4214,
    0x4215,
    0x4216,
    0x4217,
    0x4218,
    0x4219,
    0x421A,
    0x421B,
    0x421C,
    0x4220,
    0x4221,
    0x4222,
    0x4223,
    0x4224,
    0x4225,
    0x4226,
    0x4227,
    0x4228,
    0x4229,
    0x422A,
    0x422B,
    0x422C,
    0x422D,
    0x422E,
    0x4232,
    0x4233,
    0x4234,
    0x4235,
    0x4236,
    0x4237,
    0x4238,
    0x4239,
    0x423A,
    0x423B,
    0x423C,
    0x423D,
    0x423E,
    0x423F,
    0x4240,
    0x4241,
    0x4242,
    0x4243,
    0x4244,
    0x4245,
    0x4246,
    0x4247,
    0x4248,
    0x4249,
    0x424A,
    0x424B,
    0x424C,
    0x424D,
    0x424E,
    0x424F,
    0x4250,
    0x4368,
    0x4369,
    0x436B,
    0x436C,
    0x43AE,
    0x43AF,
    0x4470,
    0x4531,
    0x5005,
    0x5006,
    0x5007,
    0x5008,
    0x5247,
    0x5248,
    0x82A3,
    0x8334,
    0x83E1,
    0x83E2,
    0x90B1,
    0x90F8,
    0x90FB,
)

# Voltage 2.3.0 — the 0x2E WriteDataByIdentifier target DIDs (coding writes).
_WRITE_DIDS: tuple[int, ...] = (0xD21B, 0xD28E)


def read_dids() -> tuple[Did, ...]:
    """The catalogued ReadDataByIdentifier (0x22) DIDs, classified."""
    return tuple(Did(value, classify(value)) for value in _READ_DIDS)


def write_dids() -> tuple[Did, ...]:
    """The catalogued WriteDataByIdentifier (0x2E) target DIDs."""
    return tuple(Did(value, classify(value)) for value in _WRITE_DIDS)


def iter_read_dids() -> Iterator[Did]:
    """Iterate the read DIDs."""
    yield from read_dids()


def count_by_category() -> dict[str, int]:
    """How many read DIDs fall in each category."""
    counts: dict[str, int] = {}
    for did in read_dids():
        counts[did.category] = counts.get(did.category, 0) + 1
    return counts


def parse_read_response(pdu: bytes) -> tuple[int, bytes]:
    """Split a positive ReadDataByIdentifier response (``0x62`` DID value...).

    Returns ``(did, value_bytes)``. Raises :class:`ValueError` if the PDU is not a ``0x62``
    response of at least 3 bytes.
    """
    if len(pdu) < 3 or pdu[0] != READ_POSITIVE_RESPONSE:
        raise ValueError("not a 0x62 ReadDataByIdentifier positive response")
    did = (pdu[1] << 8) | pdu[2]
    return did, bytes(pdu[3:])
