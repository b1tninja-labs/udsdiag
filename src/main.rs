//! The `udskit` command line: a read-only reference over the catalogs.
//!
//! Every sub-command prints from the in-crate catalogs or decodes a hex PDU you paste. Nothing
//! here opens a bus, sends a frame, or derives a key. The commands that read GM reference data
//! (`services` partly, `did`, `dtc`, `routines`, `iocontrol`, `security`, `mode06`) require the
//! default `gm` feature; the ISO-only commands (`nrc`, `status`, `decode`, `gmlan`) are always
//! present.

use std::process::ExitCode;

#[cfg(feature = "gm")]
use udskit::gm;
#[cfg(feature = "gm")]
use udskit::vendor::VendorProfile;
use udskit::{
    accesstiming, commcontrol, did, dtc, dtcsetting, dynamicdid, ecureset, gmlan, linkcontrol,
    memory, message, nrc, periodic, readdtc, responseonevent, securedtransmission, services,
    session, testerpresent, transfer,
};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(command) = args.first() else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };
    let rest = &args[1..];
    let code = match command.as_str() {
        "services" => cmd_services(),
        "nrc" => cmd_nrc(rest),
        "session" => cmd_session(rest),
        "ecureset" => cmd_ecureset(rest),
        "testerpresent" => cmd_testerpresent(),
        "commcontrol" => cmd_commcontrol(rest),
        "accesstiming" => cmd_accesstiming(rest),
        "linkcontrol" => cmd_linkcontrol(rest),
        "dtcsetting" => cmd_dtcsetting(rest),
        "dddi" => cmd_dddi(rest),
        "rdbi" => cmd_rdbi(rest),
        "wdbi" => cmd_wdbi(rest),
        "cleardtc" => cmd_cleardtc(rest),
        "readdtc" => cmd_readdtc(rest),
        "readmem" => cmd_readmem(rest),
        "writemem" => cmd_writemem(rest),
        "transfer" => cmd_transfer(rest),
        "sdt" => cmd_sdt(rest),
        "roe" => cmd_roe(rest),
        "rsdbi" => cmd_rsdbi(rest),
        "periodic" => cmd_periodic(rest),
        "status" => cmd_status(rest),
        "decode" => cmd_decode(rest),
        "gmlan" => cmd_gmlan(rest),
        #[cfg(feature = "gm")]
        "did" => cmd_did(),
        #[cfg(feature = "gm")]
        "dtc" => cmd_dtc(rest),
        #[cfg(feature = "gm")]
        "routines" => cmd_routines(),
        #[cfg(feature = "gm")]
        "iocontrol" => cmd_iocontrol(),
        #[cfg(feature = "gm")]
        "security" => cmd_security(),
        #[cfg(feature = "gm")]
        "mode06" => cmd_mode06(),
        other => {
            eprintln!("unknown command: {other}\n{USAGE}");
            2
        }
    };
    ExitCode::from(code)
}

const USAGE: &str = "usage: udskit <services|nrc|session|ecureset|testerpresent|commcontrol|accesstiming|linkcontrol|dtcsetting|dddi|rdbi|wdbi|rsdbi|periodic|cleardtc|readdtc|readmem|writemem|transfer|sdt|roe|did|dtc|status|routines|iocontrol|security|mode06|decode|gmlan> [arg]";

fn parse_u8_hex(s: &str) -> Option<u8> {
    u8::from_str_radix(s.trim_start_matches("0x").trim_start_matches("0X"), 16).ok()
}

fn parse_u16_hex(s: &str) -> Option<u16> {
    u16::from_str_radix(s.trim_start_matches("0x").trim_start_matches("0X"), 16).ok()
}

fn parse_u32_hex(s: &str) -> Option<u32> {
    u32::from_str_radix(s.trim_start_matches("0x").trim_start_matches("0X"), 16).ok()
}

fn parse_hex_bytes(s: &str) -> Option<Vec<u8>> {
    let cleaned: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    if cleaned.len() % 2 != 0 {
        return None;
    }
    (0..cleaned.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&cleaned[i..i + 2], 16).ok())
        .collect()
}

fn hex_string(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes.iter().fold(String::new(), |mut acc, b| {
        let _ = write!(acc, "{b:02x}");
        acc
    })
}

fn print_service(service: &services::Service) {
    println!(
        "{:#04x}  {:<8}  {}  [{}]",
        service.sid,
        service.mnemonic,
        service.name,
        service.standard.label()
    );
}

fn cmd_services() -> u8 {
    // The ISO 14229-1 core, then each vendor layer (GM GMLAN) composed on top.
    for service in services::all_services() {
        print_service(service);
    }
    #[cfg(feature = "gm")]
    for service in gm::GM.services() {
        print_service(service);
    }
    0
}

fn cmd_nrc(rest: &[String]) -> u8 {
    if let Some(arg) = rest.first() {
        let Some(value) = parse_u8_hex(arg) else {
            eprintln!("nrc: expected a hex value, e.g. 31");
            return 2;
        };
        println!("{value:#04x}  {}", nrc::describe(value));
        return 0;
    }
    for code in nrc::all_codes() {
        println!("{:#04x}  {:<8}  {}", code.value, code.mnemonic, code.name);
    }
    0
}

#[cfg(feature = "gm")]
fn cmd_did() -> u8 {
    for entry in gm::dids::read_dids() {
        println!("22 {}  {}", entry.hex(), entry.category);
    }
    println!("--");
    for (category, count) in gm::dids::count_by_category() {
        println!("{count:>4}  {category}");
    }
    0
}

#[cfg(feature = "gm")]
fn cmd_dtc(rest: &[String]) -> u8 {
    if let Some(arg) = rest.first() {
        match gm::dtc::get(arg) {
            Some(entry) => {
                let flag = if entry.in_17obdg01 {
                    "17OBDG01"
                } else {
                    "not in 17OBDG01"
                };
                println!(
                    "{}",
                    format!("{}  {}  [{}]  {}", entry.code, entry.name, flag, entry.note)
                        .trim_end()
                );
            }
            None => match dtc::category_of(arg) {
                Ok(category) => println!(
                    "{}: not in the reference subset ({category})",
                    arg.to_ascii_uppercase()
                ),
                Err(err) => {
                    eprintln!("dtc: {err}");
                    return 2;
                }
            },
        }
        return 0;
    }
    for entry in gm::dtc::reference() {
        let flag = if entry.in_17obdg01 { "Y" } else { "-" };
        println!("{}  [{flag}]  {}", entry.code, entry.name);
    }
    println!("--");
    let counts = gm::dtc::APP_DTC_COUNTS
        .iter()
        .map(|&(prefix, n)| format!("{prefix}={n}"))
        .collect::<Vec<_>>()
        .join(", ");
    println!(
        "app DTC-name table: {} codes ({counts})",
        gm::dtc::APP_DTC_TOTAL
    );
    0
}

fn cmd_session(rest: &[String]) -> u8 {
    if let Some(arg) = rest.first() {
        let Some(value) = parse_u8_hex(arg) else {
            eprintln!("session: expected a hex session type, e.g. 03");
            return 2;
        };
        match session::build_request(value) {
            Ok(pdu) => println!(
                "10 {value:02x}  {}  -> {}",
                session::describe(value),
                hex_string(&pdu)
            ),
            Err(err) => {
                eprintln!("session: {err}");
                return 2;
            }
        }
        return 0;
    }
    for entry in session::session_types() {
        println!(
            "{:#04x}  {:<6}  {}",
            entry.value, entry.mnemonic, entry.name
        );
    }
    0
}

fn cmd_ecureset(rest: &[String]) -> u8 {
    if let Some(arg) = rest.first() {
        let Some(value) = parse_u8_hex(arg) else {
            eprintln!("ecureset: expected a hex reset type, e.g. 01");
            return 2;
        };
        match ecureset::build_request(value) {
            Ok(pdu) => println!(
                "11 {value:02x}  {}  -> {}",
                ecureset::describe(value),
                hex_string(&pdu)
            ),
            Err(err) => {
                eprintln!("ecureset: {err}");
                return 2;
            }
        }
        return 0;
    }
    for entry in ecureset::reset_types() {
        println!(
            "{:#04x}  {:<8}  {}",
            entry.value, entry.mnemonic, entry.name
        );
    }
    0
}

fn cmd_testerpresent() -> u8 {
    println!(
        "3e 00  zeroSubFunction  -> {}",
        hex_string(&testerpresent::build_request())
    );
    0
}

fn cmd_commcontrol(rest: &[String]) -> u8 {
    if rest.is_empty() {
        for entry in commcontrol::control_types() {
            println!(
                "{:#04x}  {:<11}  {}",
                entry.value, entry.mnemonic, entry.name
            );
        }
        return 0;
    }
    if rest.len() < 2 {
        eprintln!("commcontrol: usage: commcontrol <controlType> <communicationType> [nodeId]");
        return 2;
    }
    let (Some(control_type), Some(comm_type)) = (parse_u8_hex(&rest[0]), parse_u8_hex(&rest[1]))
    else {
        eprintln!("commcontrol: expected hex controlType and communicationType");
        return 2;
    };
    let node_id = if let Some(s) = rest.get(2) {
        let Some(value) = parse_u16_hex(s) else {
            eprintln!("commcontrol: expected a hex nodeIdentificationNumber");
            return 2;
        };
        Some(value)
    } else {
        None
    };
    match commcontrol::build_request(control_type, comm_type, node_id) {
        Ok(pdu) => println!(
            "28 {control_type:02x}  {}  -> {}",
            commcontrol::describe(control_type),
            hex_string(&pdu)
        ),
        Err(err) => {
            eprintln!("commcontrol: {err}");
            return 2;
        }
    }
    0
}

fn cmd_dtcsetting(rest: &[String]) -> u8 {
    if rest.is_empty() {
        for entry in dtcsetting::setting_types() {
            println!(
                "{:#04x}  {:<4}  {}",
                entry.value, entry.mnemonic, entry.name
            );
        }
        return 0;
    }
    let Some(setting_type) = parse_u8_hex(&rest[0]) else {
        eprintln!("dtcsetting: expected a hex DTCSettingType, e.g. 02");
        return 2;
    };
    let option_record = if let Some(s) = rest.get(1) {
        let Some(bytes) = parse_hex_bytes(s) else {
            eprintln!("dtcsetting: expected a hex option record");
            return 2;
        };
        bytes
    } else {
        Vec::new()
    };
    match dtcsetting::build_request(setting_type, &option_record) {
        Ok(pdu) => println!(
            "85 {setting_type:02x}  {}  -> {}",
            dtcsetting::describe(setting_type),
            hex_string(&pdu)
        ),
        Err(err) => {
            eprintln!("dtcsetting: {err}");
            return 2;
        }
    }
    0
}

fn cmd_rdbi(rest: &[String]) -> u8 {
    if rest.is_empty() {
        eprintln!("rdbi: usage: rdbi <DID> [DID...]");
        return 2;
    }
    let mut dids = Vec::with_capacity(rest.len());
    for arg in rest {
        let Some(did_value) = parse_u16_hex(arg) else {
            eprintln!("rdbi: expected hex DIDs, e.g. 002F");
            return 2;
        };
        dids.push(did_value);
    }
    match did::build_read_request(&dids) {
        Ok(pdu) => println!("ReadDataByIdentifier  -> {}", hex_string(&pdu)),
        Err(err) => {
            eprintln!("rdbi: {err}");
            return 2;
        }
    }
    0
}

fn cmd_wdbi(rest: &[String]) -> u8 {
    if rest.len() < 2 {
        eprintln!("wdbi: usage: wdbi <DID> <dataHex>");
        return 2;
    }
    let (Some(did_value), Some(data)) = (parse_u16_hex(&rest[0]), parse_hex_bytes(&rest[1])) else {
        eprintln!("wdbi: expected a hex DID and a hex data record");
        return 2;
    };
    match did::build_write_request(did_value, &data) {
        Ok(pdu) => println!("WriteDataByIdentifier  -> {}", hex_string(&pdu)),
        Err(err) => {
            eprintln!("wdbi: {err}");
            return 2;
        }
    }
    0
}

fn cmd_cleardtc(rest: &[String]) -> u8 {
    let Some(arg) = rest.first() else {
        eprintln!(
            "cleardtc: usage: cleardtc <groupOfDTC-hex>, e.g. FFFFFF (all) or FFFF33 (emissions)"
        );
        return 2;
    };
    let Some(group) = parse_u32_hex(arg) else {
        eprintln!("cleardtc: expected a hex groupOfDTC");
        return 2;
    };
    match dtc::build_clear_request(group) {
        Ok(pdu) => println!(
            "ClearDiagnosticInformation group {group:#08x}  -> {}",
            hex_string(&pdu)
        ),
        Err(err) => {
            eprintln!("cleardtc: {err}");
            return 2;
        }
    }
    0
}

fn cmd_readdtc(rest: &[String]) -> u8 {
    if rest.is_empty() {
        for entry in readdtc::report_types() {
            println!(
                "{:#04x}  {:<14}  {}",
                entry.value, entry.mnemonic, entry.name
            );
        }
        return 0;
    }
    let Some(report_type) = parse_u8_hex(&rest[0]) else {
        eprintln!("readdtc: expected a hex reportType, e.g. 02");
        return 2;
    };
    let parameters = if let Some(s) = rest.get(1) {
        let Some(bytes) = parse_hex_bytes(s) else {
            eprintln!("readdtc: expected hex parameters");
            return 2;
        };
        bytes
    } else {
        Vec::new()
    };
    let pdu = readdtc::build_request(report_type, &parameters);
    println!(
        "19 {report_type:02x}  {}  -> {}",
        readdtc::describe(report_type),
        hex_string(&pdu)
    );
    0
}

fn cmd_readmem(rest: &[String]) -> u8 {
    if rest.len() < 2 {
        eprintln!("readmem: usage: readmem <addressHex> <sizeHex>");
        return 2;
    }
    let (Some(address), Some(size)) = (parse_hex_bytes(&rest[0]), parse_hex_bytes(&rest[1])) else {
        eprintln!("readmem: expected hex memoryAddress and memorySize");
        return 2;
    };
    match memory::build_read_request(&address, &size) {
        Ok(pdu) => println!("ReadMemoryByAddress  -> {}", hex_string(&pdu)),
        Err(err) => {
            eprintln!("readmem: {err}");
            return 2;
        }
    }
    0
}

fn cmd_writemem(rest: &[String]) -> u8 {
    if rest.len() < 3 {
        eprintln!("writemem: usage: writemem <addressHex> <sizeHex> <dataHex>");
        return 2;
    }
    let (Some(address), Some(size), Some(data)) = (
        parse_hex_bytes(&rest[0]),
        parse_hex_bytes(&rest[1]),
        parse_hex_bytes(&rest[2]),
    ) else {
        eprintln!("writemem: expected hex memoryAddress, memorySize and dataRecord");
        return 2;
    };
    match memory::build_write_request(&address, &size, &data) {
        Ok(pdu) => println!("WriteMemoryByAddress  -> {}", hex_string(&pdu)),
        Err(err) => {
            eprintln!("writemem: {err}");
            return 2;
        }
    }
    0
}

fn cmd_rsdbi(rest: &[String]) -> u8 {
    let Some(arg) = rest.first() else {
        eprintln!("rsdbi: usage: rsdbi <DID>, e.g. F190");
        return 2;
    };
    let Some(did_value) = parse_u16_hex(arg) else {
        eprintln!("rsdbi: expected a hex DID");
        return 2;
    };
    println!(
        "ReadScalingDataByIdentifier  -> {}",
        hex_string(&did::build_scaling_request(did_value))
    );
    0
}

fn cmd_periodic(rest: &[String]) -> u8 {
    if rest.is_empty() {
        for entry in periodic::transmission_modes() {
            println!(
                "{:#04x}  {:<4}  {}",
                entry.value, entry.mnemonic, entry.name
            );
        }
        return 0;
    }
    let Some(mode) = parse_u8_hex(&rest[0]) else {
        eprintln!("periodic: expected a hex transmissionMode, e.g. 02");
        return 2;
    };
    let mut ids = Vec::with_capacity(rest.len().saturating_sub(1));
    for arg in &rest[1..] {
        let Some(id) = parse_u8_hex(arg) else {
            eprintln!("periodic: expected hex periodicDataIdentifiers (1 byte each)");
            return 2;
        };
        ids.push(id);
    }
    match periodic::build_request(mode, &ids) {
        Ok(pdu) => println!(
            "2a {mode:02x}  {}  -> {}",
            periodic::describe(mode),
            hex_string(&pdu)
        ),
        Err(err) => {
            eprintln!("periodic: {err}");
            return 2;
        }
    }
    0
}

fn cmd_accesstiming(rest: &[String]) -> u8 {
    if rest.is_empty() {
        for entry in accesstiming::access_types() {
            println!(
                "{:#04x}  {:<6}  {}",
                entry.value, entry.mnemonic, entry.name
            );
        }
        return 0;
    }
    let Some(access_type) = parse_u8_hex(&rest[0]) else {
        eprintln!("accesstiming: expected a hex timingParameterAccessType, e.g. 03");
        return 2;
    };
    let record = if let Some(s) = rest.get(1) {
        let Some(bytes) = parse_hex_bytes(s) else {
            eprintln!("accesstiming: expected a hex TimingParameterRequestRecord");
            return 2;
        };
        bytes
    } else {
        Vec::new()
    };
    match accesstiming::build_request(access_type, &record) {
        Ok(pdu) => println!(
            "83 {access_type:02x}  {}  -> {}",
            accesstiming::describe(access_type),
            hex_string(&pdu)
        ),
        Err(err) => {
            eprintln!("accesstiming: {err}");
            return 2;
        }
    }
    0
}

fn cmd_dddi(rest: &[String]) -> u8 {
    match rest.first().map(String::as_str) {
        None => {
            for entry in dynamicdid::definition_types() {
                println!(
                    "{:#04x}  {:<6}  {}",
                    entry.value, entry.mnemonic, entry.name
                );
            }
            0
        }
        Some("clear") => {
            let dddi = if let Some(s) = rest.get(1) {
                let Some(value) = parse_u16_hex(s) else {
                    eprintln!("dddi: expected a hex dynamicallyDefinedDataIdentifier");
                    return 2;
                };
                Some(value)
            } else {
                None
            };
            println!(
                "clearDynamicallyDefinedDataIdentifier  -> {}",
                hex_string(&dynamicdid::build_clear(dddi))
            );
            0
        }
        Some(_) => {
            eprintln!(
                "dddi: from the CLI only 'dddi' (list) and 'dddi clear [DID]' are available; the \
                 define builders are library functions"
            );
            2
        }
    }
}

fn cmd_linkcontrol(rest: &[String]) -> u8 {
    match rest.first().map(String::as_str) {
        None => {
            for entry in linkcontrol::link_control_types() {
                println!(
                    "type {:#04x}  {:<6}  {}",
                    entry.value, entry.mnemonic, entry.name
                );
            }
            for entry in linkcontrol::mode_identifiers() {
                println!(
                    "mode {:#04x}  {:<10}  {}",
                    entry.value, entry.mnemonic, entry.name
                );
            }
            0
        }
        Some("fixed") => {
            let Some(value) = rest.get(1).and_then(|s| parse_u8_hex(s)) else {
                eprintln!("linkcontrol: usage: linkcontrol fixed <modeIdentifier>");
                return 2;
            };
            println!(
                "verifyModeTransitionWithFixedParameter ({})  -> {}",
                linkcontrol::mode_identifier(value).map_or("?", |m| m.name),
                hex_string(&linkcontrol::build_verify_fixed(value))
            );
            0
        }
        Some("specific") => {
            let Some(baud) = rest.get(1).and_then(|s| parse_u32_hex(s)) else {
                eprintln!("linkcontrol: usage: linkcontrol specific <baudRateHex>");
                return 2;
            };
            match linkcontrol::build_verify_specific(baud) {
                Ok(pdu) => println!(
                    "verifyModeTransitionWithSpecificParameter  -> {}",
                    hex_string(&pdu)
                ),
                Err(err) => {
                    eprintln!("linkcontrol: {err}");
                    return 2;
                }
            }
            0
        }
        Some("transition") => {
            println!(
                "transitionMode  -> {}",
                hex_string(&linkcontrol::build_transition())
            );
            0
        }
        Some(_) => {
            eprintln!(
                "linkcontrol: usage: linkcontrol [fixed <mode> | specific <baud> | transition]"
            );
            2
        }
    }
}

fn cmd_transfer(rest: &[String]) -> u8 {
    match rest.first().map(String::as_str) {
        Some("download" | "upload") if rest.len() >= 4 => {
            let (Some(dfi), Some(addr), Some(size)) = (
                parse_u8_hex(&rest[1]),
                parse_hex_bytes(&rest[2]),
                parse_hex_bytes(&rest[3]),
            ) else {
                eprintln!("transfer: expected hex dataFormatIdentifier, address and size");
                return 2;
            };
            let built = if rest[0] == "download" {
                transfer::build_request_download(dfi, &addr, &size)
            } else {
                transfer::build_request_upload(dfi, &addr, &size)
            };
            match built {
                Ok(pdu) => println!("{}  -> {}", rest[0], hex_string(&pdu)),
                Err(err) => {
                    eprintln!("transfer: {err}");
                    return 2;
                }
            }
            0
        }
        Some("data") if rest.len() >= 2 => {
            let Some(bsc) = parse_u8_hex(&rest[1]) else {
                eprintln!("transfer: expected a hex blockSequenceCounter");
                return 2;
            };
            let data = rest
                .get(2)
                .and_then(|s| parse_hex_bytes(s))
                .unwrap_or_default();
            println!(
                "TransferData  -> {}",
                hex_string(&transfer::build_transfer_data(bsc, &data))
            );
            0
        }
        Some("exit") => {
            let params = rest
                .get(1)
                .and_then(|s| parse_hex_bytes(s))
                .unwrap_or_default();
            println!(
                "RequestTransferExit  -> {}",
                hex_string(&transfer::build_request_transfer_exit(&params))
            );
            0
        }
        Some("modes") | None => {
            for entry in transfer::modes_of_operation() {
                println!("{:#04x}  {}", entry.value, entry.name);
            }
            0
        }
        _ => {
            eprintln!(
                "transfer: usage: transfer <download|upload <dfi> <addr> <size> | data <bsc> [data] | exit [params] | modes>"
            );
            2
        }
    }
}

fn cmd_sdt(rest: &[String]) -> u8 {
    let Some(record) = rest.first().and_then(|s| parse_hex_bytes(s)) else {
        eprintln!("sdt: usage: sdt <securityDataRequestRecordHex>");
        return 2;
    };
    match securedtransmission::build_request(&record) {
        Ok(pdu) => println!("SecuredDataTransmission  -> {}", hex_string(&pdu)),
        Err(err) => {
            eprintln!("sdt: {err}");
            return 2;
        }
    }
    0
}

fn cmd_roe(rest: &[String]) -> u8 {
    if rest.is_empty() {
        for entry in responseonevent::event_types() {
            println!(
                "{:#04x}  {:<8}  {} (record {}B)",
                entry.value, entry.mnemonic, entry.name, entry.event_type_record_len
            );
        }
        return 0;
    }
    if rest.len() < 2 {
        eprintln!(
            "roe: usage: roe <eventType> <eventWindowTime> [eventTypeRecordHex] [serviceToRespondToHex]"
        );
        return 2;
    }
    let (Some(event_type), Some(window)) = (parse_u8_hex(&rest[0]), parse_u8_hex(&rest[1])) else {
        eprintln!("roe: expected hex eventType and eventWindowTime");
        return 2;
    };
    let record = rest
        .get(2)
        .and_then(|s| parse_hex_bytes(s))
        .unwrap_or_default();
    let str_record = rest
        .get(3)
        .and_then(|s| parse_hex_bytes(s))
        .unwrap_or_default();
    match responseonevent::build_request(event_type, window, &record, &str_record) {
        Ok(pdu) => println!(
            "86 {event_type:02x}  {}  -> {}",
            responseonevent::describe(event_type & 0x3F),
            hex_string(&pdu)
        ),
        Err(err) => {
            eprintln!("roe: {err}");
            return 2;
        }
    }
    0
}

fn cmd_status(rest: &[String]) -> u8 {
    let Some(arg) = rest.first() else {
        eprintln!("status: expected a hex status byte, e.g. 24");
        return 2;
    };
    let Some(value) = parse_u8_hex(arg) else {
        eprintln!("status: a DTC status byte is one octet (0x00-0xFF)");
        return 2;
    };
    println!("status {value:#04x}  ({value:08b})");
    let bits = dtc::decode_status(value);
    for bit in &bits {
        println!(
            "  bit {}  {:<6}  {}: {}",
            bit.bit, bit.mnemonic, bit.name, bit.meaning
        );
    }
    if bits.is_empty() {
        println!("  (no status bits set)");
    }
    0
}

#[cfg(feature = "gm")]
fn cmd_routines() -> u8 {
    for entry in gm::routines::catalog() {
        println!(
            "31 .. {}  [{}]  {}: {}",
            entry.hex(),
            entry.risk,
            entry.name,
            entry.effect
        );
    }
    0
}

#[cfg(feature = "gm")]
fn cmd_iocontrol() -> u8 {
    for &(group, signals) in gm::signals::groups() {
        println!("{group} ({})", signals.len());
        for signal in signals {
            println!("    {signal}");
        }
    }
    0
}

#[cfg(feature = "gm")]
fn cmd_security() -> u8 {
    for &(_, mech) in gm::security::MECHANISMS {
        println!(
            "{}  [{}]  invalidKey={:#04x}",
            mech.name, mech.family, mech.invalid_key_nrc
        );
        println!("    {}", mech.summary);
    }
    println!("--");
    println!(
        "level pairing: requestSeed 0x01 <-> sendKey 0x02, 0x03 <-> 0x04, ... (odd <-> next even)"
    );
    0
}

#[cfg(feature = "gm")]
fn cmd_mode06() -> u8 {
    for document in gm::mode06::documents() {
        let groups = if document.obd_groups.is_empty() {
            "generic".to_owned()
        } else {
            document.obd_groups.join(",")
        };
        println!("[{groups}]  {}  — {}", document.filename, document.what);
    }
    println!("--");
    println!(
        "Chevrolet Volt is OBD Group {}: {} applicable files",
        gm::mode06::VOLT_OBD_GROUP,
        gm::mode06::volt_files().len()
    );
    0
}

fn cmd_decode(rest: &[String]) -> u8 {
    let joined = rest.join(" ");
    let Some(raw) = parse_hex_bytes(&joined) else {
        eprintln!("decode: expected response PDU bytes, e.g. '62 002F 6C' or 7F2231");
        return 2;
    };
    let response = match message::decode_response(&raw) {
        Ok(response) => response,
        Err(err) => {
            eprintln!("decode: {err}");
            return 2;
        }
    };
    match response {
        message::Response::Negative(negative) => {
            println!("{}", negative.describe());
            if negative.is_response_pending() {
                println!("(response pending — keep waiting)");
            }
        }
        message::Response::Positive(positive) => {
            let req_sid = positive.request_sid();
            println!(
                "positive response to {} ({req_sid:#04x})",
                services::name_of(req_sid)
            );
            if req_sid == did::READ_DATA_BY_IDENTIFIER && raw.len() >= 3 {
                if let Ok((did_value, value)) = did::parse_read_response(&raw) {
                    println!(
                        "  DID {did_value:04X} ({}) = {}",
                        did::classify(did_value),
                        hex_string(&value)
                    );
                    return 0;
                }
            }
            println!("  data = {}", hex_string(&positive.data));
        }
    }
    0
}

fn cmd_gmlan(rest: &[String]) -> u8 {
    let Some(arg) = rest.first() else {
        eprintln!("gmlan: expected an 11-bit request CAN id, e.g. 241");
        return 2;
    };
    let Some(request_id) = parse_u16_hex(arg) else {
        eprintln!("gmlan: expected a hex CAN id");
        return 2;
    };
    let (Ok(response), Ok(filter)) = (gmlan::response_id(request_id), gmlan::filter_id(request_id))
    else {
        eprintln!("gmlan: {arg} is out of range for the GMLAN offsets");
        return 2;
    };
    println!("request  {request_id:#05x}");
    println!("response {response:#05x}  (+0x400)");
    println!("filter   {filter:#05x}  (+0x300)");
    0
}
