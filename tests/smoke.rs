//! Integration tests use only the public API, as a downstream user would.

use udskit::{dtc, message, security, services};

#[test]
fn a_read_did_round_trips_through_the_codec() {
    // Build a ReadDataByIdentifier request and decode its positive response.
    let request = message::encode_request(0x22, &[0x00, 0x2F]);
    assert_eq!(request, vec![0x22, 0x00, 0x2F]);
    let response = message::decode_response(&[0x62, 0x00, 0x2F, 0x6C]).unwrap();
    let message::Response::Positive(positive) = response else {
        panic!("expected a positive response");
    };
    assert_eq!(positive.request_sid(), 0x22);
    assert_eq!(services::name_of(0x22), "ReadDataByIdentifier");
}

#[test]
fn the_status_decoder_is_public() {
    // ISO 14229-1:2013 Annex D.2: statusOfDTC 0x2F sets five bits.
    assert_eq!(dtc::decode_status(0x2F).len(), 5);
    assert_eq!(dtc::decode(&[0x0A, 0x7F]).unwrap(), "P0A7F");
}

#[test]
fn security_access_exposes_no_key_derivation() {
    // Only the requestSeed PDU is buildable in the core; there is no seed->key function to call.
    assert_eq!(
        security::build_request_seed(0x01).unwrap(),
        vec![0x27, 0x01]
    );
}

#[cfg(feature = "gm")]
#[test]
fn gm_vendor_layer_is_reachable() {
    use udskit::vendor::{VendorProfile, resolve_service};
    // The GM layer composes onto the ISO core through the vendor interface.
    assert_eq!(
        udskit::gm::GM.service(0xAE).unwrap().name,
        "DeviceControl (GMLAN)"
    );
    assert_eq!(
        resolve_service(0xAE, &[&udskit::gm::GM]).unwrap().name,
        "DeviceControl (GMLAN)"
    );
    // The GM reference catalogs are routed through the gm layer.
    assert_eq!(udskit::gm::dids::read_dids().len(), 115);
    assert!(udskit::gm::security::mechanism(udskit::gm::security::ADIT_ALD).is_ok());
}
