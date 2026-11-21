"""The GM Mode $06 document catalog.

Mode $06 (On-Board Monitoring Test Results) and Mode $09 (Vehicle Information) are the generic
OBD-II services of SAE J1979 / ISO 15031-5. Manufacturers publish the per-engine-family
diagnostic-parameter files and the Mode $06 monitor/test-ID definitions for the independent
aftermarket, as the CARB/EPA OBD-II service-information rules require. This module catalogs
GM's own published 2017 OBD reference PDFs (``docs/sources.md``): these are GM documents, not
reverse-engineered material.

They are the *standardized* OBD-II counterpart to the *enhanced* GMLAN DIDs/routines the rest
of this package models — complementary, not overlapping. For the Chevrolet Volt, which is OBD
Group 1A, the applicable files are ``17OBDG01 Hybrid Diagnostics`` and the ``GM CAN mode $06
data`` reference; :func:`files_for_group` and :data:`VOLT_OBD_GROUP` encode that.
"""

from __future__ import annotations

from dataclasses import dataclass

GM_MODE6_BASE = "https://gsitlc.ext.gm.com/gmspo/mode6/pdf/"
VOLT_OBD_GROUP = "1A"

MODE_06_ON_BOARD_MONITORING = 0x06
MODE_09_VEHICLE_INFORMATION = 0x09


@dataclass(frozen=True, slots=True)
class Document:
    """One GM OBD reference PDF."""

    filename: str
    obd_groups: tuple[str, ...]
    what: str


# GM 2017 OBD reference PDFs (docs/sources.md). obd_groups () marks a generic (all-vehicle) file.
_DOCS: tuple[Document, ...] = (
    Document("2017_OBD_GROUP_SUMMARY.pdf", (), "index: Engine/Transmission diagnostic parameter file matrix"),
    Document(
        "17OBDG01 Hybrid Diagnostics.pdf", ("1A", "1B"), "Hybrid powertrain diagnostic parameters (Volt, ELR, CT6)"
    ),
    Document("17OBDG02 Hybrid Diagnostics.pdf", ("2",), "Hybrid powertrain diagnostic parameters (Malibu 1.8L hybrid)"),
    Document("17OBDG03 Engine Diagnostics.pdf", ("3",), "Engine diagnostic parameters (conventional gas 1.4L-6.2L)"),
    Document("17OBDG03 Transmisson Diagnostics.pdf", ("3",), "Transmission diagnostic parameters (GM's spelling)"),
    Document(
        "17OBDG04 Engine Diagnostics.pdf", ("4",), "Engine diagnostic parameters (diesels: LH7, LWN, L5P Duramax)"
    ),
    Document("17OBDG04 Transmisson Diagnostics.pdf", ("4",), "Transmission diagnostic parameters (GM's spelling)"),
    Document("17OBDG05 Engine Diagnostics.pdf", ("5",), "Engine diagnostic parameters (6.0L L96 HD)"),
    Document("17OBDG05 Transmission Diagnostics.pdf", ("5",), "Transmission diagnostic parameters"),
    Document("17OBDG06 Engine Diagnostics.pdf", ("6",), "Engine diagnostic parameters (6.0L L96)"),
    Document("17OBDG06 Transmission Diagnostics.pdf", ("6",), "Transmission diagnostic parameters"),
    Document(
        "GM CAN mode $06 data final_dm.pdf", (), "Generic Mode $06 monitor/test-ID definitions (CAN/GMLAN, ISO 15031-5)"
    ),
    Document(
        "GM Class2 mode $06 data final_dm.pdf", (), "Generic Mode $06 monitor/test-ID definitions (legacy Class 2)"
    ),
)


def documents() -> tuple[Document, ...]:
    """Every catalogued GM OBD reference PDF."""
    return _DOCS


def files_for_group(group: str) -> tuple[Document, ...]:
    """The documents applicable to an OBD group: its group-specific files plus the generic ones."""
    return tuple(doc for doc in _DOCS if group in doc.obd_groups or not doc.obd_groups)


def volt_files() -> tuple[Document, ...]:
    """The documents applicable to the Chevrolet Volt (OBD Group 1A)."""
    return files_for_group(VOLT_OBD_GROUP)
