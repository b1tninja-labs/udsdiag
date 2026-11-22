//! The vendor-extension interface.
//!
//! The crate core models ISO 14229-1. Manufacturers layer their own services and addressing on
//! top of UDS (GM GMLAN is the one this assessment recovered), so those live behind a
//! [`VendorProfile`] rather than being baked into the ISO catalog. A consumer composes the ISO
//! core with zero or more vendor profiles; the core never depends on a vendor.
//!
//! The first profile is [`crate::gm::Gm`]. A profile contributes its vendor-specific services and
//! its request→response / request→filter addressing; richer reference data (routines, IO signals,
//! SecurityAccess mechanisms, the DID set) is GM-sourced too and is being routed through this
//! interface incrementally — see `ROADMAP.md`.

use crate::services::Service;

/// A manufacturer layer on top of the ISO 14229-1 core.
pub trait VendorProfile {
    /// A human label for the vendor layer (e.g. `"GM GMLAN"`).
    fn name(&self) -> &'static str;

    /// The vendor-specific services this layer adds to the ISO catalog.
    fn services(&self) -> &'static [Service];

    /// The vendor's physical response CAN id for a request id, or `None` when it does not apply.
    fn response_id(&self, request_id: u16) -> Option<u16>;

    /// The vendor's flow/acceptance filter id for a request id, or `None` when it does not apply.
    fn filter_id(&self, request_id: u16) -> Option<u16>;

    /// The vendor service for a request SID, or `None` when this layer does not define it.
    fn service(&self, sid: u8) -> Option<Service> {
        self.services().iter().copied().find(|s| s.sid == sid)
    }
}

/// Look a SID up in the ISO core first, then in each vendor profile in order.
pub fn resolve_service(sid: u8, vendors: &[&dyn VendorProfile]) -> Option<Service> {
    if let Some(service) = crate::services::get(sid) {
        return Some(service);
    }
    vendors.iter().find_map(|vendor| vendor.service(sid))
}

#[cfg(all(test, feature = "gm"))]
mod tests {
    use super::*;
    use crate::gm::Gm;

    #[test]
    fn resolve_searches_core_then_vendors() {
        let vendors: &[&dyn VendorProfile] = &[&Gm];
        // An ISO service resolves from the core.
        assert_eq!(
            resolve_service(0x22, vendors).unwrap().name,
            "ReadDataByIdentifier"
        );
        // A GMLAN service resolves from the vendor layer, not the core.
        assert!(crate::services::get(0xAE).is_none());
        assert_eq!(
            resolve_service(0xAE, vendors).unwrap().name,
            "DeviceControl (GMLAN)"
        );
        // Unknown everywhere.
        assert!(resolve_service(0x05, vendors).is_none());
    }
}
