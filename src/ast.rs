//! Defining an abstract syntax tree for bacnet APDUs

use std::string;

/// Defines the whole body of a BACnet APDU message
pub struct APDU {
    message_type: MessageType,
    body: ValueSequence,
}

/// Specifies a message type for a BACnet APDU
pub type MessageType = u8;

/// A sequence of BACnet values
pub type ValueSequence = Vec<SequenceableValue>;

/// The Bacnet types whih can be elements of a sequence
#[derive(Debug, PartialEq)]
pub enum SequenceableValue {
    ApplicationValue(PrimitiveValue),
    ContextValue(Context, PrimitiveValue),
    ContextValueSequence(Context, ValueSequence),
}

/// Context values have an id which has a meaning specific to the message it is within
pub type Context = u8;

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
    // ObjectId
}

