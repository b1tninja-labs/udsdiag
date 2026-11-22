# udskit

An offline model of the UDS (ISO 14229-1:2013) and GM GMLAN diagnostic services, split out of the
`volt` GM head-unit and OBD-app assessment so the protocol model is reusable apart from the research
write-ups. Written in Rust (edition 2024, MSRV 1.85), standard library only.

## This crate

- The diagnostic service catalog (`services`) and the negative-response-code catalog (`nrc`), from
  ISO 14229-1:2013.
- The request/response codec (`message`): build a request, decode a positive or negative response,
  recognise response-pending. Application PDU only.
- The reference catalogs recovered in the assessment: the DID set (`did`), the DTC model, status-byte
  decoder and a representative subset (`dtc`), the RoutineControl and IOControl catalogs (`routine`,
  `iocontrol`), the SecurityAccess exchange model and mechanism descriptors (`security`), the GM
  Mode $06 document catalog (`mode06`), and GMLAN addressing (`gmlan`).
- A read-only CLI over all of it (`src/main.rs`).

## Architecture: core + vendor layers

The service catalog core is pure ISO 14229-1; manufacturer layers plug in through
`vendor::VendorProfile` rather than being baked into it. `gm::Gm` is the first vendor layer — the
GMLAN services and `+0x400`/`+0x300` addressing — and `vendor::resolve_service` composes the ISO
core with any number of vendor profiles. The GM layer lives in this crate for now; the trait
boundary lets it be lifted into its own crate later without touching the core. The other
GM-sourced catalogs (routine, iocontrol, security, DID, Mode $06) are being routed through the
profile incrementally (`ROADMAP.md`).

## Not this crate

- **No transport.** The ISO-TP (ISO 15765-2) codec that segments and reassembles a PDU is the sibling
  `isotpkit`. `message` works on the reassembled PDU.
- **No vehicle I/O and no key material.** No CAN interface, no vehicle write, no SecurityAccess
  seed->key algorithm or key.
- **Not the full DTC database.** The complete 841-code app table joined to GM 17OBDG01 is the sibling
  `gmdtcdb`; this crate carries a representative subset and the counts.
- **Not the Mode $06 parser.** Parsing GM's Mode $06 PDFs into records is the sibling `mode06kit`;
  this crate catalogs the documents only.

## History

This crate began as a Python package and was ported to Rust in full (same scope, same anchors, same
obligation matrix) when the portfolio moved its live diagnostic tooling to Rust. The Python history
is in the git log; the Rust port is the current and only implementation.

## Siblings

`isotpkit` (transport under this), `mode06kit` (Mode $06 parser), `gmdtcdb` (full DTC database),
`voltkit` / `spskit` / `techlinekit` (the firmware-read, reflash and TDS-client sides). None imports
another. All trace to the same read-only `volt` assessment.
