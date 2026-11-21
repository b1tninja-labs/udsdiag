# Instructions for coding agents

`udsdiag` is an offline model of the UDS (ISO 14229-1) and GM GMLAN diagnostic services, written in
Rust: the service and NRC catalogs, a request/response codec, and the DID, DTC, Mode $06,
RoutineControl, IOControl and SecurityAccess reference catalogs. `PLAN.md` says what is in this
slice, `ROADMAP.md` is the obligation matrix, and `docs/sources.md` records where each catalog came
from. This file takes precedence over `D:\code\AGENTS.md` for work in this project.

## Commands

Run from this directory, never from `D:\code`:

    cargo test
    cargo clippy --all-targets -- -D warnings
    cargo fmt --check
    cargo doc --no-deps          # with RUSTDOCFLAGS=-D warnings

`python -m projecthub shells verify --rust D:/code/udsdiag` runs all five gates in one pass.

## Invariants

1. **The matrix and the code agree.** A row is `[ok]` only when it names a callable and a Rust test
   that exists and would fail if the behaviour regressed (`ROADMAP.md`). projecthub reads the table
   but checks Python tests only, so keep the Rust test names in the Test cell honest by hand.
2. **No vehicle I/O, no key material.** This crate opens no bus and implements no transport (the
   ISO-TP codec is the sibling `isotpkit`). It does not derive, hold, or verify a SecurityAccess key:
   `src/security.rs` models the exchange shape and two mechanism descriptors only. Do not add a
   seed->key algorithm, a key value, a CAN interface, or a vehicle write.
3. **Protocol wording comes from `primarysource`, never memory.** Read ISO 14229-1:2013 and the other
   standards through the `iso_*` / `rfc_*` tools before stating a value or a clause. Record every
   citation in `docs/iso-map.md`. A claim with no loaded source (the SAE J2012 / ISO 15031-6 DTC
   packing) is marked unverified in the code and in `docs/iso-map.md`; do not present it as confirmed.
4. **Catalog data is sourced, not invented.** Every DID, routine, signal, DTC and Mode $06 document
   traces to a `volt` assessment document named in `docs/sources.md`. `volt` is read-only; do not
   edit it. Do not copy owner personal data (VIN, order IDs) into this tree.
5. **Standard library only.** No runtime dependencies. Rust edition 2024, MSRV 1.85. Lints live in
   the manifest `[lints]` table (see `docs/layout.md`), not in scattered `#![...]` attributes.
