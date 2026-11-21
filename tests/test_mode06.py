from __future__ import annotations

from udsdiag import mode06


def test_volt_is_group_1a():
    assert mode06.VOLT_OBD_GROUP == "1A"
    volt = mode06.volt_files()
    names = {doc.filename for doc in volt}
    assert "17OBDG01 Hybrid Diagnostics.pdf" in names
    assert "GM CAN mode $06 data final_dm.pdf" in names
    # the Group 2 hybrid file is not a Volt file
    assert "17OBDG02 Hybrid Diagnostics.pdf" not in names


def test_files_for_group_include_generic():
    group3 = mode06.files_for_group("3")
    names = {doc.filename for doc in group3}
    assert "17OBDG03 Engine Diagnostics.pdf" in names
    # generic (no obd_groups) files apply to every group
    assert "GM CAN mode $06 data final_dm.pdf" in names
    assert "2017_OBD_GROUP_SUMMARY.pdf" in names
    assert len(mode06.documents()) >= 10
