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

pub type Context = fn(u8) -> u8;

/// Should be called when the reader's next octet is the start of a tag. Returns none if the parsed
/// value is a close tag
///
/// # Errors
///
/// - if this is a context tag and there is no entry in the context for this tag
/// - if the application tag is unsupported / reserved
/// - if the length is too long
/// - if the value can't be parsed
/// - if the reader reaches the end of input before the parsing is complete
pub fn parse_sequenceable_value(reader: &mut Read, context: Context) -> Result<Option<SequenceableValue>, ParseError> {
    match try!(parse_tag(reader)) {
        Tag::Application(tag, tag_value) => 
            Ok(Some(SequenceableValue::ApplicationValue(try!(tag_to_value(reader, tag, tag_value))))),
        Tag::Context(tag, tag_value) => 
            Ok(Some(SequenceableValue::ContextValue(tag, try!(tag_to_value(reader, context(tag), tag_value))))),
        Tag::Close(_) =>
            Ok(None),
        Tag::Open(tag) => {
            let mut list = vec!();
            while let Some(child) = try!(parse_sequenceable_value(reader, context)) {
                list.push(child);
            }
            Ok(Some(SequenceableValue::ContextValueSequence(tag, list)))
        },
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
        loop {
            return match reader.read(&mut buf) {
            Ok(0) => Err(ParseError::InputEndedBeforeParsingCompleted),
            Ok(..) => Ok(buf[0]),
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => Err(ParseError::ReadError(e)),
        };
    }
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
