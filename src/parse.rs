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

/// Should be called when the reader's next octet is the start of a tag.
///
/// # Errors
///
/// - if this is a context tag and there is no entry in the context for this tag
/// - if the application tag is unsupported / reserved
/// - if the length is too long
/// - if the value can't be parsed
/// - if the reader reaches the end of input before the parsing is complete
pub fn parse_sequenceable_value(reader: &mut Read, context: Context) -> Result<SequenceableValue, ParseError> {
    let (tag, class, tag_value) = try!(parse_tag(reader));

    if class {
        Ok(SequenceableValue::ContextValue(tag, try!(tag_to_value(reader, context(tag), tag_value))))
    } else {
        Ok(SequenceableValue::ApplicationValue(try!(tag_to_value(reader, tag, tag_value))))
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

    fn parse_array(data: &[u8]) -> Result<ast::SequenceableValue, ParseError> {
        let mut reader: &mut io::Read = &mut io::Cursor::new(data);
        parse_sequenceable_value(reader, context)
    }

    fn parsed_context_value_eq(data: &[u8], value: ast::SequenceableValue) {
        assert_eq!(value, parse_array(data).unwrap());
    }

//    #[test]
//    fn parse_context_wrapped_application_value() {
//        parsed_context_value_eq(&[0x3eu8, 0x21, 0x01, 0x3f], ContextValueSequence(3, vec!(ApplicationValue(PrimitiveValue::Unsigned(1)))));
//    } // TODO parse_tag needs to distinguish the 3 types of tag so that parse_sequenceable_value can do things properly

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
        assert_eq!(ast::SequenceableValue::ApplicationValue(value), parse_array(data).unwrap());
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

/// Call when the next octet is the start of a tag, advances the reader to after the tag and
/// returns the tag data in a tuple of (tag number, context class, value)
///
/// # Errors
/// - If the reader reaches the end before the parsing is complete
///
/// # Examples
/// Single octet application tag.
/// 
/// ```rust
/// use bacnet::parse::parse_tag;
/// use std::io::Read;
///
/// let mut data: &[u8] = &[0x24u8];
/// let mut reader: &mut Read = &mut data;
/// let tag = parse_tag(&mut reader).unwrap();
/// assert_eq!(tag, (2, false, 4));
/// ```
///
/// Extended tag number context tag.
/// 
/// ```rust
/// use bacnet::parse::parse_tag;
/// use std::io::Read;
///
/// let mut data: &[u8] = &[0xF9u8, 0x59];
/// let mut reader: &mut Read = &mut data;
/// let tag = parse_tag(&mut reader).unwrap();
/// assert_eq!(tag, (0x59, true, 1));
/// ```
///
/// Extended value (8-bit) application tag.
/// 
/// ```rust
/// use bacnet::parse::parse_tag;
/// use std::io::Read;
///
/// let mut data: &[u8] = &[0x05u8, 200];
/// let mut reader: &mut Read = &mut data;
/// let tag = parse_tag(&mut reader).unwrap();
/// assert_eq!(tag, (0, false, 200));
/// ```
///
/// Extended value (16-bit) application tag.
/// 
/// ```rust
/// use bacnet::parse::parse_tag;
/// use std::io::Read;
///
/// let mut data: &[u8] = &[0x05u8, 0xFE, 0x59, 0x59];
/// let mut reader: &mut Read = &mut data;
/// let tag = parse_tag(&mut reader).unwrap();
/// assert_eq!(tag, (0, false, 0x5959));
/// ```
///
/// Extended value (32-bit) application tag.
/// 
/// ```rust
/// use bacnet::parse::parse_tag;
/// use std::io::Read;
///
/// let mut data: &[u8] = &[0x05u8, 0xFF, 0x59, 0x59, 0x59, 0x59];
/// let mut reader: &mut Read = &mut data;
/// let tag = parse_tag(&mut reader).unwrap();
/// assert_eq!(tag, (0, false, 0x59595959));
/// ```
pub fn parse_tag(reader: &mut Read) -> Result<(u8, bool, u32), ParseError> {
    let first_byte = try!(read_one_byte(reader));
    let mut tag_num = (first_byte & 0xF0) >> 4;
    let class = (first_byte & 0x08) == 0x08;
    let mut value = (first_byte & 0x07) as u32;

    // Extended tag numbers
    if tag_num == 0xF {
        tag_num = try!(read_one_byte(reader));
    }
    // Extended values
    if value == 0x5 {
        value = try!(read_one_byte(reader)) as u32;
        if value == 0xFE {
            value = try!(read_unsigned(reader, 2));
        } else if value == 0xFF {
            value = try!(read_unsigned(reader, 4));
        }
    }
    Ok((tag_num, class, value as u32))
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

