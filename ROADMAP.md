# udskit

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

**The goal is to model every ISO 14229-1 service's application-message layer** — its sub-function
and parameter tables and its request/response framing, sans I/O. If the standard defines a service,
it belongs in the kit. The services not yet modelled are tracked as `[open]` rows below, each
anchored to its clause; the non-goals above are only the *sibling repos* and the SecurityAccess key
boundary, not anything ISO 14229-1 itself defines.

**Architecture — core + vendor layers.** The catalog core is pure ISO 14229-1; manufacturer layers
plug in through `vendor::VendorProfile` rather than being baked into it (GM GMLAN is the first, in the
`gm` module behind the default-on `gm` cargo feature). **All** GM-sourced reference data lives in `gm`
— the GMLAN services and `+0x400`/`+0x300` addressing, the Volt DID set, the DTC reference subset, the
GMG3 RoutineControl / IOControl / SecurityAccess catalogs, and the Mode $06 document list — reusing the
ISO core types. `--no-default-features` gives a pure ISO 14229-1 core with no GM data linked (23 core
tests). The whole layer can lift into its own crate behind this boundary later (decided: in-crate now,
extract later).

**Catalog corrections against `primarysource`:** Authentication (0x29) is a standard UDS service from
the **2020** edition, not 2013 — it is absent from the 2013 Table 22 service list (confirmed via
`primarysource`; the 2020 edition is not loaded there, so the 2020 attribution is unverified). It is
**retagged** `ISO 14229-1:2020` in the catalog, not removed. RequestFileTransfer (0x38, §14.6) is a
2013 service and was added.

Every `[ok]` row below names a Rust test that fails if the behaviour regresses. The `[open]` rows are
ISO 14229-1 services this crate catalogs by SID but does not yet model, each anchored to its clause.

| Status | Obligation | Anchor | Test |
|---|---|---|---|
| [ok] | `services::all_services` lists the ISO 14229-1 core services, each flagged by standard (GMLAN is a vendor layer, not here) | ISO 14229-1:2013 (+ 0x29 from 2020) | `src/services.rs` `catalog_has_the_core_services`, `core_is_iso_only`, `get_returns_none_for_unknown_sid` |
| [ok] | `vendor::VendorProfile` composes vendor services and request→response/filter addressing onto the ISO core; `gm::Gm` is the first vendor layer (the 7 GMLAN services + `+0x400`/`+0x300`) | ISO 14229-1 core + GMLAN assessment (docs/sources.md) | `src/vendor.rs` `resolve_searches_core_then_vendors`; `src/gm.rs` `gm_services_are_flagged_and_addressed` |
| [ok] | `services::positive_response_sid` adds `0x40`; `request_sid_for_response` / `is_response_sid` invert it | ISO 14229-1:2013 §7.2 | `src/services.rs` `positive_response_sid_adds_0x40` (an `0xFF` request has no response SID) |
| [ok] | `nrc::all_codes` carries the Table A.1 named codes; `describe` falls back to the reserved range | ISO 14229-1:2013 Annex A, Table A.1 | `src/nrc.rs` `table_a1_values_are_named`, `describe_falls_back_to_the_range` |
| [ok] | `nrc::is_response_pending` recognises `0x78` | ISO 14229-1:2013 Table A.1 (`0x78` RCRRP) | `src/nrc.rs` `response_pending_is_0x78` |
| [ok] | `message::encode_request` frames SID then data | ISO 14229-1:2013 §7.1 | `src/message.rs` `a_request_encodes_sid_then_data` |
| [ok] | `message::decode_response` reads a positive response and answers its request | ISO 14229-1:2013 §7.2 | `src/message.rs` `a_positive_response_decodes_and_answers` |
| [ok] | `message::decode_response` reads a `0x7F` negative response and its NRC, and rejects a malformed one | ISO 14229-1:2013 §7.5.6 | `src/message.rs` `a_negative_response_decodes_with_its_nrc` (a 2-byte negative response errors) |
| [ok] | `message::decode_response` marks a `0x78` response pending | ISO 14229-1:2013 Table A.1 | `src/message.rs` `response_pending_is_recognised` |
| [ok] | `gm::dids::read_dids` holds the 115 ReadDataByIdentifier DIDs; the write targets are separate (GM layer) | assessment (docs/sources.md) | `src/gm.rs` `dids::the_catalog_has_115_read_dids` |
| [ok] | `did::classify` names the identification and GM-enhanced-dense ranges by high byte; `gm::dids::count_by_category` holds the per-category counts | assessment | `src/did.rs` `classify_names_the_ranges`; `src/gm.rs` `dids::counts_by_category` (gm-enhanced-dense == 92, identification == 3) |
| [ok] | `did::parse_read_response` splits a `0x62` response into DID and value | ISO 14229-1:2013 | `src/did.rs` `parse_read_response_splits_did_and_value` |
| [ok] | `dtc::encode` / `dtc::decode` round-trip a display code | SAE J2012 / ISO 15031-6 (**unverified**, see docs/iso-map.md) | `src/dtc.rs` `encode_decode_round_trips_a_code` (`P0A7F` <-> `0A 7F`) |
| [ok] | `dtc::category_of` and `Dtc::category` read the category from the leading letter | assessment | `src/dtc.rs` `category_comes_from_the_letter` |
| [ok] | `gm::dtc::reference` / `gm::dtc::get` carry the representative subset and the per-prefix counts (GM layer) | assessment | `src/gm.rs` `dtc::the_reference_subset_and_counts` (`APP_DTC_TOTAL` == 841) |
| [ok] | `dtc::decode_status` names the status bits set in a statusOfDTC / availability-mask byte | ISO 14229-1:2013 Annex D.2 (Tables D.3–D.11); 11.3.5 examples (Tables 287–289) | `src/dtc.rs` `status_byte_decodes_against_iso_examples` (statusOfDTC `0x24` -> pendingDTC + testFailedSinceLastClear; `0x2F` -> bits 0,1,2,3,5) |
| [ok] | `routine::build_request` frames the `0x31` sub-function and routine id | ISO 14229-1:2013 §13.2 | `src/routine.rs` `build_request_frames_the_routine`, `subfunction_names` |
| [ok] | `gm::routines::catalog` names the resolved routines including the sharp edges (GM layer) | assessment | `src/gm.rs` `routines::the_catalog_names_the_sharp_edges` (`0x3202` StressTest, `0xD001` SetSystemToDeliveryState, risk H) |
| [ok] | `iocontrol::build_request` frames the `0x2F` DID and parameter | ISO 14229-1:2013 §E.1 | `src/iocontrol.rs` `build_request_frames_the_signal` |
| [ok] | `gm::signals::groups` include the IPSIM HMI-injection group (GM layer) | assessment | `src/gm.rs` `signals::signal_groups_include_ipsim_injection`, `signals::all_signals_are_unique` |
| [ok] | `security::send_key_for` pairs each requestSeed with its sendKey | ISO 14229-1:2013 §9.4 | `src/security.rs` `requestseed_pairs_with_sendkey` |
| [ok] | `gm::security::mechanism` describes the classic seed/key and the ADIT ALD mechanisms (GM layer) | assessment (docs/security-access.md) | `src/gm.rs` `security::mechanisms_cover_classic_and_ald` (ALD is RSA-2048, NRC `0x35`) |
| [ok] | `security::build_request_seed` builds requestSeed; no key-deriving function exists | scope boundary | `src/security.rs` `only_the_exchange_shape_is_modelled` (compile-time: no seed->key fn to call) |
| [ok] | `gm::mode06::volt_files` selects the Group 1A files for the Volt; `files_for_group` adds the generic files (GM layer) | SAE J1979 / assessment | `src/gm.rs` `mode06::volt_is_group_1a`, `mode06::files_for_group_include_generic` |
| [ok] | `gmlan::response_id` / `gmlan::filter_id` apply `+0x400` / `+0x300` | GMLAN assessment (docs/sources.md) | `src/gmlan.rs` `response_and_filter_offsets` (and reject an overflow of 11 bits) |
| [ok] | `gmlan::obd_response_id` applies the ISO 15765-4 `+8` physical offset | ISO 15765-4 | `src/gmlan.rs` `obd_physical_response_offset` |
| [ok] | The public API builds a request, decodes its response, and decodes a status byte end to end | — | `tests/smoke.rs` `a_read_did_round_trips_through_the_codec`, `the_status_decoder_is_public`, `security_access_exposes_no_key_derivation` |
| [ok] | The read-only CLI prints each catalog, decodes a PDU and status byte from hex, and exits non-zero on bad usage | — | `tests/cli.rs` `catalog_commands_print_expected`, `decode_and_gmlan_render`, `bad_usage_exits_nonzero` (drives the built binary) |
| [ok] | DiagnosticSessionControl (0x10): a `diagnosticSessionType` catalog (default / programming / extended / safetySystem) and a request builder | ISO 14229-1:2013 §9.2, Table 25 | `src/session.rs` `build_request_frames_the_session_type`, `session_types_match_table_25`; `tests/cli.rs` `session_command_lists_and_builds` |
| [ok] | ECUReset (0x11): a `resetType` catalog (hardReset / keyOffOnReset / softReset / enableRapidPowerShutDown / disableRapidPowerShutDown) and a request builder | ISO 14229-1:2013 §9.3, Table 34 | `src/ecureset.rs` `build_request_frames_the_reset_type`, `reset_types_match_table_34`; `tests/cli.rs` `ecureset_command_lists_and_builds` |
| [ok] | TesterPresent (0x3E): the zeroSubFunction request builder | ISO 14229-1:2013 §9.6, Table 66 | `src/testerpresent.rs` `build_request_is_the_zero_sub_function`; `tests/cli.rs` `testerpresent_command_builds` |
| [ok] | CommunicationControl (0x28): a `controlType` catalog and the `communicationType` byte, with a request builder | ISO 14229-1:2013 §9.5, Table 54; Annex B.1 | `src/commcontrol.rs` `build_request_frames_control_and_communication_type`, `control_types_match_table_54`, `communication_type_byte_follows_annex_b1`; `tests/cli.rs` `commcontrol_command_lists_and_builds` |
| [ok] | ControlDTCSetting (0x85): the `DTCSettingType` on/off sub-functions and a request builder | ISO 14229-1:2013 §9.9, Table 87 | `src/dtcsetting.rs` `build_request_frames_the_setting_type`, `setting_types_match_table_87`; `tests/cli.rs` `dtcsetting_command_lists_and_builds` |
| [ok] | ReadDTCInformation (0x19): the `reportType` sub-function catalog and per-report request builders (pairs with the existing Annex D.2 status-byte decoder) | ISO 14229-1:2013 §11.3, Table 269 and request Tables 256–268 | `src/readdtc.rs` `report_types_match_table_269`, `builders_frame_each_request_shape`; `tests/cli.rs` `readdtc_command_lists_and_builds` |
| [ok] | The `suppressPosRspMsgIndicationBit` (bit 7 of a sub-function byte): set on encode, strip on decode, for sub-function services | ISO 14229-1:2013 §8.2.2, Tables 11/14; §8.3.2 | `src/subfunction.rs` `suppress_bit_sets_strips_and_tests` |
| [ok] | ReadDataByIdentifier (0x22): a typed request builder for one or more DIDs (the `0x62` response parse already exists) | ISO 14229-1:2013 §10.2, Table 142 | `src/did.rs` `build_read_request_frames_dids`; `tests/cli.rs` `rdbi_wdbi_commands_build` |
| [ok] | WriteDataByIdentifier (0x2E): a request builder over the write-DID catalog | ISO 14229-1:2013 §10.7, Table 231 | `src/did.rs` `build_write_request_frames_did_and_data`; `tests/cli.rs` `rdbi_wdbi_commands_build` |
| [ok] | ClearDiagnosticInformation (0x14): a request builder over a groupOfDTC | ISO 14229-1:2013 §11.2, Table 250; §11.2.5 (emissions group) | `src/dtc.rs` `build_clear_request_frames_the_group`; `tests/cli.rs` `cleardtc_command_builds` |
| [ok] | ReadMemoryByAddress (0x23): request builder with the addressAndLengthFormatIdentifier | ISO 14229-1:2013 §10.3, Table 151/152 (Annex H) | `src/memory.rs` `alfid_composes_and_decodes`, `build_read_request_frames_address_and_size`; `tests/cli.rs` `readmem_writemem_commands_build` |
| [ok] | ReadScalingDataByIdentifier (0x24): request builder and the scalingByte model | ISO 14229-1:2013 §10.4 (request); Annex C.2/C.3 (scalingByte, response-side) | `src/did.rs` `build_scaling_request_frames_the_did`; `tests/cli.rs` `rsdbi_periodic_commands_build` |
| [ok] | ReadDataByPeriodicIdentifier (0x2A): transmissionMode + periodic DID request builder | ISO 14229-1:2013 §10.5, Table 174; Annex C.4 Table C.10 | `src/periodic.rs` `transmission_modes_match_table_c10`, `build_request_frames_mode_and_ids`; `tests/cli.rs` `rsdbi_periodic_commands_build` |
| [ok] | DynamicallyDefineDataIdentifier (0x2C): defineByIdentifier / defineByMemoryAddress / clear builders | ISO 14229-1:2013 §10.6, Tables 190/191/192 | `src/dynamicdid.rs` `definition_types_match_the_tables`, `build_define_by_identifier_frames_sources`, `build_define_by_memory_address_frames_alfid_and_locations`, `build_clear_frames_optional_dddi`; `tests/cli.rs` `dddi_command_lists_and_clears` |
| [ok] | WriteMemoryByAddress (0x3D): request builder with the addressAndLengthFormatIdentifier | ISO 14229-1:2013 §10.8, Table 238/239 (Annex H) | `src/memory.rs` `build_write_request_frames_address_size_and_data`; `tests/cli.rs` `readmem_writemem_commands_build` |
| [ok] | AccessTimingParameter (0x83): the four timing-parameter access modes and a builder | ISO 14229-1:2013 §9.7, Tables 73/74 | `src/accesstiming.rs` `access_types_match_table_74`, `build_request_frames_access_type_and_record`; `tests/cli.rs` `accesstiming_command_lists_and_builds` |
| [open] | SecuredDataTransmission (0x84): the secured-message envelope framing | ISO 14229-1:2013 §9.8 | — |
| [open] | ResponseOnEvent (0x86): eventType catalog and the setup request framing | ISO 14229-1:2013 §9.10; Annex B.2 | — |
| [ok] | LinkControl (0x87): linkControlType + linkControlModeIdentifier catalog and a builder | ISO 14229-1:2013 §9.11, Tables 124/125/126/127; Annex B.3 Table B.3 | `src/linkcontrol.rs` `catalogs_match_the_tables`, `builders_frame_each_request`; `tests/cli.rs` `linkcontrol_command_lists_and_builds` |
| [open] | Upload/Download unit (0x34–0x38): RequestDownload / RequestUpload / TransferData / RequestTransferExit / RequestFileTransfer request framing (sans I/O; no flashing — the flow is `spskit`) | ISO 14229-1:2013 §14; Annex G | — |
| [open] | Authentication (0x29): the sub-functions and request builders | ISO 14229-1:**2020** (edition not in `primarysource`; model when loaded) | — |
| [ok] | The GM-sourced reference data (routine, iocontrol, SecurityAccess mechanisms, Volt DID set, Mode $06 docs) lives in the `gm` layer behind the default `gm` cargo feature; `--no-default-features` is a pure ISO core | architecture (GMLAN assessment, docs/sources.md) | `src/gm.rs` `dids::*`, `dtc::*`, `routines::*`, `signals::*`, `security::*`, `mode06::*`; `tests/smoke.rs` `gm_vendor_layer_is_reachable` (and the no-default-features lib build) |
