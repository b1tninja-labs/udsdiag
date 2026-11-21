"""RoutineControl (``0x31``): sub-functions, request building, and the routine catalog.

RoutineControl runs a server-side routine named by a 2-byte routineIdentifier. The three
sub-functions are ``0x01`` startRoutine, ``0x02`` stopRoutine and ``0x03`` requestRoutineResults
(ISO 14229-1:2013, 13.2; startRoutine is mandatory, the other two user-optional).
:func:`build_request` assembles ``0x31 <sub-function> <routineId MSB> <routineId LSB> [data]``.

The routine catalog is the set resolved in the GM GMG3 head-unit's ``procdiagnosis`` image
(``docs/sources.md``), each routine's on-wire ID and effect. These are GM-specific routines on
one module, not ISO-defined identifiers. ``risk`` flags the two sharpest ones: ``0x3202``
StressTest (a fixed ``execv("/usr/bin/stress", argv)`` with an attacker-chosen argument vector
— resource-exhaustion DoS, not arbitrary command execution) and ``0xD001``
SetSystemToDeliveryState (factory/shipping reset). All leaves are gated by SecurityAccess
(``0x27`` -> ALD); see :mod:`udsdiag.security`.
"""

from __future__ import annotations

from dataclasses import dataclass

ROUTINE_CONTROL = 0x31

START_ROUTINE = 0x01
STOP_ROUTINE = 0x02
REQUEST_ROUTINE_RESULTS = 0x03

_SUBFUNCTIONS: dict[int, str] = {
    START_ROUTINE: "startRoutine",
    STOP_ROUTINE: "stopRoutine",
    REQUEST_ROUTINE_RESULTS: "requestRoutineResults",
}


def subfunction_name(subfunction: int) -> str:
    """The name of a RoutineControl sub-function byte."""
    try:
        return _SUBFUNCTIONS[subfunction]
    except KeyError:
        raise ValueError(f"not a RoutineControl sub-function: {subfunction:#04x}") from None


def build_request(subfunction: int, routine_id: int, data: bytes = b"") -> bytes:
    """Build a RoutineControl request PDU."""
    if subfunction not in _SUBFUNCTIONS:
        raise ValueError(f"not a RoutineControl sub-function: {subfunction:#04x}")
    if not 0x0000 <= routine_id <= 0xFFFF:
        raise ValueError(f"routine id out of range: {routine_id:#x}")
    return bytes([ROUTINE_CONTROL, subfunction, routine_id >> 8, routine_id & 0xFF]) + data


@dataclass(frozen=True, slots=True)
class Routine:
    """One concrete routine on the assessed module."""

    routine_id: int
    name: str
    effect: str
    risk: str  # "H" | "M" | "L"

    def __post_init__(self) -> None:
        if not 0x0000 <= self.routine_id <= 0xFFFF:
            raise ValueError(f"routine id out of range: {self.routine_id:#x}")

    @property
    def hex(self) -> str:
        """The 4-hex-digit routine identifier."""
        return f"{self.routine_id:04X}"


# GMG3 head-unit procdiagnosis routines (docs/sources.md).
_CATALOG: tuple[Routine, ...] = (
    Routine(0x3202, "StressTest", "fork()+execv(/usr/bin/stress, argv) with attacker-chosen args (DoS)", "H"),
    Routine(0xD001, "SetSystemToDeliveryState", "factory / shipping-state reset (de-personalise the unit)", "H"),
    Routine(0x3200, "RunIn", "end-of-line burn-in start", "M"),
    Routine(0x3201, "RunInClearResults", "clear burn-in results", "M"),
    Routine(0x3069, "GNSSCalcAndVerifyChecksum", "GNSS firmware CRC verify", "L"),
    Routine(0x3040, "IPODSelfTest", "iPod / USB self-test", "L"),
    Routine(0x3204, "DisplayTestScreenSlideshow", "pushes display test patterns", "L"),
)

_BY_ID: dict[int, Routine] = {routine.routine_id: routine for routine in _CATALOG}


def catalog() -> tuple[Routine, ...]:
    """The routine catalog."""
    return _CATALOG


def get(routine_id: int) -> Routine | None:
    """The routine for an on-wire ID, or ``None`` when it is not catalogued."""
    return _BY_ID.get(routine_id)
