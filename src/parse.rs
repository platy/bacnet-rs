use ast;
use ast::SequenceableValue;
use std::io;
use std::io::Read;


#[derive(Debug)]
pub enum ParseError {
    ReadError(io::Error),
    InputEndedBeforeParsingCompleted, // "Input ended before parsing completed"
    ValueSizeNotSupported, // such as an 8byte integer
    NotImplemented(&'static str),
}

impl PartialEq for ParseError {
    fn eq(&self, other: &ParseError) -> bool {
        match (self, other) {
            (&ParseError::ReadError(_), &ParseError::ReadError(_)) => true,
            (&ParseError::InputEndedBeforeParsingCompleted, &ParseError::InputEndedBeforeParsingCompleted) => true,
            (&ParseError::ValueSizeNotSupported, &ParseError::ValueSizeNotSupported) => true,
            (&ParseError::NotImplemented(string1), &ParseError::NotImplemented(string2)) => string1 == string2,
            _ => false
        }
    }
}

pub fn parse_apdu_header(reader: &mut Read) -> Result<ast::ApduHeader, ParseError> {
    use ast::ApduHeader;

    let first_byte = try!(read_one_byte(reader));
    match ((first_byte & 0xF0u8) >> 4, first_byte & 0x0Fu8) {
        (0, pdu_flags) => {
            let second_byte = try!(read_one_byte(reader));
            Ok(ApduHeader::ConfirmedReq { 
                pdu_flags: pdu_flags,
                max_segments: (second_byte >> 4) & 0b111,
                max_apdu: second_byte & 0x0F,
                invoke_id: try!(read_one_byte(reader)),
                service: try!(read_one_byte(reader)),
            })
        },
        (1, _) =>
            Ok(ApduHeader::UnconfirmedReq {
                service: try!(read_one_byte(reader)),
            }),
        (2, _) =>
            Ok(ApduHeader::SimpleAck {
                invoke_id: try!(read_one_byte(reader)),
                service: try!(read_one_byte(reader)),
            }),
        _ => Err(ParseError::NotImplemented("")),
    }
}

#[cfg(test)]
mod test_apdu_header_parse {
    use super::parse_apdu_header;
    use super::ParseError;
    use ast::ApduHeader;
    use std::io;

    fn parse_array(data: &[u8]) -> Result<ApduHeader, ParseError> {
        let mut reader: &mut io::Read = &mut io::Cursor::new(data);
        parse_apdu_header(reader)
    }

    #[test]
    fn parse_unconfirmed() {
        assert_eq!(Ok(ApduHeader::ConfirmedReq {
            pdu_flags: 0b0000,
            max_segments: 0x0,
            max_apdu: 5,
            invoke_id: 1,
            service: 15,
        }), parse_array(&[0u8, 5, 1, 15]));
    }

    #[test]
    fn parse_confirmed() {
        assert_eq!(Ok(ApduHeader::UnconfirmedReq {
            service: 8,
        }), parse_array(&[0x10u8, 8]));
    }

    #[test]
    fn parse_simple_ack() {
        assert_eq!(Ok(ApduHeader::SimpleAck {
            invoke_id: 1,
            service: 15,
        }), parse_array(&[0x20u8, 1, 15]));
    }
}

pub type Context = fn(u8) -> u8;

pub fn parse_value_sequence_to_end(reader: &mut Read, context: Context) -> Result<ast::ValueSequence, ParseError> {
    let mut list = vec!();
    while let Some(child) = try!(parse_sequenceable_value(reader, context)) {
        list.push(child);
    }
    Ok(list)
}

#[cfg(test)]
mod test_value_sequence_parse {
    use super::ParseError;
    use ast;
    use ast::PrimitiveValue;
    use ast::SequenceableValue::ContextValue;
    use ast::SequenceableValue::ApplicationValue;
    use std::io;
    use super::parse_value_sequence_to_end;

    fn context(context_tag: u8) -> u8 {
        context_tag - 1 // this gives us access to all the types for testing in a simple way
    }

    fn parse_array(data: &[u8]) -> Result<ast::ValueSequence, ParseError> {
        let mut reader: &mut io::Read = &mut io::Cursor::new(data);
        parse_value_sequence_to_end(reader, context)
    }

    fn parsed_value_sequence_eq(data: &[u8], value: ast::ValueSequence) {
        assert_eq!(Ok(value), parse_array(data));
    }

    #[test]
    fn parse_empty_value_sequence() {
        parsed_value_sequence_eq(&[], vec!())
    }

    #[test]
    fn parse_basic_value_sequence() {
        parsed_value_sequence_eq(&[0x22u8, 0x99, 0x88, 0x28], vec!(ApplicationValue(PrimitiveValue::Unsigned(0x9988)), ContextValue(2, PrimitiveValue::Boolean(false))))
    }
}

/// Should be called when the reader's next octet is the start of a tag. Returns none if the parsed
/// value is a close tag, or if the reader is at the end to start with.
///
/// # Errors
///
/// - if this is a context tag and there is no entry in the context for this tag
/// - if the application tag is unsupported / reserved
/// - if the length is too long
/// - if the value can't be parsed
/// - if the reader reaches the end of input before the parsing is complete
pub fn parse_sequenceable_value(reader: &mut Read, context: Context) -> Result<Option<SequenceableValue>, ParseError> {
    match parse_tag(reader) {
        Ok(Tag::Application(tag, tag_value)) => 
            Ok(Some(SequenceableValue::ApplicationValue(try!(tag_to_value(reader, tag, tag_value))))),
        Ok(Tag::Context(tag, tag_value)) => 
            Ok(Some(SequenceableValue::ContextValue(tag, try!(tag_to_value(reader, context(tag), tag_value))))),
        Ok(Tag::Close(_)) =>
            Ok(None),
        Ok(Tag::Open(tag)) => {
            let mut list = vec!();
            while let Some(child) = try!(parse_sequenceable_value(reader, context)) {
                list.push(child);
            }
            Ok(Some(SequenceableValue::ContextValueSequence(tag, list)))
        },
        Err(ParseError::InputEndedBeforeParsingCompleted) =>
            Ok(None),
        Err(other) =>
            Err(other),
    }
}

fn tag_to_value(reader: &mut Read, tag: u8, tag_value: u32) -> Result<ast::PrimitiveValue, ParseError> {
    use ast::PrimitiveValue;

    match tag {
        0 => Ok(PrimitiveValue::Null),
        1 => Ok(PrimitiveValue::Boolean(tag_value != 0)),
        2 => Ok(PrimitiveValue::Unsigned(try!(read_unsigned(reader, tag_value as usize)))),
        _ => Err(ParseError::NotImplemented("Some tag")),
    }
}

#[cfg(test)]
mod parse_sequenceable_value_tests {
    use super::ParseError;
    use super::parse_sequenceable_value;
    use ast;
    use ast::PrimitiveValue;
    use ast::SequenceableValue::ContextValue;
    use ast::SequenceableValue::ApplicationValue;
    use ast::SequenceableValue::ContextValueSequence;
    use std::io;

    fn context(context_tag: u8) -> u8 {
        context_tag - 1 // this gives us access to all the types for testing in a simple way
    }

    fn parse_array(data: &[u8]) -> Result<Option<ast::SequenceableValue>, ParseError> {
        let mut reader: &mut io::Read = &mut io::Cursor::new(data);
        parse_sequenceable_value(reader, context)
    }

    fn parsed_context_value_eq(data: &[u8], value: ast::SequenceableValue) {
        assert_eq!(value, parse_array(data).unwrap().unwrap());
    }

    #[test]
    fn parse_empty_reader() {
        assert_eq!(Ok(None), parse_array(&[]));
    }

    #[test]
    fn parse_context_wrapped_application_value() {
        parsed_context_value_eq(&[0x3eu8, 0x21, 0x01, 0x3f], ContextValueSequence(3, vec!(ApplicationValue(PrimitiveValue::Unsigned(1)))));
    }

    #[test]
    fn parse_context_sequence_of_application_values() {
        parsed_context_value_eq(&[0x5eu8, 0x21, 0x01, 0x21, 0x02, 0x21, 0x03, 0x5f], ContextValueSequence(5, vec!(ApplicationValue(PrimitiveValue::Unsigned(1)), ApplicationValue(PrimitiveValue::Unsigned(2)), ApplicationValue(PrimitiveValue::Unsigned(3)))));
    }

    #[test]
    fn parse_context_null() {
        parsed_context_value_eq(&[0x18u8], ContextValue(1, PrimitiveValue::Null));
    }

    #[test]
    fn parse_context_boolean() {
        parsed_context_value_eq(&[0x28u8], ContextValue(2, PrimitiveValue::Boolean(false)));
        parsed_context_value_eq(&[0x29u8], ContextValue(2, PrimitiveValue::Boolean(true)));
    }

    #[test]
    fn parse_context_unsigned() {
        use ast::PrimitiveValue::Unsigned;
        parsed_context_value_eq(&[0x39u8, 200], ContextValue(3, Unsigned(200)));
        parsed_context_value_eq(&[0x3Au8, 0x99, 0x88], ContextValue(3, Unsigned(0x9988)));
        parsed_context_value_eq(&[0x3Bu8, 0x99, 0x88, 0x77], ContextValue(3, Unsigned(0x998877)));
        parsed_context_value_eq(&[0x3Cu8, 0x99, 0x88, 0x77, 0x66], ContextValue(3, Unsigned(0x99887766)));
        // length > 4 not supported
        assert_eq!(ParseError::ValueSizeNotSupported, parse_array(&[0x3du8, 5]).unwrap_err());
    }

    fn parsed_application_value_eq(data: &[u8], value: PrimitiveValue) {
        assert_eq!(ast::SequenceableValue::ApplicationValue(value), parse_array(data).unwrap().unwrap());
    }

    #[test]
    fn parse_null() {
        parsed_application_value_eq(&[0x00u8], PrimitiveValue::Null);
    }

    #[test]
    fn parse_boolean() {
        parsed_application_value_eq(&[0x10u8], PrimitiveValue::Boolean(false));
        parsed_application_value_eq(&[0x11u8], PrimitiveValue::Boolean(true));
    }

    #[test]
    fn parse_unsigned() {
        use ast::PrimitiveValue::Unsigned;
        parsed_application_value_eq(&[0x21u8, 200], Unsigned(200));
        parsed_application_value_eq(&[0x22u8, 0x99, 0x88], Unsigned(0x9988));
        parsed_application_value_eq(&[0x23u8, 0x99, 0x88, 0x77], Unsigned(0x998877));
        parsed_application_value_eq(&[0x24u8, 0x99, 0x88, 0x77, 0x66], Unsigned(0x99887766));
        // length > 4 not supported
        assert_eq!(ParseError::ValueSizeNotSupported, parse_array(&[0x25u8, 5]).unwrap_err());
    }
    // TODO tests for all the error types
}

#[derive(PartialEq, Debug)]
enum Tag {
    Application(u8, u32),
    Context(u8, u32),
    Open(u8),
    Close(u8),
}

/// Call when the next octet is the start of a tag, advances the reader to after the tag and
/// returns the Tag enum
///
/// # Errors
/// - If the reader reaches the end before the parsing is complete
///
/// # Description of encoding
///
/// BACnet encoding encodes meta data into a preceding octet.
/// BACnet has 2 classes of tags, context tags and application tags. The class is
/// identifed by the 5th bit of the tag octet, a `1` indicating a context tag.
/// The first 4 bits of a context tag are the context number - a context for the current production
/// should then specify the purpose and the primitive type of the value.
/// The first 4 bits of an application tag are the type number - identifiying the type of the
/// value.
/// The use of the last 3 bits of the tag depend on the type (identified either by the type field or
/// by the context identified by the context field). For a boolean the value can be encoded
/// directly into the tag, for a named tag, the last 3 bits identify the name, in general the last
/// 3 bits identify the length of the value in octets.
/// PD open / close tags are types of named context tags - their use must be identified by their
/// context number and they will be encoded by an 0xE (for open) and 0xF (for close). Their purpose
/// is to either allow for a context value with any type (the value would be specified as an
/// application tag inside), to specify a contructed type (where the production inside would have a
/// new context), or to group a sequence of values.
///
/// ## Extending type and length fields
///
/// The type field can be extended into another octet by using 0xF in the type field, then the next
/// octet becomes the type field and type / context values 0-254 (255 is reserved).
///
/// The Length field can be extended by using 0b111 in the length field, the next octet becomes a
/// length field, if the next octet is 0xFE, the next 2 octets become a length field, if it is 0xFF
/// then the next 4 octets become a length field. This encoding allows lengths up to 2^32-1.
fn parse_tag(reader: &mut Read) -> Result<Tag, ParseError> {
    let first_byte = try!(read_one_byte(reader));
    let mut tag_num = (first_byte & 0xF0) >> 4;
    let class = (first_byte & 0x08) == 0x08;
    let mut value = (first_byte & 0x07) as u32;

    // Extended tag numbers
    if tag_num == 0xF {
        tag_num = try!(read_one_byte(reader));
    }
    // Open / Close tags
    if value == 0x6 {
        Ok(Tag::Open(tag_num))
    } else if value == 0x7 {
        Ok(Tag::Close(tag_num))
    } // Extended values
    else {
        if value == 0x5 {
            value = try!(read_one_byte(reader)) as u32;
            if value == 0xFE {
                value = try!(read_unsigned(reader, 2));
            } else if value == 0xFF {
                value = try!(read_unsigned(reader, 4));
            }
        }
        Ok(
            if class {
                Tag::Context(tag_num, value)
            } else {
                Tag::Application(tag_num, value)
            }
        )
    }
}

#[cfg(test)]
mod test_read_tag {
    use super::parse_tag;
    use super::Tag;
    use std::io::Read;
   
    #[test]
    fn open() {
        let mut data: &[u8] = &[0x2eu8];
        let mut reader: &mut Read = &mut data;
        let tag = parse_tag(&mut reader).unwrap();
        assert_eq!(Tag::Open(2), tag);
    }

    #[test]
    fn close() {
        let mut data: &[u8] = &[0x2fu8];
        let mut reader: &mut Read = &mut data;
        let tag = parse_tag(&mut reader).unwrap();
        assert_eq!(Tag::Close(2), tag);
    }

    #[test]
    fn application0() {
        let mut data: &[u8] = &[0x24u8];
        let mut reader: &mut Read = &mut data;
        let tag = parse_tag(&mut reader).unwrap();
        assert_eq!(Tag::Application(2, 4), tag);
    }

    /// Extended tag number context tag.
    #[test]
    fn context1() {
        let mut data: &[u8] = &[0xF9u8, 0x59];
        let mut reader: &mut Read = &mut data;
        let tag = parse_tag(&mut reader).unwrap();
        assert_eq!(Tag::Context(0x59, 1), tag);
    }
    
    /// Extended value (8-bit) application tag.
    #[test]
    fn application1() {
        let mut data: &[u8] = &[0x05u8, 200];
        let mut reader: &mut Read = &mut data;
        let tag = parse_tag(&mut reader).unwrap();
        assert_eq!(Tag::Application(0, 200), tag);
    }
    
    /// Extended value (16-bit) application tag.
    #[test]
    fn application2() {
        let mut data: &[u8] = &[0x05u8, 0xFE, 0x59, 0x59];
        let mut reader: &mut Read = &mut data;
        let tag = parse_tag(&mut reader).unwrap();
        assert_eq!(Tag::Application(0, 0x5959), tag);
    }
    
    /// Extended value (32-bit) application tag.
    #[test]
    fn application32 () {
        let mut data: &[u8] = &[0x05u8, 0xFF, 0x59, 0x59, 0x59, 0x59];
        let mut reader: &mut Read = &mut data;
        let tag = parse_tag(&mut reader).unwrap();
        assert_eq!(Tag::Application(0, 0x59595959), tag);
    }
}

// Read an unsigned integer of the specified number of bytes
fn read_unsigned(reader: &mut Read, size: usize) -> Result<u32, ParseError> {
    if size > 4 {
        return Err(ParseError::ValueSizeNotSupported)
    }
    let mut value: u32 = 0;
    for i in (0..size).rev() {
        let next_byte = try!(read_one_byte(reader)) as u32;
        value = value | (next_byte << (8 * i));
    }
    Ok(value)
}

fn read_one_byte(reader: &mut Read) -> Result<u8, ParseError> {
    let mut buf = [0];
    let ret;
    loop {
        match reader.read(&mut buf) {
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {},
            Err(e) => {
                ret = Err(ParseError::ReadError(e));
                break;
            },
            Ok(0) => {
                ret = Err(ParseError::InputEndedBeforeParsingCompleted);
                break;
            },
            Ok(..) => {
                ret = Ok(buf[0]);
                break;
            },
        };
    }
    ret
}

#[test]
fn test_read_unsigned_16() {
    let mut data: &[u8] = &[0x99u8, 0x11];
    let mut reader: &mut Read = &mut data;
    assert_eq!(read_unsigned(reader, 2).unwrap(), 0x9911);
}

#[test]
fn test_read_unsigned_32() {
    let mut data: &[u8] = &[0x22u8, 0x33, 0x44, 0x55];
    let mut reader: &mut Read = &mut data;
    assert_eq!(read_unsigned(reader, 4).unwrap(), 0x22334455);
}
