//! CommunicationControl (`0x28`): controlType catalog, the communicationType byte, and request
//! building (ISO 14229-1:2013, 9.5; Annex B.1).
//!
//! CommunicationControl switches a server's normal and/or network-management message traffic on or
//! off. The request is `0x28 <controlType> <communicationType>`, with a 2-byte
//! `nodeIdentificationNumber` appended only when `controlType` is `0x04` or `0x05` (the enhanced
//! address-information variants). [`control_types`] / [`describe`] name the controlType values
//! (Table 54); [`communication_type`] composes the communicationType byte from a subnet selector
//! and a message-type selector (Annex B.1); [`build_request`] frames the PDU.

use crate::error::{Error, Result};

/// CommunicationControl service id.
pub const COMMUNICATION_CONTROL: u8 = 0x28;

/// enableRxAndTx (`0x00`).
pub const ENABLE_RX_AND_TX: u8 = 0x00;
/// enableRxAndDisableTx (`0x01`).
pub const ENABLE_RX_AND_DISABLE_TX: u8 = 0x01;
/// disableRxAndEnableTx (`0x02`).
pub const DISABLE_RX_AND_ENABLE_TX: u8 = 0x02;
/// disableRxAndTx (`0x03`).
pub const DISABLE_RX_AND_TX: u8 = 0x03;
/// enableRxAndDisableTxWithEnhancedAddressInformation (`0x04`).
pub const ENABLE_RX_AND_DISABLE_TX_WITH_ENHANCED_ADDRESS_INFORMATION: u8 = 0x04;
/// enableRxAndTxWithEnhancedAddressInformation (`0x05`).
pub const ENABLE_RX_AND_TX_WITH_ENHANCED_ADDRESS_INFORMATION: u8 = 0x05;

/// communicationType message selector (bits 1-0): normalCommunicationMessages.
pub const NORMAL_COMMUNICATION_MESSAGES: u8 = 0x01;
/// communicationType message selector (bits 1-0): networkManagementCommunicationMessages.
pub const NETWORK_MANAGEMENT_COMMUNICATION_MESSAGES: u8 = 0x02;
/// communicationType message selector (bits 1-0): both normal and network-management.
pub const NWM_AND_NORMAL_COMMUNICATION_MESSAGES: u8 = 0x03;

/// One defined controlType (ISO 14229-1:2013, Table 54).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlType {
    /// The sub-function value.
    pub value: u8,
    /// The controlType name.
    pub name: &'static str,
    /// The controlType mnemonic.
    pub mnemonic: &'static str,
}

const fn control(value: u8, name: &'static str, mnemonic: &'static str) -> ControlType {
    ControlType {
        value,
        name,
        mnemonic,
    }
}

/// The controlType values defined by ISO 14229-1:2013 (Table 54).
pub const CONTROL_TYPES: &[ControlType] = &[
    control(ENABLE_RX_AND_TX, "enableRxAndTx", "ERXTX"),
    control(ENABLE_RX_AND_DISABLE_TX, "enableRxAndDisableTx", "ERXDTX"),
    control(DISABLE_RX_AND_ENABLE_TX, "disableRxAndEnableTx", "DRXETX"),
    control(DISABLE_RX_AND_TX, "disableRxAndTx", "DRXTX"),
    control(
        ENABLE_RX_AND_DISABLE_TX_WITH_ENHANCED_ADDRESS_INFORMATION,
        "enableRxAndDisableTxWithEnhancedAddressInformation",
        "ERXDTXWEAI",
    ),
    control(
        ENABLE_RX_AND_TX_WITH_ENHANCED_ADDRESS_INFORMATION,
        "enableRxAndTxWithEnhancedAddressInformation",
        "ERXTXWEAI",
    ),
];

/// Every defined controlType.
pub const fn control_types() -> &'static [ControlType] {
    CONTROL_TYPES
}

/// The defined controlType for a value, or `None` for a reserved/range value.
pub fn get(value: u8) -> Option<ControlType> {
    CONTROL_TYPES.iter().copied().find(|c| c.value == value)
}

/// A label for any controlType value, naming the range for an undefined one.
pub fn describe(value: u8) -> &'static str {
    if let Some(control) = get(value) {
        return control.name;
    }
    match value {
        0x40..=0x5F => "vehicleManufacturerSpecific",
        0x60..=0x7E => "systemSupplierSpecific",
        _ => "ISOSAEReserved",
    }
}

/// Compose a communicationType byte (Annex B.1): the `subnet` selector in bits 7-4 (`0x0` = the
/// specified type on all connected networks, `0x1`-`0xE` = a specific subnet, `0xF` = the network
/// the request arrived on) and the message-type selector in bits 1-0 (one of the
/// `*_COMMUNICATION_MESSAGES` constants).
#[must_use]
pub fn communication_type(subnet: u8, messages: u8) -> u8 {
    ((subnet & 0x0F) << 4) | (messages & 0x03)
}

/// Build a CommunicationControl request PDU
/// (`0x28 <controlType> <communicationType> [nodeId hi, lo]`).
///
/// # Errors
/// Returns an error when `control_type` is not a usable sub-function value `0x00`-`0x7E`, when a
/// `node_id` is supplied for a non-enhanced controlType, or when an enhanced controlType
/// (`0x04`/`0x05`) is missing its required `node_id`.
pub fn build_request(
    control_type: u8,
    communication_type: u8,
    node_id: Option<u16>,
) -> Result<Vec<u8>> {
    if control_type > 0x7E {
        return Err(Error::new(format!(
            "not a controlType (0x00-0x7E): {control_type:#04x}"
        )));
    }
    let enhanced = control_type == ENABLE_RX_AND_DISABLE_TX_WITH_ENHANCED_ADDRESS_INFORMATION
        || control_type == ENABLE_RX_AND_TX_WITH_ENHANCED_ADDRESS_INFORMATION;
    match (enhanced, node_id) {
        (true, None) => {
            return Err(Error::new(
                "controlType 0x04/0x05 requires a nodeIdentificationNumber",
            ));
        }
        (false, Some(_)) => {
            return Err(Error::new(
                "nodeIdentificationNumber is only valid for controlType 0x04/0x05",
            ));
        }
        _ => {}
    }
    let mut pdu = vec![COMMUNICATION_CONTROL, control_type, communication_type];
    if let Some(node_id) = node_id {
        pdu.extend_from_slice(&node_id.to_be_bytes());
    }
    Ok(pdu)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_request_frames_control_and_communication_type() {
        // disableRxAndTx of normal messages on all networks.
        assert_eq!(
            build_request(
                DISABLE_RX_AND_TX,
                communication_type(0x0, NORMAL_COMMUNICATION_MESSAGES),
                None
            )
            .unwrap(),
            vec![0x28, 0x03, 0x01]
        );
        // enhanced variant appends the 2-byte nodeIdentificationNumber.
        assert_eq!(
            build_request(0x04, 0x01, Some(0x1234)).unwrap(),
            vec![0x28, 0x04, 0x01, 0x12, 0x34]
        );
        assert!(build_request(0x04, 0x01, None).is_err()); // enhanced needs a node id
        assert!(build_request(0x03, 0x01, Some(0x1234)).is_err()); // non-enhanced forbids it
        assert!(build_request(0x7F, 0x01, None).is_err()); // reserved / bit 7
    }

    #[test]
    fn control_types_match_table_54() {
        assert_eq!(control_types().len(), 6);
        assert_eq!(get(DISABLE_RX_AND_ENABLE_TX).unwrap().mnemonic, "DRXETX");
        assert_eq!(get(0x00).unwrap().name, "enableRxAndTx");
        assert!(get(0x06).is_none());
        assert_eq!(describe(0x50), "vehicleManufacturerSpecific");
        assert_eq!(describe(0x70), "systemSupplierSpecific");
        assert_eq!(describe(0x06), "ISOSAEReserved");
    }

    #[test]
    fn communication_type_byte_follows_annex_b1() {
        // subnet 0x1, both normal + network-management messages -> 0x13.
        assert_eq!(
            communication_type(0x1, NWM_AND_NORMAL_COMMUNICATION_MESSAGES),
            0x13
        );
        assert_eq!(communication_type(0x0, NORMAL_COMMUNICATION_MESSAGES), 0x01);
        assert_eq!(
            communication_type(0xF, NETWORK_MANAGEMENT_COMMUNICATION_MESSAGES),
            0xF2
        );
    }
}
