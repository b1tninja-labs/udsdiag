# ISO / standard map

Every protocol claim this crate makes is anchored to a standard read through the
`primarysource` MCP tools, never stated from memory (a `D:\code\AGENTS.md` rule). This file is
the index: which clause backs which module, and which claims have **no** primary source loaded
and are therefore marked unverified in the code.

## Confirmed against a loaded source

| Claim | Module | Standard | Where |
|---|---|---|---|
| Negative-response-code values and names (0x10 generalReject … 0x78 responsePending) | `src/nrc.rs` | ISO 14229-1:2013 | Annex A (normative), Table A.1 — `iso:14229-1@2013§15.4.4` (Table A.1 body) |
| Positive response = request SID + 0x40; negative response = `0x7F <SID> <NRC>` | `src/services.rs`, `src/message.rs` | ISO 14229-1:2013 | 7.2, 7.5.6 |
| Service-identifier catalog membership (which SIDs the 2013 edition defines) | `src/services.rs` | ISO 14229-1:2013 | §9.1 Table 22 (Diag. & Comm. Mgmt list) and Contents §9–§14. Correction: Authentication (0x29) is absent from the 2013 Table 22 — it is retagged `ISO 14229-1:2020` (see the unverified table); RequestFileTransfer (0x38, §14.6) was added. |
| RoutineControl sub-functions: 0x01 startRoutine (M), 0x02 stopRoutine (U), 0x03 requestRoutineResults | `src/routine.rs` | ISO 14229-1:2013 | 13.2; request example tables confirm the `0x31 / 0x01-0x03` bytes |
| DiagnosticSessionControl diagnosticSessionType: 0x01 defaultSession, 0x02 programmingSession, 0x03 extendedDiagnosticSession, 0x04 safetySystemDiagnosticSession; 0x40–0x5F VMS, 0x60–0x7E SSS; request has no data parameters | `src/session.rs` | ISO 14229-1:2013 | §9.2, Table 25 (read in full); 9.2.2.3 (no request data) |
| ECUReset resetType: 0x01 hardReset, 0x02 keyOffOnReset, 0x03 softReset, 0x04 enableRapidPowerShutDown, 0x05 disableRapidPowerShutDown; 0x40–0x5F VMS, 0x60–0x7E SSS; request has no data parameters | `src/ecureset.rs` | ISO 14229-1:2013 | §9.3, Table 34 (read in full); 9.3.2.3 (no request data) |
| TesterPresent: the only sub-function is 0x00 zeroSubFunction (0x01–0x7F reserved); request has no data parameters | `src/testerpresent.rs` | ISO 14229-1:2013 | §9.6, Table 66 (read in full); 9.6.2.3 (no request data) |
| CommunicationControl controlType: 0x00 enableRxAndTx, 0x01 enableRxAndDisableTx, 0x02 disableRxAndEnableTx, 0x03 disableRxAndTx, 0x04/0x05 …WithEnhancedAddressInformation (require nodeIdentificationNumber); communicationType byte = subnet (bits 7-4) + message type (bits 1-0: 0x1 NCM, 0x2 NWMCM, 0x3 both) | `src/commcontrol.rs` | ISO 14229-1:2013 | §9.5, Tables 53/54 (read in full); Annex B.1 Table B.1 |
| ControlDTCSetting DTCSettingType: 0x01 on, 0x02 off; 0x40–0x5F VMS, 0x60–0x7E SSS; request is 0x85 <type> [DTCSettingControlOptionRecord] (option record optional, manufacturer-specific) | `src/dtcsetting.rs` | ISO 14229-1:2013 | §9.9, Tables 86/87 (read in full) |
| ReadDataByIdentifier request = 0x22 + one or more 2-byte DIDs; WriteDataByIdentifier request = 0x2E + 2-byte DID + dataRecord | `src/did.rs` (`build_read_request`, `build_write_request`) | ISO 14229-1:2013 | §10.2 Table 142; §10.7 Table 231 |
| ClearDiagnosticInformation request = 0x14 + 3-byte groupOfDTC; groups: 0xFFFF33 Emissions, 0xFFFFFF All Groups | `src/dtc.rs` (`build_clear_request`, `ALL_GROUPS_OF_DTC`, `EMISSIONS_RELATED_SYSTEMS_GROUP`) | ISO 14229-1:2013 | §11.2 Table 250; Annex D.1 Table D.1 (0xFFFFFF All Groups, 0xFFFFxx FunctionalGroupIdentifier incl. 0xFFFF33 Emissions) |
| ReadScalingDataByIdentifier request = 0x24 + one 2-byte DID | `src/did.rs` (`build_scaling_request`) | ISO 14229-1:2013 | §10.4 (request; scalingByte response-side is Annex C.2/C.3, not modelled) |
| ReadDataByPeriodicIdentifier request = 0x2A + transmissionMode + 1-byte periodicDataIdentifiers; transmissionMode 0x01 slow, 0x02 medium, 0x03 fast, 0x04 stopSending | `src/periodic.rs` | ISO 14229-1:2013 | §10.5 Table 174; Annex C.4 Table C.10 (read in full) |
| ReadDTCInformation reportType catalog (27 values 0x01–0x19, 0x42, 0x55) and request shapes (status-mask, by-DTC-number+record, no-params, etc.) | `src/readdtc.rs` | ISO 14229-1:2013 | §11.3 Table 269 (sub-function defs) and request Tables 256–268 (read in full) |
| SecuredDataTransmission request = 0x84 + securityDataRequestRecord (opaque Security Sub-Layer output; this crate frames the envelope only, no securing) | `src/securedtransmission.rs` | ISO 14229-1:2013 | §9.8 Tables 81/82 |
| Upload/Download unit: RequestDownload/Upload (0x34/0x35) = SID + dataFormatIdentifier (compression<<4 \| encryption) + ALFID + memoryAddress + memorySize; TransferData (0x36) = SID + blockSequenceCounter + record; RequestTransferExit (0x37) = SID + record; RequestFileTransfer (0x38) = SID + modeOfOperation (0x01 AddFile..0x05 ReadDir) + filePathAndNameLength(2) + filePathAndName + mode-dependent trailer | `src/transfer.rs` | ISO 14229-1:2013 | §14 Tables 393/433; Annex G (modeOfOperation), Annex H (ALFID) |
| LinkControl linkControlType: 0x01 verifyModeTransitionWithFixedParameter (+ linkControlModeIdentifier), 0x02 verifyModeTransitionWithSpecificParameter (+ 3-byte linkRecord baud rate), 0x03 transitionMode; linkControlModeIdentifier (Annex B.3): PC9600..PC115200 (0x01-0x05), CAN125000..CAN1000000 (0x10-0x13), ProgrammingSetup (0x20) | `src/linkcontrol.rs` | ISO 14229-1:2013 | §9.11 Tables 124/125/126/127; Annex B.3 Table B.3 |
| DynamicallyDefineDataIdentifier definitionType: 0x01 defineByIdentifier (DDDI + sourceDID/position/memorySize tuples), 0x02 defineByMemoryAddress (DDDI + ALFID + address/size pairs), 0x03 clearDynamicallyDefinedDataIdentifier (optional DDDI = clear all) | `src/dynamicdid.rs` | ISO 14229-1:2013 | §10.6 Tables 190/191/192 (read in full) |
| AccessTimingParameter timingParameterAccessType: 0x01 readExtendedTimingParameterSet, 0x02 setTimingParametersToDefaultValues, 0x03 readCurrentlyActiveTimingParameters, 0x04 setTimingParametersToGivenValues (carries the data-link-specific TimingParameterRequestRecord) | `src/accesstiming.rs` | ISO 14229-1:2013 | §9.7 Tables 73/74 (read in full) |
| ReadMemoryByAddress = 0x23 + ALFID + memoryAddress + memorySize; WriteMemoryByAddress = 0x3D + ALFID + memoryAddress + memorySize + dataRecord; ALFID high nibble = memorySize length, low nibble = memoryAddress length | `src/memory.rs` | ISO 14229-1:2013 | §10.3 Tables 151/152; §10.8 Tables 238/239; Annex H |
| suppressPosRspMsgIndicationBit = bit 7 of the sub-function byte (1 = suppress positive response); bits 6-0 = sub-function value; positive response echoes bits 6-0 with bit 7 cleared | `src/subfunction.rs` | ISO 14229-1:2013 | §8.2.2 Tables 11/14 (read in full); §8.3.2 |
| SecurityAccess pairing: requestSeed odd ↔ sendKey = requestSeed + 1; NRC 0x35 invalidKey | `src/security.rs` | ISO 14229-1:2013 | 9.4.2; "requestSeed = 0x01 … sendKey = 0x02" |
| requestOutOfRange 0x31, securityAccessDenied 0x33, invalidKey 0x35 definitions | `src/nrc.rs` | ISO 14229-1:2013 | Table A.1 (read in full) |
| DTC status byte: bit 0 testFailed … bit 7 warningIndicatorRequested, with M/U availability | `src/dtc.rs` (`STATUS_BITS`, `decode_status`) | ISO 14229-1:2013 | Annex D.2, Tables D.3–D.11 (DTC status bit definitions); layout confirmed against the 0x19 examples in 11.3.5 (Tables 287–289, statusOfDTC 0x24 / 0x2F) |
| ISO-TP carries multi-frame UDS PDUs (segmentation out of scope here) | `src/message.rs` docs | ISO 15765-2:2016 | referenced; the codec is the sibling `isotpkit` |
| Mode $06 / Mode $09 are the generic OBD-II services | `src/mode06.rs` | SAE J1979 / ISO 15031-5 | named; GM's own PDFs are the sibling `mode06kit`'s source |

The ISO 14229-1 service SID values (0x10, 0x11, 0x14, 0x19, 0x22, 0x23, 0x24, 0x27, 0x28,
0x29, 0x2A, 0x2C, 0x2E, 0x2F, 0x31, 0x34–0x37, 0x3D, 0x3E, 0x83–0x87) are the service
identifiers defined per clause in ISO 14229-1:2013; the ones this assessment exercised
(`0x22`, `0x2E`, `0x27`, `0x2F`, `0x31`, `0x19`, `0x14`, `0x10`, `0x11`) are additionally
corroborated against the GMG3 firmware dispatch tree and the app's captured frames
(`docs/sources.md`).

## Not in `primarysource` — marked unverified in code

| Claim | Module | Would-be source | Status |
|---|---|---|---|
| Two-byte DTC packing: top-2 bits of byte 1 → P/C/B/U; nibbles → digits | `src/dtc.rs` (`encode` / `decode`) | SAE J2012 / ISO 15031-6 | **Unverified.** Implements the conventional format; the module docs and `encode`/`decode` say so. Neither J2012 nor ISO 15031-6 is loaded (`iso_docs` lists 15031-3 only). |
| GMLAN `+0x400` response / `+0x300` filter 11-bit addressing | `src/gmlan.rs` | GM GMW3110 (GMLAN) | GM-proprietary convention, taken from the app capture in `docs/sources.md`, not a public standard. The OBD-II `0x7E0+8` physical response offset is ISO 15765-4. |
| GMLAN service aliases ($1A, $3B, $AE, …) | `src/gm.rs` (`gm::Gm` vendor layer) | GM GMW3110 (GMLAN) | GM-specific; sourced from the firmware/app assessment, flagged `Standard::Gmlan`. Not in the ISO core — exposed through `vendor::VendorProfile`. |
| Authentication (0x29) belongs to ISO 14229-1:**2020** | `src/services.rs` (`Standard::Iso14229Later`) | ISO 14229-1:2020 | **Unverified.** Confirmed only that 0x29 is *absent* from the 2013 Table 22; the 2020 edition is not loaded in `primarysource` (`iso_docs` lists `14229-1@2013` only), so the 2020 attribution rests on general knowledge, not a loaded clause. Flagged `Iso14229Later` and noted in the code. Load the 2020 edition to confirm and cite. |

If any of these documents is later fetched into `primarysource`, read it and replace the
"unverified" note with a clause citation here.
