# Obligations and their evidence

Every obligation `udsdiag` has taken on, with an honest status against the code.

**The honesty rule:** a row reads `[ok]` only when the test named in its Evidence cell exists
and would fail if the behaviour regressed. `tests/test_roadmap_rows.py` resolves every `[ok]`
row's test and every callable the matrix names.

Legend: `[ok]` implemented and pinned by a named test — `[part]` partial, what is missing
stated — `[ ]` not built — `[no]` deliberate non-goal, with the reason.

Protocol wording is read through `primarysource` and mapped in `docs/iso-map.md`. Catalog data
traces to the read-only `volt` assessment documents named in `docs/sources.md`.

## 1. Matrix

### Service and negative-response-code catalogs (ISO 14229-1:2013)

| Id | Obligation | Spec | Status | Evidence |
|---|---|---|---|---|
| `ud.svc.catalog` | `services.all_services()` lists the UDS and GMLAN services | ISO 14229-1:2013 | `[ok]` | `tests/test_services.py::test_catalog_has_the_core_services` |
| `ud.svc.response` | `services.positive_response_sid()` adds 0x40 to the request SID | ISO 14229-1:2013 7.2 | `[ok]` | `tests/test_services.py::test_positive_response_sid_adds_0x40` |
| `ud.svc.unknown` | `services.get()` returns None for a SID outside the catalog | ISO 14229-1:2013 | `[ok]` | `tests/test_services.py::test_get_returns_none_for_unknown_sid` |
| `ud.svc.gmlan` | `services.get()` flags the GMLAN aliases as GM-specific | GMLAN assessment | `[ok]` | `tests/test_services.py::test_gmlan_services_are_flagged` |
| `ud.nrc.named` | `nrc.all_codes()` carries the Table A.1 named codes | ISO 14229-1:2013 A.1 | `[ok]` | `tests/test_nrc.py::test_table_a1_values_are_named` |
| `ud.nrc.range` | `nrc.describe()` falls back to the reserved range for an unnamed value | ISO 14229-1:2013 A.1 | `[ok]` | `tests/test_nrc.py::test_describe_falls_back_to_the_range` |
| `ud.nrc.pending` | `nrc.is_response_pending()` recognises 0x78 | ISO 14229-1:2013 A.1 | `[ok]` | `tests/test_nrc.py::test_response_pending_is_0x78` |

### Request/response codec

| Id | Obligation | Spec | Status | Evidence |
|---|---|---|---|---|
| `ud.msg.request` | `message.encode_request()` frames SID then data | ISO 14229-1:2013 7.1 | `[ok]` | `tests/test_message.py::test_a_request_encodes_sid_then_data` |
| `ud.msg.positive` | `message.decode_response()` reads a positive response and answers its request | ISO 14229-1:2013 7.2 | `[ok]` | `tests/test_message.py::test_a_positive_response_decodes_and_answers` |
| `ud.msg.negative` | `message.decode_response()` reads a 0x7F negative response and its NRC | ISO 14229-1:2013 7.5.6 | `[ok]` | `tests/test_message.py::test_a_negative_response_decodes_with_its_nrc` |
| `ud.msg.pending` | `message.decode_response()` marks a 0x78 response pending | ISO 14229-1:2013 A.1 | `[ok]` | `tests/test_message.py::test_response_pending_is_recognised` |

### DID catalog (Voltage 2.3.0)

| Id | Obligation | Spec | Status | Evidence |
|---|---|---|---|---|
| `ud.did.catalog` | `did.read_dids()` holds the 115 ReadDataByIdentifier DIDs | assessment | `[ok]` | `tests/test_did.py::test_the_catalog_has_115_read_dids` |
| `ud.did.classify` | `did.classify()` names the identification and GM-enhanced ranges | assessment | `[ok]` | `tests/test_did.py::test_classify_names_the_identification_range` |
| `ud.did.parse` | `did.parse_read_response()` splits a 0x62 response into DID and value | ISO 14229-1:2013 | `[ok]` | `tests/test_did.py::test_parse_read_response_splits_did_and_value` |

### DTC model and reference

| Id | Obligation | Spec | Status | Evidence |
|---|---|---|---|---|
| `ud.dtc.codec` | `dtc.encode()` and `dtc.decode()` round-trip a display code | SAE J2012 / ISO 15031-6 (unverified) | `[ok]` | `tests/test_dtc.py::test_encode_decode_round_trips_a_code` |
| `ud.dtc.category` | `dtc.category_of()` reads the category from the leading letter | assessment | `[ok]` | `tests/test_dtc.py::test_category_comes_from_the_letter` |
| `ud.dtc.reference` | `dtc.catalog()` carries the representative subset and the per-prefix counts | assessment | `[ok]` | `tests/test_dtc.py::test_the_reference_subset_and_counts` |
| `ud.dtc.status` | `dtc.decode_status()` names the status bits set in a statusOfDTC / availability-mask byte | ISO 14229-1:2013 D.2 | `[ok]` | `tests/test_dtc.py::test_status_byte_decodes_against_iso_examples` |

### RoutineControl and IOControl

| Id | Obligation | Spec | Status | Evidence |
|---|---|---|---|---|
| `ud.rc.build` | `routine.build_request()` frames 0x31 sub-function and routine id | ISO 14229-1:2013 13.2 | `[ok]` | `tests/test_routine.py::test_build_request_frames_the_routine` |
| `ud.rc.catalog` | `routine.catalog()` names the resolved routines incl. the sharp edges | assessment | `[ok]` | `tests/test_routine.py::test_the_catalog_names_the_sharp_edges` |
| `ud.rc.subfn` | `routine.subfunction_name()` names start/stop/results | ISO 14229-1:2013 13.2 | `[ok]` | `tests/test_routine.py::test_subfunction_names` |
| `ud.io.build` | `iocontrol.build_request()` frames 0x2F DID and parameter | ISO 14229-1:2013 E.1 | `[ok]` | `tests/test_iocontrol.py::test_build_request_frames_the_signal` |
| `ud.io.signals` | `iocontrol.signal_groups()` includes the IPSIM HMI-injection group | assessment | `[ok]` | `tests/test_iocontrol.py::test_signal_groups_include_ipsim_injection` |

### SecurityAccess (exchange shape only)

| Id | Obligation | Spec | Status | Evidence |
|---|---|---|---|---|
| `ud.sa.pairing` | `security.send_key_for()` pairs each requestSeed with its sendKey | ISO 14229-1:2013 9.4 | `[ok]` | `tests/test_security.py::test_requestseed_pairs_with_sendkey` |
| `ud.sa.mechanisms` | `security.mechanism()` describes classic seed/key and the ADIT ALD | assessment | `[ok]` | `tests/test_security.py::test_mechanisms_cover_classic_and_ald` |
| `ud.sa.nokeys` | `security.build_request_seed()` builds requestSeed; no key-deriving builder exists | scope boundary | `[ok]` | `tests/test_security.py::test_no_key_derivation_is_exposed` |

### Mode $06 catalog and GMLAN addressing

| Id | Obligation | Spec | Status | Evidence |
|---|---|---|---|---|
| `ud.m6.volt` | `mode06.volt_files()` selects the Group 1A files for the Volt | SAE J1979 / assessment | `[ok]` | `tests/test_mode06.py::test_volt_is_group_1a` |
| `ud.m6.generic` | `mode06.files_for_group()` adds the generic files to a group | assessment | `[ok]` | `tests/test_mode06.py::test_files_for_group_include_generic` |
| `ud.gm.offsets` | `gmlan.response_id()` and `gmlan.filter_id()` apply +0x400 / +0x300 | GMLAN assessment | `[ok]` | `tests/test_gmlan.py::test_response_and_filter_offsets` |
| `ud.gm.obd` | `gmlan.obd_response_id()` applies the ISO 15765-4 +8 physical offset | ISO 15765-4 | `[ok]` | `tests/test_gmlan.py::test_obd_physical_response_offset` |

### CLI

| Id | Obligation | Spec | Status | Evidence |
|---|---|---|---|---|
| `ud.cli.list` | `cli.main()` prints the service catalog | — | `[ok]` | `tests/test_cli.py::test_services_command_lists_services` |
| `ud.cli.decode` | `cli.main()` decodes a response PDU from hex | — | `[ok]` | `tests/test_cli.py::test_decode_command_reads_a_negative_response` |

## 2. Planned and non-goals

| Id | Obligation | Spec | Status | Evidence |
|---|---|---|---|---|
| `ud.dtc.full` | The full 841-code DTC table joined to 17OBDG01 | assessment | `[no]` | that dataset is the sibling `gmdtcdb` |
| `ud.transport.isotp` | ISO-TP segmentation and reassembly under the PDU | ISO 15765-2:2016 | `[no]` | the transport codec is the sibling `isotpkit` |
| `ud.m6.parse` | Parse the GM Mode $06 PDFs into monitor/test-ID records | SAE J1979 | `[no]` | the parser is the sibling `mode06kit` |
| `ud.did.semantics` | Field semantics of the GM-enhanced 41xx/42xx DIDs | GM-proprietary | `[ ]` | needs an owner-authorised vehicle; catalog only for now |
