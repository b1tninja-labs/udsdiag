"""The ``udsdiag`` command line: a read-only reference over the catalogs.

Every sub-command prints from the in-package catalogs or decodes a hex PDU you paste. Nothing
here opens a bus, sends a frame, or derives a key.
"""

from __future__ import annotations

import argparse
from collections.abc import Sequence

from udsdiag import did, dtc, gmlan, iocontrol, message, mode06, nrc, routine, security, services


def _cmd_services(_: argparse.Namespace) -> int:
    for service in services.all_services():
        print(f"{service.sid:#04x}  {service.mnemonic:<8}  {service.name}  [{service.standard}]")
    return 0


def _cmd_nrc(args: argparse.Namespace) -> int:
    if args.value is not None:
        value = int(args.value, 16)
        print(f"{value:#04x}  {nrc.describe(value)}")
        return 0
    for code in nrc.all_codes():
        print(f"{code.value:#04x}  {code.mnemonic:<8}  {code.name}")
    return 0


def _cmd_did(_: argparse.Namespace) -> int:
    for entry in did.read_dids():
        print(f"22 {entry.hex}  {entry.category}")
    print("--")
    for category, count in sorted(did.count_by_category().items()):
        print(f"{count:>4}  {category}")
    return 0


def _cmd_dtc(args: argparse.Namespace) -> int:
    if args.code is not None:
        entry = dtc.get(args.code)
        if entry is None:
            print(f"{args.code.upper()}: not in the reference subset ({dtc.category_of(args.code)})")
            return 0
        flag = "17OBDG01" if entry.in_17obdg01 else "not in 17OBDG01"
        print(f"{entry.code}  {entry.name}  [{flag}]  {entry.note}".rstrip())
        return 0
    for entry in dtc.catalog():
        flag = "Y" if entry.in_17obdg01 else "-"
        print(f"{entry.code}  [{flag}]  {entry.name}")
    print("--")
    print(f"app DTC-name table: {dtc.APP_DTC_TOTAL} codes {dtc.APP_DTC_COUNTS}")
    return 0


def _cmd_routines(_: argparse.Namespace) -> int:
    for entry in routine.catalog():
        print(f"31 .. {entry.hex}  [{entry.risk}]  {entry.name}: {entry.effect}")
    return 0


def _cmd_iocontrol(_: argparse.Namespace) -> int:
    for group, signals in iocontrol.signal_groups().items():
        print(f"{group} ({len(signals)})")
        for signal in signals:
            print(f"    {signal}")
    return 0


def _cmd_security(_: argparse.Namespace) -> int:
    for mech in security.MECHANISMS.values():
        print(f"{mech.name}  [{mech.family}]  invalidKey={mech.invalid_key_nrc:#04x}")
        print(f"    {mech.summary}")
    print("--")
    print("level pairing: requestSeed 0x01 <-> sendKey 0x02, 0x03 <-> 0x04, ... (odd <-> next even)")
    return 0


def _cmd_mode06(_: argparse.Namespace) -> int:
    for doc in mode06.documents():
        groups = ",".join(doc.obd_groups) if doc.obd_groups else "generic"
        print(f"[{groups}]  {doc.filename}  — {doc.what}")
    print("--")
    print(f"Chevrolet Volt is OBD Group {mode06.VOLT_OBD_GROUP}: {len(mode06.volt_files())} applicable files")
    return 0


def _cmd_decode(args: argparse.Namespace) -> int:
    raw = bytes.fromhex(args.hex.replace(" ", ""))
    response = message.decode_response(raw)
    if isinstance(response, message.NegativeResponse):
        print(response.describe())
        if response.is_response_pending:
            print("(response pending — keep waiting)")
        return 0
    req_sid = response.request_sid
    print(f"positive response to {services.name_of(req_sid)} ({req_sid:#04x})")
    if req_sid == did.READ_DATA_BY_IDENTIFIER and len(raw) >= 3:
        did_value, value = did.parse_read_response(raw)
        print(f"  DID {did_value:04X} ({did.classify(did_value)}) = {value.hex()}")
    else:
        print(f"  data = {response.data.hex()}")
    return 0


def _cmd_gmlan(args: argparse.Namespace) -> int:
    request_id = int(args.request_id, 16)
    print(f"request  {request_id:#05x}")
    print(f"response {gmlan.response_id(request_id):#05x}  (+0x400)")
    print(f"filter   {gmlan.filter_id(request_id):#05x}  (+0x300)")
    return 0


def build_parser() -> argparse.ArgumentParser:
    """The argument parser for the ``udsdiag`` command."""
    parser = argparse.ArgumentParser(prog="udsdiag", description="Offline UDS/GMLAN diagnostic reference.")
    sub = parser.add_subparsers(dest="command", required=True)

    sub.add_parser("services", help="list the diagnostic service catalog").set_defaults(func=_cmd_services)

    p_nrc = sub.add_parser("nrc", help="list or describe negative response codes")
    p_nrc.add_argument("value", nargs="?", help="a hex NRC value, e.g. 31")
    p_nrc.set_defaults(func=_cmd_nrc)

    sub.add_parser("did", help="list the Voltage 2.3.0 DID catalog").set_defaults(func=_cmd_did)

    p_dtc = sub.add_parser("dtc", help="list or look up DTC reference entries")
    p_dtc.add_argument("code", nargs="?", help="a DTC display code, e.g. P0A7F")
    p_dtc.set_defaults(func=_cmd_dtc)

    sub.add_parser("routines", help="list the RoutineControl catalog").set_defaults(func=_cmd_routines)
    sub.add_parser("iocontrol", help="list the IOControl signal groups").set_defaults(func=_cmd_iocontrol)
    sub.add_parser("security", help="show the SecurityAccess mechanisms").set_defaults(func=_cmd_security)
    sub.add_parser("mode06", help="list the GM Mode $06 document catalog").set_defaults(func=_cmd_mode06)

    p_decode = sub.add_parser("decode", help="decode a response PDU from hex")
    p_decode.add_argument("hex", help="response PDU bytes, e.g. '62 002F 6C' or 7F2231")
    p_decode.set_defaults(func=_cmd_decode)

    p_gmlan = sub.add_parser("gmlan", help="show GMLAN response/filter IDs for a request ID")
    p_gmlan.add_argument("request_id", help="an 11-bit request CAN id, e.g. 241")
    p_gmlan.set_defaults(func=_cmd_gmlan)

    return parser


def main(argv: Sequence[str] | None = None) -> int:
    """Run the ``udsdiag`` command line."""
    parser = build_parser()
    args = parser.parse_args(argv)
    func = args.func
    result: int = func(args)
    return result
