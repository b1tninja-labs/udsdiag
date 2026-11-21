"""udsdiag: an offline model of the UDS (ISO 14229-1) and GM GMLAN diagnostic services.

The package carries the service and negative-response-code catalogs, a request/response
codec, and the DID, DTC, Mode $06, RoutineControl, IOControl and SecurityAccess reference
catalogs drawn from a GM head-unit firmware and third-party-app assessment. It encodes and
decodes diagnostic messages and looks identifiers up; it does not open a bus, derive a
SecurityAccess key, or write to a vehicle.
"""

from __future__ import annotations

__version__ = "0.0.0"
