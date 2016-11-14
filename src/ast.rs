//! Defining an abstract syntax tree for bacnet APDUs

/// Defines the whole body of a BACnet APDU message
#[derive(Debug, PartialEq)]
pub enum ApduHeader {
    ConfirmedReq { 
        pdu_flags: u8, 
        max_segments: u8, 
        max_apdu: u8, 
        invoke_id: u8, 
        service: u8 
    },
    UnconfirmedReq { 
        service: u8 
    },
    SimpleAck { 
        invoke_id: u8, 
        service: u8 
    },
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

pub mod ObjectType {
    pub type Type = u16;
    pub const Device: Type = 8;
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
    ObjectId(ObjectType::Type, u32),
}

