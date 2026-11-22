//! The read-only CLI, driven as a built binary — the Rust counterpart of the Python `test_cli.py`.

use std::process::Command;

/// Run the `udskit` binary with `args`; return its stdout and exit code.
fn run(args: &[&str]) -> (String, i32) {
    let output = Command::new(env!("CARGO_BIN_EXE_udskit"))
        .args(args)
        .output()
        .expect("spawn udskit");
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");
    (stdout, output.status.code().unwrap_or(-1))
}

#[test]
fn catalog_commands_print_expected() {
    let (out, code) = run(&["services"]);
    assert_eq!(code, 0);
    assert!(out.contains("RoutineControl"));
    assert!(out.contains("SecurityAccess"));
    assert!(out.contains("[GMLAN]"));

    let (out, code) = run(&["nrc"]);
    assert_eq!(code, 0);
    assert!(out.contains("requestCorrectlyReceived-ResponsePending"));
    let (out, _) = run(&["nrc", "31"]);
    assert!(out.contains("requestOutOfRange"));

    let (out, _) = run(&["did"]);
    assert!(out.contains("22 002F"));
    assert!(out.contains("identification"));

    let (out, _) = run(&["dtc", "P0A7F"]);
    assert!(out.contains("Hybrid/EV Battery Pack"));
    assert!(out.contains("17OBDG01"));
    let (out, _) = run(&["dtc", "U0184"]);
    assert!(out.contains("not in 17OBDG01"));
    let (out, _) = run(&["dtc", "P9999"]);
    assert!(out.contains("not in the reference subset (powertrain)"));

    let (out, _) = run(&["status", "24"]);
    assert!(out.contains("status 0x24"));
    assert!(out.contains("pendingDTC"));
    assert!(out.contains("testFailedSinceLastClear"));
    let (out, _) = run(&["status", "00"]);
    assert!(out.contains("(no status bits set)"));

    let (out, _) = run(&["routines"]);
    assert!(out.contains("StressTest"));
    assert!(out.contains("[H]"));

    let (out, _) = run(&["iocontrol"]);
    assert!(out.contains("hmi-input-injection"));
    assert!(out.contains("IPSIM_PRESSED_KEY"));

    let (out, _) = run(&["security"]);
    assert!(out.contains("ADIT ALD"));
    assert!(out.contains("RSA-2048"));
    assert!(out.contains("invalidKey=0x35"));

    let (out, _) = run(&["mode06"]);
    assert!(out.contains("17OBDG01 Hybrid Diagnostics.pdf"));
    assert!(out.contains("Chevrolet Volt is OBD Group 1A"));
}

#[test]
fn session_command_lists_and_builds() {
    let (out, code) = run(&["session"]);
    assert_eq!(code, 0);
    assert!(out.contains("defaultSession"));
    assert!(out.contains("extendedDiagnosticSession"));
    let (out, _) = run(&["session", "03"]);
    assert!(out.contains("extendedDiagnosticSession"));
    assert!(out.contains("1003"));
    assert_eq!(run(&["session", "00"]).1, 2); // reserved
}

#[test]
fn ecureset_command_lists_and_builds() {
    let (out, code) = run(&["ecureset"]);
    assert_eq!(code, 0);
    assert!(out.contains("hardReset"));
    assert!(out.contains("enableRapidPowerShutDown"));
    let (out, _) = run(&["ecureset", "01"]);
    assert!(out.contains("hardReset"));
    assert!(out.contains("1101"));
    assert_eq!(run(&["ecureset", "00"]).1, 2); // reserved
}

#[test]
fn testerpresent_command_builds() {
    let (out, code) = run(&["testerpresent"]);
    assert_eq!(code, 0);
    assert!(out.contains("zeroSubFunction"));
    assert!(out.contains("3e00"));
}

#[test]
fn decode_and_gmlan_render() {
    let (out, code) = run(&["decode", "7F2231"]);
    assert_eq!(code, 0);
    assert!(out.contains("requestOutOfRange"));

    let (out, _) = run(&["decode", "62 002F 6C"]);
    assert!(out.contains("ReadDataByIdentifier"));
    assert!(out.contains("DID 002F"));

    let (out, _) = run(&["decode", "7F1A78"]);
    assert!(out.contains("response pending"));

    let (out, code) = run(&["gmlan", "241"]);
    assert_eq!(code, 0);
    assert!(out.contains("response 0x641"));
    assert!(out.contains("filter   0x541"));
}

#[test]
fn bad_usage_exits_nonzero() {
    assert_eq!(run(&[]).1, 2); // no command
    assert_eq!(run(&["nope"]).1, 2); // unknown command
    assert_eq!(run(&["nrc", "zz"]).1, 2); // not hex
    assert_eq!(run(&["dtc", "Z0000"]).1, 2); // not a DTC letter
    assert_eq!(run(&["status", "FFF"]).1, 2); // not a one-octet status byte
    assert_eq!(run(&["gmlan", "800"]).1, 2); // out of 11-bit range
}
