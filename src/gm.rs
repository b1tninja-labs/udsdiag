//! The GM / GMLAN vendor layer on top of the ISO 14229-1 core (feature `gm`, default-on).
//!
//! The crate core is pure ISO 14229-1. Everything GM-specific — recovered in the GM GMG3 head-unit
//! and Volt-app assessment (`docs/sources.md`) — lives here, behind the `gm` cargo feature, reusing
//! the ISO core types:
//!
//! - [`Gm`] / [`GM`]: the GMLAN services and the `+0x400` / `+0x300` addressing, as a
//!   [`VendorProfile`].
//! - [`dids`]: the Volt ReadDataByIdentifier / WriteDataByIdentifier catalog.
//! - [`dtc`]: the representative DTC reference subset and the per-prefix counts.
//! - [`routines`]: the GMG3 `procdiagnosis` RoutineControl catalog.
//! - [`signals`]: the GMG3 IOControl signal enumeration.
//! - [`security`]: the two SecurityAccess mechanism descriptors (no key material).
//! - [`mode06`]: the catalog of GM's published Mode $06 reference PDFs.
//!
//! The ISO core depends on none of this; `--no-default-features` drops the whole module. The trait
//! boundary lets the layer lift into its own crate later without touching the core.

use crate::gmlan;
use crate::services::{Service, Standard};
use crate::vendor::VendorProfile;

const fn service(sid: u8, name: &'static str, mnemonic: &'static str) -> Service {
    Service {
        sid,
        name,
        mnemonic,
        standard: Standard::Gmlan,
    }
}

/// The GMLAN services recovered in the GM GMG3 head-unit assessment (`docs/sources.md`).
pub const GM_SERVICES: &[Service] = &[
    service(0x1A, "ReadDataByIdentifier (GMLAN)", "GM-RDBI"),
    service(0x3B, "WriteDataByIdentifier (GMLAN)", "GM-WDBI"),
    service(0xA2, "ReportProgrammedState (GMLAN)", "GM-RPS"),
    service(0xA5, "ProgrammingMode (GMLAN)", "GM-PM"),
    service(0xA9, "ReadDiagnosticInformation (GMLAN)", "GM-RDI"),
    service(0xAA, "ReadDataByPacketIdentifier (GMLAN)", "GM-RDBPI"),
    service(0xAE, "DeviceControl (GMLAN)", "GM-DC"),
];

/// The GM / GMLAN vendor layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gm;

/// The GM vendor profile.
pub const GM: Gm = Gm;

impl VendorProfile for Gm {
    fn name(&self) -> &'static str {
        "GM GMLAN"
    }

    fn services(&self) -> &'static [Service] {
        GM_SERVICES
    }

    fn response_id(&self, request_id: u16) -> Option<u16> {
        gmlan::response_id(request_id).ok()
    }

    fn filter_id(&self, request_id: u16) -> Option<u16> {
        gmlan::filter_id(request_id).ok()
    }
}

#[cfg(test)]
mod gmlan_tests {
    use super::*;

    #[test]
    fn gm_services_are_flagged_and_addressed() {
        assert_eq!(GM.name(), "GM GMLAN");
        let device_control = GM.service(0xAE).expect("0xAE catalogued");
        assert_eq!(device_control.standard, Standard::Gmlan);
        assert_eq!(device_control.name, "DeviceControl (GMLAN)");
        assert_eq!(GM.services().len(), 7);
        assert_eq!(GM.response_id(0x241), Some(0x641));
        assert_eq!(GM.filter_id(0x241), Some(0x541));
        assert_eq!(GM.response_id(0x800), None); // not 11-bit
    }
}

/// The Volt Data Identifier catalog (ReadDataByIdentifier / WriteDataByIdentifier).
pub mod dids {
    use crate::did::{Did, classify};

    /// Voltage 2.3.0 — the 115 ReadDataByIdentifier (`0x22`) DIDs (`docs/sources.md`). The `0x62`
    /// demo pair 22002F -> 62002F6C confirmed 002F.
    const READ_DIDS: &[u16] = &[
        0x002F, 0x40D4, 0x4184, 0x4185, 0x4186, 0x4187, 0x4188, 0x4189, 0x4190, 0x4191, 0x4192,
        0x4196, 0x4197, 0x4198, 0x4199, 0x419A, 0x419B, 0x419C, 0x419D, 0x419E, 0x41A6, 0x41AD,
        0x41C5, 0x41C6, 0x4202, 0x4203, 0x4204, 0x4205, 0x4206, 0x4207, 0x420B, 0x420C, 0x420D,
        0x420E, 0x420F, 0x4210, 0x4211, 0x4212, 0x4213, 0x4214, 0x4215, 0x4216, 0x4217, 0x4218,
        0x4219, 0x421A, 0x421B, 0x421C, 0x4220, 0x4221, 0x4222, 0x4223, 0x4224, 0x4225, 0x4226,
        0x4227, 0x4228, 0x4229, 0x422A, 0x422B, 0x422C, 0x422D, 0x422E, 0x4232, 0x4233, 0x4234,
        0x4235, 0x4236, 0x4237, 0x4238, 0x4239, 0x423A, 0x423B, 0x423C, 0x423D, 0x423E, 0x423F,
        0x4240, 0x4241, 0x4242, 0x4243, 0x4244, 0x4245, 0x4246, 0x4247, 0x4248, 0x4249, 0x424A,
        0x424B, 0x424C, 0x424D, 0x424E, 0x424F, 0x4250, 0x4368, 0x4369, 0x436B, 0x436C, 0x43AE,
        0x43AF, 0x4470, 0x4531, 0x5005, 0x5006, 0x5007, 0x5008, 0x5247, 0x5248, 0x82A3, 0x8334,
        0x83E1, 0x83E2, 0x90B1, 0x90F8, 0x90FB,
    ];

    /// Voltage 2.3.0 — the `0x2E` WriteDataByIdentifier target DIDs (coding writes).
    const WRITE_DIDS: &[u16] = &[0xD21B, 0xD28E];

    /// The catalogued ReadDataByIdentifier (`0x22`) DIDs, classified.
    pub fn read_dids() -> Vec<Did> {
        READ_DIDS
            .iter()
            .map(|&value| Did {
                value,
                category: classify(value),
            })
            .collect()
    }

    /// The catalogued WriteDataByIdentifier (`0x2E`) target DIDs.
    pub fn write_dids() -> Vec<Did> {
        WRITE_DIDS
            .iter()
            .map(|&value| Did {
                value,
                category: classify(value),
            })
            .collect()
    }

    /// How many read DIDs fall in each category.
    pub fn count_by_category() -> std::collections::BTreeMap<&'static str, usize> {
        let mut counts = std::collections::BTreeMap::new();
        for &value in READ_DIDS {
            *counts.entry(classify(value)).or_insert(0) += 1;
        }
        counts
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn the_catalog_has_115_read_dids() {
            let dids = read_dids();
            assert_eq!(dids.len(), 115);
            let values: Vec<u16> = dids.iter().map(|d| d.value).collect();
            let unique: std::collections::BTreeSet<u16> = values.iter().copied().collect();
            assert_eq!(values.len(), unique.len());
            assert!(values.contains(&0x002F));
            assert!(values.contains(&0x90B1));
            let write: std::collections::BTreeSet<u16> =
                write_dids().iter().map(|d| d.value).collect();
            assert_eq!(write, std::collections::BTreeSet::from([0xD21B, 0xD28E]));
        }

        #[test]
        fn counts_by_category() {
            let counts = count_by_category();
            assert_eq!(counts["gm-enhanced-dense"], 92); // 22 in 41xx + 70 in 42xx
            assert_eq!(counts["identification"], 3);
        }
    }
}

/// The representative DTC reference subset and the app DTC-name counts.
pub mod dtc {
    use crate::dtc::Dtc;

    /// App DTC-name table totals, per prefix, from the Voltage 2.3.0 `classes.dex` extraction.
    pub const APP_DTC_COUNTS: &[(char, u32)] = &[('P', 309), ('B', 139), ('C', 63), ('U', 330)];
    /// The total DTC-name count across prefixes.
    pub const APP_DTC_TOTAL: u32 = 841;

    const fn dtc(
        code: &'static str,
        name: &'static str,
        in_17obdg01: bool,
        note: &'static str,
    ) -> Dtc {
        Dtc {
            code,
            name,
            in_17obdg01,
            note,
        }
    }

    // Representative cross-reference subset (docs/sources.md). in_17obdg01 marks codes GM's
    // 17OBDG01 Hybrid Diagnostics (Volt, OBD Group 1A) carries in its powertrain cert set.
    const CATALOG: &[Dtc] = &[
        dtc(
            "P0A0D",
            "High Voltage System Interlock Circuit Voltage",
            true,
            "HVIL safety interlock",
        ),
        dtc("P0A7E", "Hybrid/EV Battery Pack High Temperature", true, ""),
        dtc(
            "P0A7F",
            "Hybrid/EV Battery Pack",
            true,
            "GM scopes it to pack capability / end-of-life",
        ),
        dtc(
            "P0AE3",
            "Hybrid/EV Battery Precharge Contactor Stuck Open",
            true,
            "",
        ),
        dtc(
            "P0AF8",
            "Hybrid System Voltage",
            true,
            "pack vs TPIM/charger bus correlation > 16 V",
        ),
        dtc(
            "P0AFB",
            "Hybrid/EV Battery System Voltage",
            true,
            "pack or cell over-voltage",
        ),
        dtc(
            "P0B13",
            "Hybrid/EV Battery Current Sensor",
            true,
            "fine vs coarse current > 10 A",
        ),
        dtc(
            "P0C76",
            "Hybrid/EV Battery System High Voltage Present",
            true,
            "HV discharge time too long",
        ),
        dtc(
            "P0D5E",
            "Battery Charger System High Voltage Present",
            true,
            "",
        ),
        dtc("P0A1C", "Drive Motor Control Module Performance", true, ""),
        dtc(
            "P0A79",
            "Drive Motor Inverter Performance",
            true,
            "IGBT desaturation",
        ),
        dtc("P0C04", "Drive Motor High Current", true, ""),
        dtc(
            "P0C5C",
            "Drive Motor 1 Position Sensor",
            true,
            "resolver circuit low",
        ),
        dtc(
            "P0CF6",
            "Control Pilot Circuit",
            true,
            "J1772 pilot out-of-range high",
        ),
        dtc(
            "P0D26",
            "Battery Charger System Precharge Time Too Long",
            true,
            "",
        ),
        dtc(
            "P0D59",
            "Proximity Detection Circuit Low/High Voltage",
            true,
            "charge-cord latch",
        ),
        dtc("P1101", "Intake Air Flow System Performance", true, ""),
        dtc("P2097", "Post Catalyst Fuel Trim System", true, ""),
        dtc("P2138", "Accelerator Pedal Position (APP) Sensor", true, ""),
        dtc("U0073", "Control Module Communication Bus A Off", true, ""),
        dtc(
            "U0100",
            "Lost Communication With Engine Control Module",
            true,
            "",
        ),
        dtc(
            "U0111",
            "Lost Communication with Battery Energy Control Module",
            true,
            "",
        ),
        dtc(
            "U0140",
            "Lost Communication With Body Control Module",
            true,
            "",
        ),
        dtc(
            "U0121",
            "Lost Communication With Electronic Brake Control Module",
            false,
            "module: EBCM",
        ),
        dtc(
            "U0146",
            "Lost Communication with Serial Data Gateway Module",
            false,
            "",
        ),
        dtc(
            "U0155",
            "Lost Communication With Instrument Cluster",
            false,
            "module: IPC",
        ),
        dtc(
            "U0184",
            "Lost Communication With Radio",
            false,
            "module: Radio (head unit)",
        ),
        dtc(
            "U0256",
            "Lost Communication (faceplate)",
            false,
            "named in OBD-CONFIG.md",
        ),
        dtc("C0075", "Brake Pedal Position Sensor Performance", true, ""),
        dtc("C0591", "Brake Booster Motor", true, ""),
        dtc(
            "C10EA",
            "Powertrain Control Module Indicated Requested Torque",
            true,
            "",
        ),
    ];

    /// The representative DTC reference subset.
    pub const fn reference() -> &'static [Dtc] {
        CATALOG
    }

    /// The reference entry for a display code, or `None` if not in the subset.
    pub fn get(code: &str) -> Option<Dtc> {
        let upper = code.to_ascii_uppercase();
        CATALOG.iter().copied().find(|entry| entry.code == upper)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn the_reference_subset_and_counts() {
            assert_eq!(APP_DTC_TOTAL, 841);
            assert_eq!(APP_DTC_COUNTS.iter().map(|&(_, n)| n).sum::<u32>(), 841);
            assert!(!get("U0184").unwrap().in_17obdg01);
            assert!(get("P0A0D").unwrap().in_17obdg01);
            assert_eq!(get("P0A7F").unwrap().category(), "powertrain");
            assert!(get("ZZZZZ").is_none());
            assert!(reference().len() >= 20);
        }
    }
}

/// The GMG3 `procdiagnosis` RoutineControl catalog.
pub mod routines {
    use crate::routine::Routine;

    const fn routine(
        id: u16,
        name: &'static str,
        effect: &'static str,
        risk: &'static str,
    ) -> Routine {
        Routine {
            routine_id: id,
            name,
            effect,
            risk,
        }
    }

    // GMG3 head-unit procdiagnosis routines (docs/sources.md).
    const CATALOG: &[Routine] = &[
        routine(
            0x3202,
            "StressTest",
            "fork()+execv(/usr/bin/stress, argv) with attacker-chosen args (DoS)",
            "H",
        ),
        routine(
            0xD001,
            "SetSystemToDeliveryState",
            "factory / shipping-state reset (de-personalise the unit)",
            "H",
        ),
        routine(0x3200, "RunIn", "end-of-line burn-in start", "M"),
        routine(0x3201, "RunInClearResults", "clear burn-in results", "M"),
        routine(
            0x3069,
            "GNSSCalcAndVerifyChecksum",
            "GNSS firmware CRC verify",
            "L",
        ),
        routine(0x3040, "IPODSelfTest", "iPod / USB self-test", "L"),
        routine(
            0x3204,
            "DisplayTestScreenSlideshow",
            "pushes display test patterns",
            "L",
        ),
    ];

    /// The routine catalog.
    pub const fn catalog() -> &'static [Routine] {
        CATALOG
    }

    /// The routine for an on-wire ID, or `None` when it is not catalogued.
    pub fn get(routine_id: u16) -> Option<Routine> {
        CATALOG.iter().copied().find(|r| r.routine_id == routine_id)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn the_catalog_names_the_sharp_edges() {
            let stress = get(0x3202).expect("0x3202 catalogued");
            assert_eq!(stress.name, "StressTest");
            assert_eq!(stress.risk, "H");
            assert_eq!(get(0xD001).unwrap().name, "SetSystemToDeliveryState");
            assert!(get(0x0000).is_none());
            for r in catalog() {
                assert!(["H", "M", "L"].contains(&r.risk));
            }
        }
    }
}

/// The GMG3 IOControl signal enumeration, grouped by what each signal touches.
pub mod signals {
    /// `DIA_EN_IOCTRL_SIGTYPE_*` signal enumeration, grouped (GMG3 firmware; `docs/sources.md`).
    pub const SIGNAL_GROUPS: &[(&str, &[&str])] = &[
        (
            "hmi-input-injection",
            &[
                "IPSIM_PRESSED_KEY",
                "IPSIM_KEY_CODE_SIMULATION",
                "IPSIM_CONTROL_KNOB_SIMULATION",
                "IPSIM_VOLUME_SIMULATION",
                "IPSIM_LOCK_INFO_CONTROL",
                "PRESSED_KEY",
            ],
        ),
        (
            "speaker-audio",
            &[
                "ACTUATOR_FRONT_LEFT",
                "ACTUATOR_FRONT_RIGHT",
                "ACTUATOR_REAR_LEFT",
                "ACTUATOR_REAR_RIGHT",
                "AUDIO_TESTTONE",
                "AUDIO_VOLUME",
                "AUDIO_SOURCE",
                "AUDIO_BALANCE",
                "AUDIO_FADER",
                "AUDIO_LINEARMODE",
                "AUDIO_EXT_AMPL_CONTLINE",
                "AUDIO_SELECT_COMPRESSED_AUDIO_TITLE",
            ],
        ),
        (
            "microphone-power",
            &["MICROPHONE_POWER_SUPPLY", "MICROPHONE_POWER_SUPPLY_2"],
        ),
        (
            "camera",
            &["RVC_CONTROLLINE", "ACTIVE_VIDEO_SOURCE", "VIDEO_SOURCE"],
        ),
        (
            "display",
            &[
                "DISPLAY_TEST",
                "DISPLAY_TEST_PATTERN",
                "DISPLAY_BRIGHTNESS",
                "KEY_ILLUMINATION_PWM",
                "KEYILLUMINATION_BRIGHTNESS",
            ],
        ),
        (
            "thermal-power",
            &[
                "FAN",
                "USB_POWER_SUPPLY_1",
                "USB_POWER_SUPPLY_2",
                "USB_POWER_SUPPLY_3",
            ],
        ),
        (
            "bluetooth",
            &[
                "BLUETOOTH_MODULE_RESET",
                "BLUETOOTH_SEARCH_DEVICES",
                "BLUETOOTH_PAGING",
                "BLUETOOTH_MODE",
            ],
        ),
        (
            "optical-drive",
            &[
                "CDROM_EJECT",
                "CD_RESET_LINE",
                "CDDRIVE_TEST_QUICK",
                "CD_MODE_CONTROL",
                "PLAYED_TRACK",
            ],
        ),
        (
            "tuner-rf",
            &[
                "TUNERAMFM_ANT1_POWER_SUPPLY",
                "TUNERAMFM_ANT2_POWER_SUPPLY",
                "TUNERDAB_ANT1_POWER_SUPPLY",
                "TUNERDAB_ANT1_ENSEMBLE_FREQUENCY",
                "TUNERDAB_ANT1_SERVICE_ID",
                "TUNERXM_1_C",
                "TUNER_SEL_ACTIVE_TUNER_AMFM",
                "TUNER_SET_BAND_AMFM",
                "TUNER_SET_FREQUENCY_AMFM",
            ],
        ),
        (
            "system-misc",
            &[
                "SYSTEM_HMI_ON",
                "SYSTEM_REMOTE_CONTROL",
                "REMOTE_CONTROL_STATE",
                "GNSS_SYSTEM",
                "HMI_LANGUAGE",
                "NEONODE_RESETLINE",
                "DATASET_BLOCK_SIZE",
                "UNKNOWN",
            ],
        ),
    ];

    /// The signal enumeration, grouped by what each signal touches.
    pub const fn groups() -> &'static [(&'static str, &'static [&'static str])] {
        SIGNAL_GROUPS
    }

    /// Every signal name across the groups, in group then declaration order.
    pub fn all_signals() -> Vec<&'static str> {
        SIGNAL_GROUPS
            .iter()
            .flat_map(|&(_, signals)| signals.iter().copied())
            .collect()
    }

    /// The group a signal name belongs to, or `None` when it is not catalogued.
    pub fn group_of(signal: &str) -> Option<&'static str> {
        SIGNAL_GROUPS
            .iter()
            .find(|&&(_, signals)| signals.contains(&signal))
            .map(|&(group, _)| group)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn signal_groups_include_ipsim_injection() {
            assert_eq!(group_of("IPSIM_PRESSED_KEY"), Some("hmi-input-injection"));
            assert_eq!(
                group_of("IPSIM_VOLUME_SIMULATION"),
                Some("hmi-input-injection")
            );
            assert_eq!(group_of("FAN"), Some("thermal-power"));
            assert_eq!(group_of("NO_SUCH_SIGNAL"), None);
        }

        #[test]
        fn all_signals_are_unique() {
            let signals = all_signals();
            let unique: std::collections::BTreeSet<&str> = signals.iter().copied().collect();
            assert_eq!(signals.len(), unique.len());
            assert!(signals.len() >= 50);
            let _ = groups();
        }
    }
}

/// The two SecurityAccess mechanism descriptors (reference only; no key material).
pub mod security {
    use crate::security::Mechanism;

    /// The lookup key for the classic GM seed/key mechanism.
    pub const CLASSIC_GM_SEED_KEY: &str = "classic-gm-seed-key";
    /// The lookup key for the ADIT ALD mechanism.
    pub const ADIT_ALD: &str = "adit-ald";

    /// Reference descriptors (`docs/security-access.md`). Neither carries key material or an
    /// algorithm.
    pub const MECHANISMS: &[(&str, Mechanism)] = &[
        (
            CLASSIC_GM_SEED_KEY,
            Mechanism {
                name: "Classic GM seed/key",
                family: "seed-key",
                summary: "A module returns a short seed; the tester returns a key computed from it by a \
                          per-module secret algorithm held by GM. The algorithm and keys are not in scope.",
                invalid_key_nrc: 0x35,
            },
        ),
        (
            ADIT_ALD,
            Mechanism {
                name: "ADIT ALD (GMG3 head unit)",
                family: "challenge-response",
                summary: "SecurityAccess 0x27 is a front-end over the ADIT Access-Level Daemon. The server \
                          returns a 96-byte device-bound challenge (16-byte rand() nonce, 8-byte timestamp, \
                          8-byte i.MX6 serial, 16-byte caller ECU_ID). The tester returns an RSA-2048 signed \
                          block; the daemon verifies the signature against the per-level public key (level.pub) \
                          and memcmp's the response against the live challenge. A bad key is NRC 0x35. The \
                          per-level private keys are held by GM/Bosch and are out of scope.",
                invalid_key_nrc: 0x35,
            },
        ),
    ];

    /// Every mechanism descriptor.
    pub const fn mechanisms() -> &'static [(&'static str, Mechanism)] {
        MECHANISMS
    }

    /// Look up a mechanism descriptor by key.
    ///
    /// # Errors
    /// Returns an error when `key` names no catalogued mechanism.
    pub fn mechanism(key: &str) -> crate::Result<Mechanism> {
        MECHANISMS
            .iter()
            .find(|&&(k, _)| k == key)
            .map(|&(_, m)| m)
            .ok_or_else(|| crate::Error::new(format!("unknown security-access mechanism: {key:?}")))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn mechanisms_cover_classic_and_ald() {
            let classic = mechanism(CLASSIC_GM_SEED_KEY).unwrap();
            let ald = mechanism(ADIT_ALD).unwrap();
            assert_eq!(classic.family, "seed-key");
            assert_eq!(ald.family, "challenge-response");
            assert_eq!(ald.invalid_key_nrc, 0x35);
            assert!(ald.summary.contains("RSA-2048"));
            assert!(mechanism("nope").is_err());
            assert_eq!(mechanisms().len(), 2);
        }
    }
}

/// The catalog of GM's published Mode $06 reference PDFs.
pub mod mode06 {
    use crate::mode06::Document;

    /// The base URL GM publishes the Mode $06 PDFs under.
    pub const GM_MODE6_BASE: &str = "https://gsitlc.ext.gm.com/gmspo/mode6/pdf/";
    /// The Chevrolet Volt's OBD group.
    pub const VOLT_OBD_GROUP: &str = "1A";

    const fn doc(
        filename: &'static str,
        obd_groups: &'static [&'static str],
        what: &'static str,
    ) -> Document {
        Document {
            filename,
            obd_groups,
            what,
        }
    }

    // GM 2017 OBD reference PDFs (docs/sources.md). An empty obd_groups marks a generic file.
    const DOCS: &[Document] = &[
        doc(
            "2017_OBD_GROUP_SUMMARY.pdf",
            &[],
            "index: Engine/Transmission diagnostic parameter file matrix",
        ),
        doc(
            "17OBDG01 Hybrid Diagnostics.pdf",
            &["1A", "1B"],
            "Hybrid powertrain diagnostic parameters (Volt, ELR, CT6)",
        ),
        doc(
            "17OBDG02 Hybrid Diagnostics.pdf",
            &["2"],
            "Hybrid powertrain diagnostic parameters (Malibu 1.8L hybrid)",
        ),
        doc(
            "17OBDG03 Engine Diagnostics.pdf",
            &["3"],
            "Engine diagnostic parameters (conventional gas 1.4L-6.2L)",
        ),
        doc(
            "17OBDG03 Transmisson Diagnostics.pdf",
            &["3"],
            "Transmission diagnostic parameters (GM's spelling)",
        ),
        doc(
            "17OBDG04 Engine Diagnostics.pdf",
            &["4"],
            "Engine diagnostic parameters (diesels: LH7, LWN, L5P Duramax)",
        ),
        doc(
            "17OBDG04 Transmisson Diagnostics.pdf",
            &["4"],
            "Transmission diagnostic parameters (GM's spelling)",
        ),
        doc(
            "17OBDG05 Engine Diagnostics.pdf",
            &["5"],
            "Engine diagnostic parameters (6.0L L96 HD)",
        ),
        doc(
            "17OBDG05 Transmission Diagnostics.pdf",
            &["5"],
            "Transmission diagnostic parameters",
        ),
        doc(
            "17OBDG06 Engine Diagnostics.pdf",
            &["6"],
            "Engine diagnostic parameters (6.0L L96)",
        ),
        doc(
            "17OBDG06 Transmission Diagnostics.pdf",
            &["6"],
            "Transmission diagnostic parameters",
        ),
        doc(
            "GM CAN mode $06 data final_dm.pdf",
            &[],
            "Generic Mode $06 monitor/test-ID definitions (CAN/GMLAN, ISO 15031-5)",
        ),
        doc(
            "GM Class2 mode $06 data final_dm.pdf",
            &[],
            "Generic Mode $06 monitor/test-ID definitions (legacy Class 2)",
        ),
    ];

    /// Every catalogued GM OBD reference PDF.
    pub const fn documents() -> &'static [Document] {
        DOCS
    }

    /// The documents applicable to an OBD group: its group-specific files plus the generic ones.
    pub fn files_for_group(group: &str) -> Vec<Document> {
        DOCS.iter()
            .copied()
            .filter(|d| d.obd_groups.is_empty() || d.obd_groups.contains(&group))
            .collect()
    }

    /// The documents applicable to the Chevrolet Volt (OBD Group 1A).
    pub fn volt_files() -> Vec<Document> {
        files_for_group(VOLT_OBD_GROUP)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn volt_is_group_1a() {
            assert_eq!(VOLT_OBD_GROUP, "1A");
            let names: std::collections::BTreeSet<&str> =
                volt_files().iter().map(|d| d.filename).collect();
            assert!(names.contains("17OBDG01 Hybrid Diagnostics.pdf"));
            assert!(names.contains("GM CAN mode $06 data final_dm.pdf"));
            assert!(!names.contains("17OBDG02 Hybrid Diagnostics.pdf"));
        }

        #[test]
        fn files_for_group_include_generic() {
            let names: std::collections::BTreeSet<&str> =
                files_for_group("3").iter().map(|d| d.filename).collect();
            assert!(names.contains("17OBDG03 Engine Diagnostics.pdf"));
            assert!(names.contains("GM CAN mode $06 data final_dm.pdf"));
            assert!(names.contains("2017_OBD_GROUP_SUMMARY.pdf"));
            assert!(documents().len() >= 10);
            assert!(GM_MODE6_BASE.starts_with("https://"));
        }
    }
}
