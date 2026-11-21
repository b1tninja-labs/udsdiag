# Sources

`udskit` is a model of the UDS (ISO 14229-1) and GM GMLAN diagnostic services. Two kinds of
source feed it: the normative standards (read through `primarysource`; mapped in
[`iso-map.md`](iso-map.md)) and the catalogs recovered in an authorized assessment of a GM
vehicle the owner holds. This file records the provenance of the catalog data. Nothing here
was produced against a live vehicle; the package opens no bus and writes nothing.

## Normative standards

- **ISO 14229-1:2013** — Unified diagnostic services (UDS), Part 1. The service model, the
  negative-response-code table (Annex A, Table A.1), RoutineControl sub-functions (13.2),
  SecurityAccess (9.4), the positive-response `+0x40` convention (7.2). Read through the
  `primarysource` `iso_*` tools; see [`iso-map.md`](iso-map.md).
- **ISO 15765-2:2016** — Diagnostic communication over CAN (DoCAN), transport protocol
  (ISO-TP). Named here because `message.py` works on the reassembled PDU and leaves the
  segmentation to a transport layer; the ISO-TP codec is the sibling `isotpkit`.
- **SAE J1979 / ISO 15031-5** — generic OBD-II services (Mode $06 etc.); referenced by
  `mode06.py`. The GM Mode $06 documents themselves are the sibling `mode06kit`'s source.
- **SAE J2012 / ISO 15031-6** — DTC format. The two-byte DTC packing in `dtc.encode` /
  `dtc.decode` follows this convention. Neither document is in `primarysource`, so those two
  functions are marked unverified against the normative source (see [`iso-map.md`](iso-map.md)).

## Assessment catalogs (GM GMG3 head unit + Voltage 2.3.0 app)

The catalog data comes from the read-only write-ups under `D:\code\volt\docs\` (treated as
read-only; this package copies no vehicle data beyond protocol identifiers). The relevant
documents:

- **`DIAGNOSTICS.md`** — the GMG3 head-unit `procdiagnosis` service-dispatch tree, read
  statically from the shipped ELF process images. Feeds: `services.py` (the GMLAN service
  aliases and the `0x31`/`0x2F`/`0x27` dispatch), `routine.py` (the resolved routine IDs:
  StressTest `0x3202`, SetSystemToDeliveryState `0xD001`, RunIn `0x3200`/`0x3201`, GNSS
  `0x3069`, iPod `0x3040`, DisplaySlideshow `0x3204`), `iocontrol.py` (the full
  `DIA_EN_IOCTRL_SIGTYPE_*` signal enumeration and parameter actions).
- **`FINDINGS.md`** — the ADIT ALD challenge/response mechanism (RSA-2048 per-level
  `level.pub`, the 96-byte device-bound challenge, NRC `0x35` on a bad key). Feeds
  `security.py`'s `adit-ald` mechanism descriptor.
- **`OBD-CONFIG-DID-APPENDIX.md`** (on branch `claude/obd-config-re-887200`, volt PR #2) —
  the Voltage 2.3.0 identifier enumeration recovered from the app's demo-mode process memory:
  the 115 `0x22` DIDs, the `0x2E` write targets (`D21B`, `D28E`), the `0x31` routine
  invocations, the `0x27` sub-functions (`2701`, `2796`), and the GMLAN `+0x400`/`+0x300`
  addressing. Feeds `did.py` and `gmlan.py`. The appendix deliberately carries **no**
  SecurityAccess seed/key algorithm or key material, and neither does this package.
- **`GM-DTC-REFERENCE.md`** (on branch `claude/obd-config-re-887200`) — the cross-reference
  of the app's 841-code DTC-name table against GM's `17OBDG01 Hybrid Diagnostics`. Feeds the
  representative subset in `dtc.py` and the per-prefix counts (`APP_DTC_COUNTS`). The full
  841-row table is the sibling `gmdtcdb`'s dataset.
- **`GM-OBD-REFERENCES.md`** — the catalog of GM's own published 2017 Mode $06 PDFs and the
  OBD-group matrix (Volt = Group 1A). Feeds `mode06.py` (document catalog only; the parser is
  `mode06kit`).

## Scope boundary

This package models the diagnostic protocol and catalogs identifiers. It does **not**: open a
CAN interface or any transport; derive, hold, or verify a SecurityAccess key; or write to a
vehicle. The SecurityAccess material is limited to the exchange shape and two mechanism
descriptors (`security.py`), consistent with the assessment's own scope boundary
(`volt/docs/OBD-CONFIG-SCOPE.md`).
