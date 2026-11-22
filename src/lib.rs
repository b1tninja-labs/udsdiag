//! `udskit`: an offline model of the UDS (ISO 14229-1:2013) and GM GMLAN diagnostic services.
//!
//! The crate carries the service and negative-response-code catalogs, a request/response PDU
//! codec, and the DID, DTC, Mode $06, RoutineControl, IOControl and SecurityAccess reference
//! catalogs drawn from an authorised assessment of a GM vehicle the owner holds. It encodes and
//! decodes diagnostic messages and looks identifiers up; it does not open a bus, derive a
//! SecurityAccess key, or write to a vehicle.
//!
//! Standard library only. There is no transport (the ISO-TP / ISO 15765-2 codec that segments a
//! multi-frame PDU is the sibling `isotpkit`), no CAN interface, and no SecurityAccess key
//! derivation, by design. Every protocol claim is anchored to a standard read through
//! `primarysource` and recorded in `docs/iso-map.md`.

pub mod accesstiming;
pub mod commcontrol;
pub mod did;
pub mod dtc;
pub mod dtcsetting;
pub mod dynamicdid;
pub mod ecureset;
pub mod error;
#[cfg(feature = "gm")]
pub mod gm;
pub mod gmlan;
pub mod iocontrol;
pub mod linkcontrol;
pub mod memory;
pub mod message;
pub mod mode06;
pub mod nrc;
pub mod periodic;
pub mod readdtc;
pub mod routine;
pub mod security;
pub mod services;
pub mod session;
pub mod subfunction;
pub mod testerpresent;
pub mod transfer;
pub mod vendor;

pub use error::{Error, Result};
pub use vendor::VendorProfile;
