# ISO / standard map

Every protocol claim this package makes is anchored to a standard read through the
`primarysource` MCP tools, never stated from memory (a `D:\code\AGENTS.md` rule). This file is
the index: which clause backs which module, and which claims have **no** primary source loaded
and are therefore marked unverified in the code.

## Confirmed against a loaded source

| Claim | Module | Standard | Where |
|---|---|---|---|
| Negative-response-code values and names (0x10 generalReject … 0x78 responsePending) | `nrc.py` | ISO 14229-1:2013 | Annex A (normative), Table A.1 — `iso:14229-1@2013§15.4.4` (Table A.1 body) |
| Positive response = request SID + 0x40; negative response = `0x7F <SID> <NRC>` | `services.py`, `message.py` | ISO 14229-1:2013 | 7.2, 7.5.6 |
| RoutineControl sub-functions: 0x01 startRoutine (M), 0x02 stopRoutine (U), 0x03 requestRoutineResults | `routine.py` | ISO 14229-1:2013 | 13.2; request example tables confirm the `0x31 / 0x01-0x03` bytes |
| SecurityAccess pairing: requestSeed odd ↔ sendKey = requestSeed + 1; NRC 0x35 invalidKey | `security.py` | ISO 14229-1:2013 | 9.4.2; "requestSeed = 0x01 … sendKey = 0x02" |
| requestOutOfRange 0x31, securityAccessDenied 0x33, invalidKey 0x35 definitions | `nrc.py` | ISO 14229-1:2013 | Table A.1 (read in full) |
| ISO-TP carries multi-frame UDS PDUs (segmentation out of scope here) | `message.py` docstring | ISO 15765-2:2016 | referenced; the codec is the sibling `isotpkit` |
| Mode $06 / Mode $09 are the generic OBD-II services | `mode06.py` | SAE J1979 / ISO 15031-5 | named; GM's own PDFs are the sibling `mode06kit`'s source |

The ISO 14229-1 service SID values (0x10, 0x11, 0x14, 0x19, 0x22, 0x23, 0x24, 0x27, 0x28,
0x29, 0x2A, 0x2C, 0x2E, 0x2F, 0x31, 0x34–0x37, 0x3D, 0x3E, 0x83–0x87) are the service
identifiers defined per clause in ISO 14229-1:2013; the ones this assessment exercised
(`0x22`, `0x2E`, `0x27`, `0x2F`, `0x31`, `0x19`, `0x14`, `0x10`, `0x11`) are additionally
corroborated against the GMG3 firmware dispatch tree and the app's captured frames
(`docs/sources.md`).

## Not in `primarysource` — marked unverified in code

| Claim | Module | Would-be source | Status |
|---|---|---|---|
| Two-byte DTC packing: top-2 bits of byte 1 → P/C/B/U; nibbles → digits | `dtc.encode` / `dtc.decode` | SAE J2012 / ISO 15031-6 | **Unverified.** Implements the conventional format; the module docstring and `encode`/`decode` say so. Neither J2012 nor ISO 15031-6 is loaded (`iso_docs` lists 15031-3 only). |
| GMLAN `+0x400` response / `+0x300` filter 11-bit addressing | `gmlan.py` | GM GMW3110 (GMLAN) | GM-proprietary convention, taken from the app capture in `docs/sources.md`, not a public standard. The OBD-II `0x7E0+8` physical response offset is ISO 15765-4. |
| GMLAN service aliases ($1A, $3B, $AE, …) | `services.py` | GM GMW3110 (GMLAN) | GM-specific; sourced from the firmware/app assessment, flagged `standard="GMLAN"` in the catalog. |

If any of these documents is later fetched into `primarysource`, read it and replace the
"unverified" note with a clause citation here.
