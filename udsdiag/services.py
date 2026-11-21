"""The diagnostic service-identifier (SID) catalog.

Each :class:`Service` names a request SID, its mnemonic and whether it is an ISO 14229-1
service or a GM/GMLAN-specific one. A positive response echoes the request SID raised by
``0x40`` (ISO 14229-1:2013, 7.2); :func:`positive_response_sid` applies that offset and
:func:`is_response_sid` recognises it.

The ISO rows are the services defined in ISO 14229-1:2013. The GM rows (``0x1A``, ``0x3B``,
``0xA2``, ``0xA5``, ``0xA9``, ``0xAA``, ``0xAE``) are the GMLAN services named in the GM
GMG3 head-unit assessment (``docs/sources.md``); GMLAN predates and parallels UDS, so its
SIDs overlap the UDS range but carry GM semantics. A SID outside the catalog is not an
error here, only unknown: a diagnostic server may define manufacturer-specific SIDs.
"""

from __future__ import annotations

from collections.abc import Iterator
from dataclasses import dataclass

POSITIVE_RESPONSE_OFFSET = 0x40
NEGATIVE_RESPONSE_SID = 0x7F


@dataclass(frozen=True, slots=True)
class Service:
    """One diagnostic service."""

    sid: int
    name: str
    mnemonic: str
    standard: str  # "ISO 14229-1:2013" or "GMLAN"

    def __post_init__(self) -> None:
        if not 0x00 <= self.sid <= 0xFF:
            raise ValueError(f"service id out of range: {self.sid:#x}")
        if self.sid in (POSITIVE_RESPONSE_OFFSET, NEGATIVE_RESPONSE_SID):
            raise ValueError(f"reserved sid used as a service: {self.sid:#x}")

    @property
    def response_sid(self) -> int:
        """The SID of this service's positive response message."""
        return self.sid + POSITIVE_RESPONSE_OFFSET


_SERVICES: tuple[Service, ...] = (
    # ISO 14229-1:2013 — diagnostic and communication management
    Service(0x10, "DiagnosticSessionControl", "DSC", "ISO 14229-1:2013"),
    Service(0x11, "ECUReset", "ER", "ISO 14229-1:2013"),
    Service(0x27, "SecurityAccess", "SA", "ISO 14229-1:2013"),
    Service(0x28, "CommunicationControl", "CC", "ISO 14229-1:2013"),
    Service(0x29, "Authentication", "AUTH", "ISO 14229-1:2013"),
    Service(0x3E, "TesterPresent", "TP", "ISO 14229-1:2013"),
    Service(0x83, "AccessTimingParameter", "ATP", "ISO 14229-1:2013"),
    Service(0x84, "SecuredDataTransmission", "SDT", "ISO 14229-1:2013"),
    Service(0x85, "ControlDTCSetting", "CDTCS", "ISO 14229-1:2013"),
    Service(0x86, "ResponseOnEvent", "ROE", "ISO 14229-1:2013"),
    Service(0x87, "LinkControl", "LC", "ISO 14229-1:2013"),
    # ISO 14229-1:2013 — data transmission
    Service(0x22, "ReadDataByIdentifier", "RDBI", "ISO 14229-1:2013"),
    Service(0x23, "ReadMemoryByAddress", "RMBA", "ISO 14229-1:2013"),
    Service(0x24, "ReadScalingDataByIdentifier", "RSDBI", "ISO 14229-1:2013"),
    Service(0x2A, "ReadDataByPeriodicIdentifier", "RDBPI", "ISO 14229-1:2013"),
    Service(0x2C, "DynamicallyDefineDataIdentifier", "DDDI", "ISO 14229-1:2013"),
    Service(0x2E, "WriteDataByIdentifier", "WDBI", "ISO 14229-1:2013"),
    Service(0x3D, "WriteMemoryByAddress", "WMBA", "ISO 14229-1:2013"),
    # ISO 14229-1:2013 — stored data transmission (DTCs)
    Service(0x14, "ClearDiagnosticInformation", "CDTCI", "ISO 14229-1:2013"),
    Service(0x19, "ReadDTCInformation", "RDTCI", "ISO 14229-1:2013"),
    # ISO 14229-1:2013 — input/output control and routine
    Service(0x2F, "InputOutputControlByIdentifier", "IOCBI", "ISO 14229-1:2013"),
    Service(0x31, "RoutineControl", "RC", "ISO 14229-1:2013"),
    # ISO 14229-1:2013 — upload/download
    Service(0x34, "RequestDownload", "RD", "ISO 14229-1:2013"),
    Service(0x35, "RequestUpload", "RU", "ISO 14229-1:2013"),
    Service(0x36, "TransferData", "TD", "ISO 14229-1:2013"),
    Service(0x37, "RequestTransferExit", "RTE", "ISO 14229-1:2013"),
    # GMLAN (GM GMG3 head-unit assessment; see docs/sources.md)
    Service(0x1A, "ReadDataByIdentifier (GMLAN)", "GM-RDBI", "GMLAN"),
    Service(0x3B, "WriteDataByIdentifier (GMLAN)", "GM-WDBI", "GMLAN"),
    Service(0xA2, "ReportProgrammedState (GMLAN)", "GM-RPS", "GMLAN"),
    Service(0xA5, "ProgrammingMode (GMLAN)", "GM-PM", "GMLAN"),
    Service(0xA9, "ReadDiagnosticInformation (GMLAN)", "GM-RDI", "GMLAN"),
    Service(0xAA, "ReadDataByPacketIdentifier (GMLAN)", "GM-RDBPI", "GMLAN"),
    Service(0xAE, "DeviceControl (GMLAN)", "GM-DC", "GMLAN"),
)

_BY_SID: dict[int, Service] = {service.sid: service for service in _SERVICES}


def all_services() -> tuple[Service, ...]:
    """Every service in the catalog, in declaration order."""
    return _SERVICES


def iter_services() -> Iterator[Service]:
    """Iterate the catalog."""
    yield from _SERVICES


def get(sid: int) -> Service | None:
    """The service for a request SID, or ``None`` when the SID is not catalogued."""
    return _BY_SID.get(sid)


def name_of(sid: int) -> str:
    """A human label for a request SID, falling back to the hex value."""
    service = _BY_SID.get(sid)
    return service.name if service is not None else f"UnknownService({sid:#04x})"


def positive_response_sid(sid: int) -> int:
    """The positive-response SID for a request SID (request SID + ``0x40``)."""
    if not 0x00 <= sid <= 0xBF:
        raise ValueError(f"request sid has no positive response sid: {sid:#x}")
    return sid + POSITIVE_RESPONSE_OFFSET


def is_response_sid(byte: int) -> bool:
    """Whether a leading byte is a positive-response SID of some catalogued service."""
    return any(service.response_sid == byte for service in _SERVICES)


def request_sid_for_response(response_sid: int) -> int:
    """The request SID a positive-response SID answers (response SID - ``0x40``)."""
    return response_sid - POSITIVE_RESPONSE_OFFSET
