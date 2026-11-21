from __future__ import annotations

import pytest

from udsdiag import cli


def test_services_command_lists_services(capsys):
    assert cli.main(["services"]) == 0
    out = capsys.readouterr().out
    assert "RoutineControl" in out
    assert "SecurityAccess" in out


def test_decode_command_reads_a_negative_response(capsys):
    assert cli.main(["decode", "7F2231"]) == 0
    out = capsys.readouterr().out
    assert "requestOutOfRange" in out
    # a positive ReadDataByIdentifier response decodes its DID
    assert cli.main(["decode", "62 002F 6C"]) == 0
    out = capsys.readouterr().out
    assert "002F" in out


def test_dtc_lookup(capsys):
    assert cli.main(["dtc", "P0A7F"]) == 0
    out = capsys.readouterr().out
    assert "Hybrid/EV Battery Pack" in out


def test_status_command_decodes_a_status_byte(capsys):
    assert cli.main(["status", "24"]) == 0
    out = capsys.readouterr().out
    assert "pendingDTC" in out
    assert "testFailedSinceLastClear" in out
    assert cli.main(["status", "FFF"]) == 2


def test_no_command_errors():
    with pytest.raises(SystemExit):
        cli.main([])
