"""The obligation matrix and the code cannot drift apart silently."""

from __future__ import annotations

import importlib
import pathlib
import re

ROOT = pathlib.Path(__file__).resolve().parents[1]
ROW = re.compile(r"^\| `(ud\.[a-z0-9-]+\.[a-z0-9-]+)` \| (.*?) \| (.*?) \| `(\[[a-z ]*\])` \| (.*?) \|$")
CALLABLE = re.compile(r"`([a-z_0-9]+)\.([A-Za-z_0-9.]+)\(\)`")
DONE = {"[ok]"}
STATUSES = {"[ok]", "[part]", "[ ]", "[no]"}


def rows() -> list[tuple[str, str, str, str]]:
    out = []
    for line in (ROOT / "ROADMAP.md").read_text(encoding="utf-8").splitlines():
        match = ROW.match(line)
        if match:
            out.append((match.group(1), match.group(2), match.group(4), match.group(5)))
    return out


def test_the_matrix_has_rows_with_unique_ids():
    ids = [row[0] for row in rows()]
    assert len(ids) >= 8
    assert len(ids) == len(set(ids))


def test_every_status_is_in_the_legend():
    assert {row[2] for row in rows()} <= STATUSES


def test_every_named_callable_exists():
    missing = []
    for row_id, obligation, _status, _evidence in rows():
        match = CALLABLE.search(obligation)
        if not match:
            continue
        target: object = importlib.import_module("udsdiag." + match.group(1))
        for part in match.group(2).split("."):
            if not hasattr(target, part):
                missing.append(f"{row_id}: {match.group(1)}.{match.group(2)}")
                break
            target = getattr(target, part)
    assert not missing, missing


def test_a_row_claimed_done_names_a_test_that_exists():
    problems = []
    for row_id, obligation, status, evidence in rows():
        if status not in DONE:
            continue
        if not CALLABLE.search(obligation):
            problems.append(f"{row_id} is [ok] but names no callable")
        match = re.fullmatch(r"`(tests/[\w/]+\.py)::(\w+)`", evidence.strip())
        if not match:
            problems.append(f"{row_id} is [ok] without a path::test evidence cell: {evidence!r}")
            continue
        path = ROOT / match.group(1)
        if not path.is_file() or f"def {match.group(2)}(" not in path.read_text(encoding="utf-8"):
            problems.append(f"{row_id}: {match.group(1)} defines no {match.group(2)}()")
    assert not problems, problems
