# udsdiag

An offline model of the UDS (ISO 14229-1:2013) and GM GMLAN diagnostic services, ported to Rust:
the service and negative-response-code catalogs, a request/response PDU codec, and the DID, DTC,
Mode $06, RoutineControl, IOControl and SecurityAccess reference catalogs recovered in an
authorised assessment of a GM vehicle the owner holds. It encodes and decodes application-layer
PDUs and looks identifiers up. It opens no bus, implements no transport, and holds no
SecurityAccess key material — the same scope boundary as the Python original it replaces.

Protocol wording is read through the `primarysource` `iso_*` tools and mapped in `docs/iso-map.md`,
never stated from memory. The two-byte DTC packing follows SAE J2012 / ISO 15031-6, which is not in
`primarysource`; it is marked unverified in the code and in `docs/iso-map.md`. Catalog provenance is
in `docs/sources.md`. Standard library only; Apache-2.0.

**Non-goals** (siblings, not re-implemented here): the ISO-TP (ISO 15765-2) transport that segments
a multi-frame PDU is `isotpkit`; the full 841-code DTC database joined to GM 17OBDG01 is `gmdtcdb`;
the Mode $06 PDF parser is `mode06kit`. This crate imports none of them. The GM-enhanced 41xx/42xx
DID *field semantics* remain catalog-only (they need an owner-authorised vehicle). SecurityAccess
key derivation is out of scope by design: `src/security.rs` models the exchange shape only.

Every `[ok]` row below names a Rust test that fails if the behaviour regresses.

| Status | Obligation | Anchor | Test |
|---|---|---|---|
| [ok] | `services::all_services` lists the UDS and GMLAN services, each flagged by standard | ISO 14229-1:2013; GMLAN rows from docs/sources.md | `src/services.rs` `catalog_has_the_core_services`, `gmlan_services_are_flagged`, `get_returns_none_for_unknown_sid` |
| [ok] | `services::positive_response_sid` adds `0x40`; `request_sid_for_response` / `is_response_sid` invert it | ISO 14229-1:2013 §7.2 | `src/services.rs` `positive_response_sid_adds_0x40` (an `0xFF` request has no response SID) |
| [ok] | `nrc::all_codes` carries the Table A.1 named codes; `describe` falls back to the reserved range | ISO 14229-1:2013 Annex A, Table A.1 | `src/nrc.rs` `table_a1_values_are_named`, `describe_falls_back_to_the_range` |
| [ok] | `nrc::is_response_pending` recognises `0x78` | ISO 14229-1:2013 Table A.1 (`0x78` RCRRP) | `src/nrc.rs` `response_pending_is_0x78` |
| [ok] | `message::encode_request` frames SID then data | ISO 14229-1:2013 §7.1 | `src/message.rs` `a_request_encodes_sid_then_data` |
| [ok] | `message::decode_response` reads a positive response and answers its request | ISO 14229-1:2013 §7.2 | `src/message.rs` `a_positive_response_decodes_and_answers` |
| [ok] | `message::decode_response` reads a `0x7F` negative response and its NRC, and rejects a malformed one | ISO 14229-1:2013 §7.5.6 | `src/message.rs` `a_negative_response_decodes_with_its_nrc` (a 2-byte negative response errors) |
| [ok] | `message::decode_response` marks a `0x78` response pending | ISO 14229-1:2013 Table A.1 | `src/message.rs` `response_pending_is_recognised` |
| [ok] | `did::read_dids` holds the 115 ReadDataByIdentifier DIDs; the write targets are separate | assessment (docs/sources.md) | `src/did.rs` `the_catalog_has_115_read_dids` |
| [ok] | `did::classify` names the identification and GM-enhanced-dense ranges by high byte | assessment | `src/did.rs` `classify_names_the_ranges` (gm-enhanced-dense == 92, identification == 3) |
| [ok] | `did::parse_read_response` splits a `0x62` response into DID and value | ISO 14229-1:2013 | `src/did.rs` `parse_read_response_splits_did_and_value` |
| [ok] | `dtc::encode` / `dtc::decode` round-trip a display code | SAE J2012 / ISO 15031-6 (**unverified**, see docs/iso-map.md) | `src/dtc.rs` `encode_decode_round_trips_a_code` (`P0A7F` <-> `0A 7F`) |
| [ok] | `dtc::category_of` and `Dtc::category` read the category from the leading letter | assessment | `src/dtc.rs` `category_comes_from_the_letter` |
| [ok] | `dtc::catalog` carries the representative subset and the per-prefix counts | assessment | `src/dtc.rs` `the_reference_subset_and_counts` (`APP_DTC_TOTAL` == 841) |
| [ok] | `dtc::decode_status` names the status bits set in a statusOfDTC / availability-mask byte | ISO 14229-1:2013 Annex D.2 (Tables D.3–D.11); 11.3.5 examples (Tables 287–289) | `src/dtc.rs` `status_byte_decodes_against_iso_examples` (statusOfDTC `0x24` -> pendingDTC + testFailedSinceLastClear; `0x2F` -> bits 0,1,2,3,5) |
| [ok] | `routine::build_request` frames the `0x31` sub-function and routine id | ISO 14229-1:2013 §13.2 | `src/routine.rs` `build_request_frames_the_routine`, `subfunction_names` |
| [ok] | `routine::catalog` names the resolved routines including the sharp edges | assessment | `src/routine.rs` `the_catalog_names_the_sharp_edges` (`0x3202` StressTest, `0xD001` SetSystemToDeliveryState, risk H) |
| [ok] | `iocontrol::build_request` frames the `0x2F` DID and parameter | ISO 14229-1:2013 §E.1 | `src/iocontrol.rs` `build_request_frames_the_signal` |
| [ok] | `iocontrol::signal_groups` include the IPSIM HMI-injection group | assessment | `src/iocontrol.rs` `signal_groups_include_ipsim_injection`, `all_signals_are_unique` |
| [ok] | `security::send_key_for` pairs each requestSeed with its sendKey | ISO 14229-1:2013 §9.4 | `src/security.rs` `requestseed_pairs_with_sendkey` |
| [ok] | `security::mechanism` describes the classic seed/key and the ADIT ALD mechanisms | assessment (docs/security-access.md) | `src/security.rs` `mechanisms_cover_classic_and_ald` (ALD is RSA-2048, NRC `0x35`) |
| [ok] | `security::build_request_seed` builds requestSeed; no key-deriving function exists | scope boundary | `src/security.rs` `only_the_exchange_shape_is_modelled` (compile-time: no seed->key fn to call) |
| [ok] | `mode06::volt_files` selects the Group 1A files for the Volt; `files_for_group` adds the generic files | SAE J1979 / assessment | `src/mode06.rs` `volt_is_group_1a`, `files_for_group_include_generic` |
| [ok] | `gmlan::response_id` / `gmlan::filter_id` apply `+0x400` / `+0x300` | GMLAN assessment (docs/sources.md) | `src/gmlan.rs` `response_and_filter_offsets` (and reject an overflow of 11 bits) |
| [ok] | `gmlan::obd_response_id` applies the ISO 15765-4 `+8` physical offset | ISO 15765-4 | `src/gmlan.rs` `obd_physical_response_offset` |
| [ok] | The public API builds a request, decodes its response, and decodes a status byte end to end | — | `tests/smoke.rs` `a_read_did_round_trips_through_the_codec`, `the_status_decoder_is_public`, `security_access_exposes_no_key_derivation` |
