//! The `udskit` command line: a read-only reference over the catalogs.
//!
//! Every sub-command prints from the in-crate catalogs or decodes a hex PDU you paste. Nothing
//! here opens a bus, sends a frame, or derives a key.

use std::process::ExitCode;

use udskit::{did, dtc, gmlan, iocontrol, message, mode06, nrc, routine, security, services};

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
        "did" => cmd_did(),
        "dtc" => cmd_dtc(rest),
        "status" => cmd_status(rest),
        "routines" => cmd_routines(),
        "iocontrol" => cmd_iocontrol(),
        "security" => cmd_security(),
        "mode06" => cmd_mode06(),
        "decode" => cmd_decode(rest),
        "gmlan" => cmd_gmlan(rest),
        other => {
            eprintln!("unknown command: {other}\n{USAGE}");
            2
        }
    };
    ExitCode::from(code)
}

const USAGE: &str = "usage: udskit <services|nrc|did|dtc|status|routines|iocontrol|security|mode06|decode|gmlan> [arg]";

fn parse_u8_hex(s: &str) -> Option<u8> {
    u8::from_str_radix(s.trim_start_matches("0x").trim_start_matches("0X"), 16).ok()
}

fn parse_u16_hex(s: &str) -> Option<u16> {
    u16::from_str_radix(s.trim_start_matches("0x").trim_start_matches("0X"), 16).ok()
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

fn cmd_services() -> u8 {
    for service in services::all_services() {
        println!(
            "{:#04x}  {:<8}  {}  [{}]",
            service.sid,
            service.mnemonic,
            service.name,
            service.standard.label()
        );
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

fn cmd_did() -> u8 {
    for entry in did::read_dids() {
        println!("22 {}  {}", entry.hex(), entry.category);
    }
    println!("--");
    for (category, count) in did::count_by_category() {
        println!("{count:>4}  {category}");
    }
    0
}

fn cmd_dtc(rest: &[String]) -> u8 {
    if let Some(arg) = rest.first() {
        match dtc::get(arg) {
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
    for entry in dtc::catalog() {
        let flag = if entry.in_17obdg01 { "Y" } else { "-" };
        println!("{}  [{flag}]  {}", entry.code, entry.name);
    }
    println!("--");
    let counts = dtc::APP_DTC_COUNTS
        .iter()
        .map(|&(prefix, n)| format!("{prefix}={n}"))
        .collect::<Vec<_>>()
        .join(", ");
    println!(
        "app DTC-name table: {} codes ({counts})",
        dtc::APP_DTC_TOTAL
    );
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

fn cmd_routines() -> u8 {
    for entry in routine::catalog() {
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

fn cmd_iocontrol() -> u8 {
    for &(group, signals) in iocontrol::signal_groups() {
        println!("{group} ({})", signals.len());
        for signal in signals {
            println!("    {signal}");
        }
    }
    0
}

fn cmd_security() -> u8 {
    for &(_, mech) in security::MECHANISMS {
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

fn cmd_mode06() -> u8 {
    for document in mode06::documents() {
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
        mode06::VOLT_OBD_GROUP,
        mode06::volt_files().len()
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
