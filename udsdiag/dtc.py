"""The Diagnostic Trouble Code (DTC) model and reference catalog.

A DTC is read with ReadDTCInformation (``0x19``) and cleared with ClearDiagnosticInformation
(``0x14``) (ISO 14229-1:2013). Its on-wire form is two bytes; its display form is a letter and
four hex digits (e.g. ``P0A7F``). The letter comes from the top two bits of the first byte:
``P`` powertrain, ``C`` chassis, ``B`` body, ``U`` network. That two-byte packing is the
conventional SAE J2012 / ISO 15031-6 format; neither document is in ``primarysource`` (see
``docs/iso-map.md``), so :func:`encode`/:func:`decode` implement the convention and are marked
unverified against the normative source.

The reference catalog is a representative subset of the cross-reference between a third-party
Volt app's DTC-name table (841 codes in its ``classes.dex``) and GM's own ``17OBDG01 Hybrid
Diagnostics`` monitor-certification document for the Volt (OBD Group 1A); see ``docs/sources.md``.
The per-prefix totals are recorded in :data:`APP_DTC_COUNTS`.
"""

from __future__ import annotations

import re
from dataclasses import dataclass

READ_DTC_INFORMATION = 0x19
CLEAR_DIAGNOSTIC_INFORMATION = 0x14

_LETTER_TO_BITS: dict[str, int] = {"P": 0b00, "C": 0b01, "B": 0b10, "U": 0b11}
_BITS_TO_LETTER: dict[int, str] = {bits: letter for letter, bits in _LETTER_TO_BITS.items()}

_CATEGORY: dict[str, str] = {
    "P": "powertrain",
    "C": "chassis",
    "B": "body",
    "U": "network",
}

_DISPLAY = re.compile(r"^([PCBU])([0-9A-F])([0-9A-F])([0-9A-F])([0-9A-F])$")

# App DTC-name table totals, per prefix, from the Voltage 2.3.0 classes.dex extraction.
APP_DTC_COUNTS: dict[str, int] = {"P": 309, "B": 139, "C": 63, "U": 330}
APP_DTC_TOTAL = 841


@dataclass(frozen=True, slots=True)
class Dtc:
    """One diagnostic trouble code with its app-side name and GM-document coverage."""

    code: str
    name: str
    in_17obdg01: bool
    note: str = ""

    def __post_init__(self) -> None:
        if not _DISPLAY.match(self.code):
            raise ValueError(f"not a DTC display code: {self.code!r}")

    @property
    def category(self) -> str:
        """``powertrain`` / ``chassis`` / ``body`` / ``network``."""
        return _CATEGORY[self.code[0]]


def category_of(code: str) -> str:
    """The category of a DTC display code, from its leading letter."""
    letter = code[:1].upper()
    if letter not in _CATEGORY:
        raise ValueError(f"not a DTC display code: {code!r}")
    return _CATEGORY[letter]


def encode(code: str) -> bytes:
    """Pack a display code (e.g. ``P0A7F``) into its two on-wire bytes.

    Conventional SAE J2012 / ISO 15031-6 packing (unverified against the normative source;
    see the module docstring).
    """
    match = _DISPLAY.match(code.upper())
    if not match:
        raise ValueError(f"not a DTC display code: {code!r}")
    letter, d1, d2, d3, d4 = match.groups()
    byte0 = (_LETTER_TO_BITS[letter] << 6) | (int(d1, 16) << 4) | int(d2, 16)
    byte1 = (int(d3, 16) << 4) | int(d4, 16)
    return bytes([byte0, byte1])


def decode(raw: bytes) -> str:
    """Unpack two on-wire bytes into a display code (the inverse of :func:`encode`)."""
    if len(raw) != 2:
        raise ValueError(f"a DTC is two bytes, got {len(raw)}")
    byte0, byte1 = raw[0], raw[1]
    letter = _BITS_TO_LETTER[(byte0 >> 6) & 0b11]
    d1 = (byte0 >> 4) & 0b11
    d2 = byte0 & 0x0F
    return f"{letter}{d1:X}{d2:X}{byte1:02X}"


# Representative cross-reference subset (docs/sources.md). in_17obdg01 marks codes GM's
# 17OBDG01 Hybrid Diagnostics (Volt, OBD Group 1A) carries in its powertrain cert set.
_CATALOG: tuple[Dtc, ...] = (
    # High-voltage battery & HV system
    Dtc("P0A0D", "High Voltage System Interlock Circuit Voltage", True, "HVIL safety interlock"),
    Dtc("P0A7E", "Hybrid/EV Battery Pack High Temperature", True),
    Dtc("P0A7F", "Hybrid/EV Battery Pack", True, "GM scopes it to pack capability / end-of-life"),
    Dtc("P0AE3", "Hybrid/EV Battery Precharge Contactor Stuck Open", True),
    Dtc("P0AF8", "Hybrid System Voltage", True, "pack vs TPIM/charger bus correlation > 16 V"),
    Dtc("P0AFB", "Hybrid/EV Battery System Voltage", True, "pack or cell over-voltage"),
    Dtc("P0B13", "Hybrid/EV Battery Current Sensor", True, "fine vs coarse current > 10 A"),
    Dtc("P0C76", "Hybrid/EV Battery System High Voltage Present", True, "HV discharge time too long"),
    Dtc("P0D5E", "Battery Charger System High Voltage Present", True),
    # Drive motor & inverter (TPIM)
    Dtc("P0A1C", "Drive Motor Control Module Performance", True),
    Dtc("P0A79", "Drive Motor Inverter Performance", True, "IGBT desaturation"),
    Dtc("P0C04", "Drive Motor High Current", True),
    Dtc("P0C5C", "Drive Motor 1 Position Sensor", True, "resolver circuit low"),
    # On-board charger & charge cord
    Dtc("P0CF6", "Control Pilot Circuit", True, "J1772 pilot out-of-range high"),
    Dtc("P0D26", "Battery Charger System Precharge Time Too Long", True),
    Dtc("P0D59", "Proximity Detection Circuit Low/High Voltage", True, "charge-cord latch"),
    # Engine / range-extender (ECM)
    Dtc("P1101", "Intake Air Flow System Performance", True),
    Dtc("P2097", "Post Catalyst Fuel Trim System", True),
    Dtc("P2138", "Accelerator Pedal Position (APP) Sensor", True),
    # Communication — in 17OBDG01 powertrain set
    Dtc("U0073", "Control Module Communication Bus A Off", True),
    Dtc("U0100", "Lost Communication With Engine Control Module", True),
    Dtc("U0111", "Lost Communication with Battery Energy Control Module", True),
    Dtc("U0140", "Lost Communication With Body Control Module", True),
    # Communication — NOT in 17OBDG01 (module the app decodes)
    Dtc("U0121", "Lost Communication With Electronic Brake Control Module", False, "module: EBCM"),
    Dtc("U0146", "Lost Communication with Serial Data Gateway Module", False),
    Dtc("U0155", "Lost Communication With Instrument Cluster", False, "module: IPC"),
    Dtc("U0184", "Lost Communication With Radio", False, "module: Radio (head unit)"),
    Dtc("U0256", "Lost Communication (faceplate)", False, "named in OBD-CONFIG.md"),
    # Chassis / brake (BSC section of 17OBDG01)
    Dtc("C0075", "Brake Pedal Position Sensor Performance", True),
    Dtc("C0591", "Brake Booster Motor", True),
    Dtc("C10EA", "Powertrain Control Module Indicated Requested Torque", True),
)

_BY_CODE: dict[str, Dtc] = {entry.code: entry for entry in _CATALOG}


def catalog() -> tuple[Dtc, ...]:
    """The representative DTC reference subset."""
    return _CATALOG


def get(code: str) -> Dtc | None:
    """The reference entry for a display code, or ``None`` if not in the subset."""
    return _BY_CODE.get(code.upper())
