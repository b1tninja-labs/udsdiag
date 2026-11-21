"""GMLAN addressing conventions (GM single-wire / low-speed CAN).

GM's GMLAN 11-bit addressing offsets a module's physical **response** CAN ID from its
**request** ID by ``+0x400`` and the flow/acceptance **filter** by ``+0x300``: a request to
``0x241`` is answered on ``0x641`` and accepted through filter ``0x541`` (``docs/sources.md``).
The generic ISO 15765-4 OBD-II layer is separate: the functional broadcast is ``0x7DF`` and a
physical request to ``0x7E0..0x7E7`` is answered on ``0x7E8..0x7EF``.

:func:`response_id` and :func:`filter_id` apply the GMLAN offsets; :func:`obd_response_id`
applies the OBD-II physical offset. These are addressing helpers only — this package does not
open a CAN interface.
"""

from __future__ import annotations

GMLAN_RESPONSE_OFFSET = 0x400
GMLAN_FILTER_OFFSET = 0x300

OBD_FUNCTIONAL_BROADCAST = 0x7DF
OBD_PHYSICAL_REQUEST_LOW = 0x7E0
OBD_PHYSICAL_REQUEST_HIGH = 0x7E7
OBD_PHYSICAL_RESPONSE_OFFSET = 0x08


def _check_11bit(can_id: int) -> None:
    if not 0x000 <= can_id <= 0x7FF:
        raise ValueError(f"not an 11-bit CAN id: {can_id:#x}")


def response_id(request_id: int) -> int:
    """The GMLAN physical response ID for a request ID (request + ``0x400``)."""
    _check_11bit(request_id)
    result = request_id + GMLAN_RESPONSE_OFFSET
    _check_11bit(result)
    return result


def filter_id(request_id: int) -> int:
    """The GMLAN flow/acceptance filter ID for a request ID (request + ``0x300``)."""
    _check_11bit(request_id)
    result = request_id + GMLAN_FILTER_OFFSET
    _check_11bit(result)
    return result


def obd_response_id(request_id: int) -> int:
    """The ISO 15765-4 OBD-II physical response ID for a ``0x7E0..0x7E7`` request (request + 8)."""
    if not OBD_PHYSICAL_REQUEST_LOW <= request_id <= OBD_PHYSICAL_REQUEST_HIGH:
        raise ValueError(f"not an OBD-II physical request id: {request_id:#x}")
    return request_id + OBD_PHYSICAL_RESPONSE_OFFSET
