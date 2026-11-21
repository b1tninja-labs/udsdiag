"""SecurityAccess (``0x27``): the level-pairing model and two mechanism descriptors.

SecurityAccess unlocks protected services. It is a two-step exchange: the client sends
requestSeed (an odd sub-function), the server returns a seed, the client sends sendKey (the
next even sub-function) with a key derived from the seed, and the server unlocks or answers
NRC ``0x35`` invalidKey (ISO 14229-1:2013, 9.4). A fixed relationship binds each requestSeed
to its sendKey: requestSeed ``0x01`` to sendKey ``0x02``, ``0x03`` to ``0x04``, and so on.

**Scope boundary.** This module models the *exchange shape* only. It contains no seed->key
algorithm, no key material, and no key-derivation method, and it will not compute a key. That
is out of scope by design (``docs/security-access.md``). :data:`MECHANISMS` describes the two
mechanisms the assessment distinguished — the classic GM seed/key (a per-module secret held
by GM) and the GMG3 head unit's ADIT ALD, an RSA-2048 signed challenge/response — as reference
descriptors, not as anything runnable.
"""

from __future__ import annotations

from dataclasses import dataclass, field

SECURITY_ACCESS = 0x27
REQUEST_SEED = "requestSeed"
SEND_KEY = "sendKey"


def is_request_seed(level: int) -> bool:
    """Whether a sub-function byte is a requestSeed (odd, ISO 14229-1:2013, 9.4.2.2)."""
    if not 0x01 <= level <= 0x7E:
        raise ValueError(f"security-access level out of range: {level:#04x}")
    return level % 2 == 1


def is_send_key(level: int) -> bool:
    """Whether a sub-function byte is a sendKey (even, the partner of a requestSeed)."""
    if not 0x01 <= level <= 0x7E:
        raise ValueError(f"security-access level out of range: {level:#04x}")
    return level % 2 == 0


def send_key_for(request_seed_level: int) -> int:
    """The sendKey sub-function paired with a requestSeed sub-function."""
    if not is_request_seed(request_seed_level):
        raise ValueError(f"not a requestSeed level: {request_seed_level:#04x}")
    return request_seed_level + 1


def request_seed_for(send_key_level: int) -> int:
    """The requestSeed sub-function paired with a sendKey sub-function."""
    if not is_send_key(send_key_level):
        raise ValueError(f"not a sendKey level: {send_key_level:#04x}")
    return send_key_level - 1


def build_request_seed(level: int) -> bytes:
    """Build a SecurityAccess requestSeed PDU (``0x27 <level>``)."""
    if not is_request_seed(level):
        raise ValueError(f"not a requestSeed level: {level:#04x}")
    return bytes([SECURITY_ACCESS, level])


@dataclass(frozen=True, slots=True)
class Mechanism:
    """A description of how a server verifies a SecurityAccess key. Reference only."""

    name: str
    family: str  # "seed-key" | "challenge-response"
    summary: str
    invalid_key_nrc: int = field(default=0x35)


# Reference descriptors (docs/security-access.md). Neither carries key material or an algorithm.
MECHANISMS: dict[str, Mechanism] = {
    "classic-gm-seed-key": Mechanism(
        name="Classic GM seed/key",
        family="seed-key",
        summary=(
            "A module returns a short seed; the tester returns a key computed from it by a "
            "per-module secret algorithm held by GM. The algorithm and keys are not in scope."
        ),
    ),
    "adit-ald": Mechanism(
        name="ADIT ALD (GMG3 head unit)",
        family="challenge-response",
        summary=(
            "SecurityAccess 0x27 is a front-end over the ADIT Access-Level Daemon. The server "
            "returns a 96-byte device-bound challenge (16-byte rand() nonce, 8-byte timestamp, "
            "8-byte i.MX6 serial, 16-byte caller ECU_ID). The tester returns an RSA-2048 "
            "signed block; the daemon verifies the signature against the per-level public key "
            "(level.pub) and memcmp's the response against the live challenge. A bad key is "
            "NRC 0x35. The per-level private keys are held by GM/Bosch and are out of scope."
        ),
    ),
}


def mechanism(key: str) -> Mechanism:
    """Look up a mechanism descriptor by key."""
    try:
        return MECHANISMS[key]
    except KeyError:
        raise ValueError(f"unknown security-access mechanism: {key!r}") from None
