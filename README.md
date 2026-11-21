# udsdiag

An offline model of the **UDS** (ISO 14229-1:2013) and **GM GMLAN** diagnostic services. It
carries the service and negative-response-code catalogs, a request/response codec, and the
DID, DTC, Mode $06, RoutineControl, IOControl and SecurityAccess reference catalogs drawn from
an authorized assessment of a GM vehicle the owner holds. It **encodes and decodes diagnostic
messages and looks identifiers up**; it does not open a bus, derive a SecurityAccess key, or
write to a vehicle.

```text
python -m udsdiag services          # the diagnostic service catalog
python -m udsdiag nrc [31]           # negative response codes, or one by hex value
python -m udsdiag did                # the Voltage 2.3.0 DID catalog
python -m udsdiag dtc [P0A7F]        # DTC reference entries, or one by code
python -m udsdiag routines           # the RoutineControl catalog
python -m udsdiag iocontrol          # the IOControl signal groups
python -m udsdiag security           # the SecurityAccess mechanisms
python -m udsdiag mode06             # the GM Mode $06 document catalog
python -m udsdiag decode 7F2231      # decode a response PDU from hex
python -m udsdiag gmlan 241          # GMLAN response/filter IDs for a request ID
```

## What it is

- `services` — the UDS (ISO 14229-1:2013) and GMLAN service-identifier catalog, with the
  positive-response `+0x40` relationship.
- `nrc` — the negative-response-code catalog (ISO 14229-1:2013, Annex A, Table A.1).
- `message` — the request/response codec: build a request PDU, decode a positive or negative
  response, recognise `0x78` response-pending. Works on the reassembled application PDU; the
  ISO-TP transport under it is the sibling `isotpkit`.
- `did` — the 115-DID ReadDataByIdentifier catalog a Volt app references, classified by range.
- `dtc` — the DTC display/on-wire model and a representative reference subset (the full
  841-code database is the sibling `gmdtcdb`).
- `routine` / `iocontrol` — RoutineControl (`0x31`) and IOControl (`0x2F`/GM `$AE`): request
  builders plus the routine and signal catalogs resolved from a GM head unit.
- `security` — the SecurityAccess exchange model and two mechanism descriptors (classic GM
  seed/key, and the head unit's ADIT ALD RSA-2048 challenge/response). **No key material.**
- `mode06` — the catalog of GM's own published Mode $06 reference PDFs (the parser is the
  sibling `mode06kit`).
- `gmlan` — the GMLAN `+0x400`/`+0x300` addressing convention.

## Boundaries

This is a reference and codec library, not a vehicle tool. It has no transport, no CAN
interface, and no SecurityAccess key derivation by design. Every protocol claim is anchored to
a standard read through `primarysource` and recorded in [`docs/iso-map.md`](docs/iso-map.md);
the DTC two-byte packing follows SAE J2012 / ISO 15031-6, which is not in `primarysource` and
is marked unverified. Provenance of the catalog data is in [`docs/sources.md`](docs/sources.md).
Standard library only; Apache-2.0.
