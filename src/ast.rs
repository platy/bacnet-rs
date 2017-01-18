//! Defining an abstract syntax tree for bacnet APDUs

use object;

/// Defines the whole body of a BACnet APDU message
#[derive(Debug, PartialEq)]
pub enum ApduHeader {
    /// BACnet Confirmed Request - Clause 20.1.2
    /// Transfers a variable length request to a service, which may be segmented, and expects an acknowledgement
    ConfirmedReq { 
        segmented: Option<SegmentInfo>,
        segmented_response_accepted: bool,
        max_segments: u8, 
        max_apdu: u8, 
        invoke_id: u8, 
        service: ServiceChoice,
    },
    /// BACnet Unconfirmed Request - Clause 20.1.3
    /// Transfers a request to a service
    UnconfirmedReq { 
        service: ServiceChoice,
    },
    /// BACnet Simple ACK - Clause 20.1.4
    /// Transfers a successful acknowledgement - with variable length content - from a service in response to a confirmed request to a service
    SimpleAck { 
        invoke_id: u8, 
        service: ServiceChoice,
    },
    /// BACnet Complex ACK - Clause 20.1.5
    /// Transfers a successful acknowledgement - with variable length content, which may be
    /// segmented, in response to a confrimed request to a service
    ComplexAck {
        segmented: Option<SegmentInfo>,
        invoke_id: u8,
        service: ServiceChoice,
    },
    /// BACnet Segment ACK - Clause 20.1.6
    /// Transfers acknowlegement of receipt of a segment, it may request retransmition of a segment
    /// or transmition of the next segments
    SegmentAck {
        negative_ack: bool,
        server: bool,
        invoke_id: ServiceChoice,
        sequence_number: u8,
        actual_window_size: u8,
    },
    /// BACnet Error PDU - Clause 20.1.7
    /// Transfers an error result from a confirmed service request, containing error data
    ErrorPdu {
        invoke_id: u8,
        error_choice: u8,
    },
    /// BACnet Reject PDU - Clause 20.1.8
    /// Transfers a rejection of a confirmed service request due to the request being invalid, the
    /// requested service has not been called to any effect
    RejectPdu {
        invoke_id: u8,
        reject_reason: u8,
    },
    /// BACnet Abort PDU - Clause 20.1.9
    /// Aborts a confirmed request between 2 peers
    AbortPdu {
        server: bool,
        invoke_id: u8,
        abort_reason: u8,
    },
}

type ServiceChoice = u8;

/// The fields which are present on message segments - they do not appear on unsegmented messages
/// TODO move into a segmentation module which generates these and maybe make the fields private
#[derive(Debug, PartialEq)]
pub struct SegmentInfo {
    pub more_follows: bool,
    pub sequence_number: u8,
    pub proposed_window_size: u8,
}

/// A sequence of BACnet values
pub type ValueSequence = Vec<SequenceableValue>;

/// Extracts a single value from the sequence with the provided context number
pub fn get_context_value<'a>(sequence: &'a ValueSequence, context_number: Context) -> Option<&'a PrimitiveValue> {
    for element in sequence.iter() {
        match element {
            &SequenceableValue::ContextValue(number, ref value) if number == context_number =>
                return Some(value),
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod test_get_context_value {
    use super::get_context_value;
    use super::PrimitiveValue::Boolean;
    use super::SequenceableValue::ContextValue;

    #[test]
    fn get_first_context_value() {
        assert_eq!(Some(&Boolean(true)),
            get_context_value(&vec!(ContextValue(4, Boolean(true))), 4));
    }

    #[test]
    fn get_second_context_value() {
        assert_eq!(Some(&Boolean(true)),
            get_context_value(&vec!(ContextValue(2, Boolean(false)), ContextValue(4, Boolean(true))), 4));
    }

    #[test]
    fn get_missing_context_value() {
        assert_eq!(None,
            get_context_value(&vec!(ContextValue(2, Boolean(false)), ContextValue(3, Boolean(true))), 4));
    }
}

/// The Bacnet types whih can be elements of a sequence
#[derive(Debug, PartialEq)]
pub enum SequenceableValue {
    ApplicationValue(PrimitiveValue),
    ContextValue(Context, PrimitiveValue),
    ContextValueSequence(Context, ValueSequence),
}

/// Context values have an id which has a meaning specific to the message it is within
pub type Context = u8;

pub mod object_type {
    pub const DEVICE: u16 = 8;
}

/// BACnet primitive application value types
#[derive(Debug, PartialEq)]
pub enum PrimitiveValue {
    Null,
    Boolean(bool),
    Unsigned(u32),
    Signed(i32),
    Real(f32),
    Double(f64),
    // OctetString([u8]), // not yey implemented as a working, non-ugly type signature will take
    // some thought
    CharacterString(String),
    // BitString([u8]), // not yet implemented as rust types dont have an efficeint representation of
    // this
    Enumerated(u32),
    // Date, // not yet implemented as they dont have close enough equvalents in rust
    // Time,
    ObjectId(object::ObjectId),
}

