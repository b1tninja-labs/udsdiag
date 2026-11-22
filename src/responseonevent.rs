//! ResponseOnEvent (`0x86`): the eventType catalog and setup request building
//! (ISO 14229-1:2013, 9.10).
//!
//! ResponseOnEvent arms the server to send a chosen service's response whenever an event fires. The
//! request is `0x86 <eventType> <eventWindowTime> [eventTypeRecord] [serviceToRespondToRecord]`
//! (Table 97). In the eventType byte, bits 5-0 select the event (Table 99), bit 6 is the
//! `storageState` ([`with_store_event`]) and bit 7 is the suppressPosRspMsgIndicationBit.
//!
//! Each eventType fixes the length of its eventTypeRecord, and the control events
//! (stop / reportActivatedEvents / start / clear) take no serviceToRespondToRecord while the setup
//! events require one; [`build_request`] enforces both. The eventWindowTime byte is Annex B.2.

use crate::error::{Error, Result};

/// ResponseOnEvent service id.
pub const RESPONSE_ON_EVENT: u8 = 0x86;

/// eventType bit 6: storeEvent (persist the event across power cycles).
pub const STORE_EVENT: u8 = 0x40;

/// stopResponseOnEvent (`0x00`).
pub const STOP_RESPONSE_ON_EVENT: u8 = 0x00;
/// onDTCStatusChange (`0x01`).
pub const ON_DTC_STATUS_CHANGE: u8 = 0x01;
/// onTimerInterrupt (`0x02`).
pub const ON_TIMER_INTERRUPT: u8 = 0x02;
/// onChangeOfDataIdentifier (`0x03`).
pub const ON_CHANGE_OF_DATA_IDENTIFIER: u8 = 0x03;
/// reportActivatedEvents (`0x04`).
pub const REPORT_ACTIVATED_EVENTS: u8 = 0x04;
/// startResponseOnEvent (`0x05`).
pub const START_RESPONSE_ON_EVENT: u8 = 0x05;
/// clearResponseOnEvent (`0x06`).
pub const CLEAR_RESPONSE_ON_EVENT: u8 = 0x06;
/// onComparisonOfValues (`0x07`).
pub const ON_COMPARISON_OF_VALUES: u8 = 0x07;

/// One defined eventType (ISO 14229-1:2013, Table 99).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventType {
    /// The event type value (bits 5-0).
    pub value: u8,
    /// The event type name.
    pub name: &'static str,
    /// The event type mnemonic.
    pub mnemonic: &'static str,
    /// The fixed length (bytes) of this event's eventTypeRecord.
    pub event_type_record_len: u8,
    /// Whether this event requires a serviceToRespondToRecord (true for setup events).
    pub requires_service_to_respond_to: bool,
}

const fn event(
    value: u8,
    name: &'static str,
    mnemonic: &'static str,
    event_type_record_len: u8,
    requires_service_to_respond_to: bool,
) -> EventType {
    EventType {
        value,
        name,
        mnemonic,
        event_type_record_len,
        requires_service_to_respond_to,
    }
}

/// The eventType values defined by ISO 14229-1:2013 (Table 99).
pub const EVENT_TYPES: &[EventType] = &[
    event(
        STOP_RESPONSE_ON_EVENT,
        "stopResponseOnEvent",
        "STPROE",
        0,
        false,
    ),
    event(ON_DTC_STATUS_CHANGE, "onDTCStatusChange", "ONDTCS", 1, true),
    event(ON_TIMER_INTERRUPT, "onTimerInterrupt", "OTI", 1, true),
    event(
        ON_CHANGE_OF_DATA_IDENTIFIER,
        "onChangeOfDataIdentifier",
        "OCODID",
        2,
        true,
    ),
    event(
        REPORT_ACTIVATED_EVENTS,
        "reportActivatedEvents",
        "RAE",
        0,
        false,
    ),
    event(
        START_RESPONSE_ON_EVENT,
        "startResponseOnEvent",
        "STRTROE",
        0,
        false,
    ),
    event(
        CLEAR_RESPONSE_ON_EVENT,
        "clearResponseOnEvent",
        "CLRROE",
        0,
        false,
    ),
    event(
        ON_COMPARISON_OF_VALUES,
        "onComparisonOfValues",
        "OCOV",
        10,
        true,
    ),
];

/// Every defined eventType.
pub const fn event_types() -> &'static [EventType] {
    EVENT_TYPES
}

/// The defined eventType for a value (bits 5-0), or `None` when it is reserved/range.
pub fn get(value: u8) -> Option<EventType> {
    EVENT_TYPES.iter().copied().find(|e| e.value == value)
}

/// A label for an eventType value (bits 5-0), naming the range for an undefined one.
pub fn describe(value: u8) -> &'static str {
    if let Some(event) = get(value) {
        return event.name;
    }
    match value {
        0x20..=0x2F => "vehicleManufacturerSpecific",
        0x30..=0x3E => "systemSupplierSpecific",
        _ => "ISOSAEReserved",
    }
}

/// Set the `storageState` bit (bit 6) on an eventType so the event persists across power cycles.
#[must_use]
pub fn with_store_event(event_type: u8) -> u8 {
    event_type | STORE_EVENT
}

/// Build a ResponseOnEvent request PDU
/// (`0x86 <eventType> <eventWindowTime> [eventTypeRecord] [serviceToRespondToRecord]`).
///
/// `event_type` may carry the `storageState` bit (bit 6); the event is identified by bits 5-0.
///
/// # Errors
/// Returns an error when the event (bits 5-0) is not one of `0x00`-`0x07`, when `event_type_record`
/// is not that event's fixed length, or when the serviceToRespondToRecord presence does not match
/// the event (required for setup events, forbidden for control events).
pub fn build_request(
    event_type: u8,
    event_window_time: u8,
    event_type_record: &[u8],
    service_to_respond_to: &[u8],
) -> Result<Vec<u8>> {
    let base = event_type & 0x3F;
    let Some(event) = get(base) else {
        return Err(Error::new(format!(
            "not a defined eventType (0x00-0x07): {base:#04x}"
        )));
    };
    if event_type_record.len() != usize::from(event.event_type_record_len) {
        return Err(Error::new(format!(
            "{} needs an eventTypeRecord of {} byte(s)",
            event.name, event.event_type_record_len
        )));
    }
    match (
        event.requires_service_to_respond_to,
        service_to_respond_to.is_empty(),
    ) {
        (true, true) => {
            return Err(Error::new(format!(
                "{} requires a serviceToRespondToRecord",
                event.name
            )));
        }
        (false, false) => {
            return Err(Error::new(format!(
                "{} takes no serviceToRespondToRecord",
                event.name
            )));
        }
        _ => {}
    }
    let mut pdu = vec![RESPONSE_ON_EVENT, event_type, event_window_time];
    pdu.extend_from_slice(event_type_record);
    pdu.extend_from_slice(service_to_respond_to);
    Ok(pdu)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_types_match_table_99() {
        assert_eq!(event_types().len(), 8);
        assert_eq!(
            get(ON_CHANGE_OF_DATA_IDENTIFIER).unwrap().mnemonic,
            "OCODID"
        );
        assert_eq!(get(0x07).unwrap().event_type_record_len, 10);
        assert!(get(0x08).is_none());
        assert_eq!(describe(0x05), "startResponseOnEvent");
        assert_eq!(describe(0x25), "vehicleManufacturerSpecific");
        assert_eq!(describe(0x08), "ISOSAEReserved");
        assert_eq!(with_store_event(0x01), 0x41);
    }

    #[test]
    fn build_request_frames_and_validates() {
        // ISO 14229-1:2013 §9.10 example: onDTCStatusChange, infinite window (0x02),
        // eventTypeRecord = DTC status mask (1 byte), serviceToRespondTo = ReadDTCInformation 0x19 0x01.
        assert_eq!(
            build_request(ON_DTC_STATUS_CHANGE, 0x02, &[0x01], &[0x19, 0x01]).unwrap(),
            vec![0x86, 0x01, 0x02, 0x01, 0x19, 0x01]
        );
        // A control event (startResponseOnEvent) takes no record and no serviceToRespondTo.
        assert_eq!(
            build_request(START_RESPONSE_ON_EVENT, 0x02, &[], &[]).unwrap(),
            vec![0x86, 0x05, 0x02]
        );
        assert!(build_request(ON_DTC_STATUS_CHANGE, 0x02, &[], &[0x19, 0x01]).is_err()); // record len
        assert!(build_request(ON_DTC_STATUS_CHANGE, 0x02, &[0x01], &[]).is_err()); // needs STR
        assert!(build_request(START_RESPONSE_ON_EVENT, 0x02, &[], &[0x19]).is_err()); // control takes no STR
        assert!(build_request(0x08, 0x02, &[], &[]).is_err()); // reserved event
    }
}
