# SecurityAccess: the two mechanisms, and the scope boundary

SecurityAccess (`0x27`) is the UDS service that unlocks protected functions. `udskit` models
the **exchange shape** and names the **two mechanisms** the GM assessment distinguished. It
carries no key material, no seed→key algorithm, and no key-derivation method, and it will not
compute a key. That boundary is deliberate and matches the assessment's own
(`volt/docs/OBD-CONFIG-SCOPE.md`).

## The exchange shape (ISO 14229-1:2013, 9.4)

1. Client → `0x27 <requestSeed level>` (an odd sub-function: `0x01`, `0x03`, …).
2. Server → seed.
3. Client → `0x27 <sendKey level> <key>` where `sendKey = requestSeed + 1` (`0x02`, `0x04`, …).
4. Server → unlocked, or `0x7F 0x27 0x35` (NRC `0x35` invalidKey).

`security.py` provides the pairing helpers (`is_request_seed`, `send_key_for`, …) and
`build_request_seed`; it does **not** provide a `send_key` that carries a key.

## Mechanism 1 — classic GM seed/key

Classic GM modules return a short seed and expect a key computed from it by a per-module
secret algorithm held by GM. The Voltage app references `0x27` sub-functions `2701` and `2796`
(requestSeed only; the appendix captured no seed, key, or algorithm). `udskit` records this
as the `classic-gm-seed-key` descriptor — family `seed-key` — and nothing more.

## Mechanism 2 — the GMG3 head unit's ADIT ALD

On the GM GMG3 infotainment head unit, SecurityAccess `0x27` is a thin front-end over the
**ADIT Access-Level Daemon (ALD)**, a signed challenge/response rather than a seed/key:

- The server builds a **96-byte device-bound challenge**: a 16-byte `rand()` nonce, an 8-byte
  `gettimeofday` timestamp, an 8-byte i.MX6 chip serial (zeroed if absent), and a 16-byte
  caller-supplied ECU_ID (with a gap; the verified region is the whole 96 bytes).
- The tester returns an **RSA-2048 signed block**. The daemon verifies the signature against
  the level's public key (`level.pub`) **and** `memcmp`s the response against the live
  challenge. Either mismatch → NRC `0x35`.
- Access is per-level (`security_levels/{0,10,20,30,35,40,50}`); level 50 grants everything
  (SSH, root login, USB-ethernet, …). A per-level V850 bitmask decides which companion-MCU
  operations unlock.
- The per-level **private** keys are held by GM/Bosch. They are not in this package and are
  out of scope. The challenge is time-bounded by a per-request glib timer
  (`CHALLENGE_TIMEOUT`), not by the timestamp inside the challenge.

`udskit` records this as the `adit-ald` descriptor — family `challenge-response`. The full
decompiled mechanism is in `volt/docs/FINDINGS.md`; this package summarizes it, it does not
reproduce or exercise it.
