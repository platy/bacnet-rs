use ast;
use ast::SequenceableValue;


// #[derive(Debug)]
// pub enum ParseError {
//     WriteError(io::Error),
//     InputEndedBeforeParsingCompleted, // "Input ended before parsing completed"
//     ValueSizeNotSupported, // such as an 8byte integer
//     NotImplemented(&'static str),
// }
// 
// impl PartialEq for ParseError {
//     fn eq(&self, other: &ParseError) -> bool {
//         match (self, other) {
//             (&ParseError::WriteError(_), &ParseError::WriteError(_)) => true,
//             (&ParseError::InputEndedBeforeParsingCompleted, &ParseError::InputEndedBeforeParsingCompleted) => true,
//             (&ParseError::ValueSizeNotSupported, &ParseError::ValueSizeNotSupported) => true,
//             (&ParseError::NotImplemented(string1), &ParseError::NotImplemented(string2)) => string1 == string2,
//             _ => false
//         }
//     }
// }
// 
// pub fn write_apdu_header(writer: &mut Write) -> Result<ast::ApduHeader, ParseError> {
//     use ast::ApduHeader;
// 
//     let first_byte = try!(write_one_byte(writer));
//     match ((first_byte & 0xF0u8) >> 4, first_byte & 0x0Fu8) {
//         (0, pdu_flags) => {
//             let second_byte = try!(write_one_byte(writer));
//             Ok(ApduHeader::ConfirmedReq { 
//                 pdu_flags: pdu_flags,
//                 max_segments: (second_byte >> 4) & 0b111,
//                 max_apdu: second_byte & 0x0F,
//                 invoke_id: try!(write_one_byte(writer)),
//                 service: try!(write_one_byte(writer)),
//             })
//         },
//         (1, _) =>
//             Ok(ApduHeader::UnconfirmedReq {
//                 service: try!(write_one_byte(writer)),
//             }),
//         (2, _) =>
//             Ok(ApduHeader::SimpleAck {
//                 invoke_id: try!(write_one_byte(writer)),
//                 service: try!(write_one_byte(writer)),
//             }),
//         _ => Err(ParseError::NotImplemented("")),
//     }
// }
// 
// #[cfg(test)]
// mod test_apdu_header_write {
//     use super::write_apdu_header;
//     use super::ParseError;
//     use ast::ApduHeader;
//     use std::io;
// 
//     fn write_array(data: &[u8]) -> Result<ApduHeader, ParseError> {
//         let mut writer: &mut io::Write = &mut io::Cursor::new(data);
//         write_apdu_header(writer)
//     }
// 
//     #[test]
//     fn write_unconfirmed() {
//         assert_eq!(Ok(ApduHeader::ConfirmedReq {
//             pdu_flags: 0b0000,
//             max_segments: 0x0,
//             max_apdu: 5,
//             invoke_id: 1,
//             service: 15,
//         }), write_array(&[0u8, 5, 1, 15]));
//     }
// 
//     #[test]
//     fn write_confirmed() {
//         assert_eq!(Ok(ApduHeader::UnconfirmedReq {
//             service: 8,
//         }), write_array(&[0x10u8, 8]));
//     }
// 
//     #[test]
//     fn write_simple_ack() {
//         assert_eq!(Ok(ApduHeader::SimpleAck {
//             invoke_id: 1,
//             service: 15,
//         }), write_array(&[0x20u8, 1, 15]));
//     }
// }
// 
// pub type Context = fn(u8) -> u8;
// 
// pub fn write_value_sequence_to_end(writer: &mut Write, context: Context) -> Result<ast::ValueSequence, ParseError> {
//     let mut list = vec!();
//     while let Some(child) = try!(write_sequenceable_value(writer, context)) {
//         list.push(child);
//     }
//     Ok(list)
// }
// 
// #[cfg(test)]
// mod test_value_sequence_write {
//     use super::ParseError;
//     use ast;
//     use ast::PrimitiveValue;
//     use ast::SequenceableValue::ContextValue;
//     use ast::SequenceableValue::ApplicationValue;
//     use std::io;
//     use super::write_value_sequence_to_end;
// 
//     fn context(context_tag: u8) -> u8 {
//         context_tag - 1 // this gives us access to all the types for testing in a simple way
//     }
// 
//     fn write_array(data: &[u8]) -> Result<ast::ValueSequence, ParseError> {
//         let mut writer: &mut io::Write = &mut io::Cursor::new(data);
//         write_value_sequence_to_end(writer, context)
//     }
// 
//     fn written_value_sequence_eq(data: &[u8], value: ast::ValueSequence) {
//         assert_eq!(Ok(value), write_array(data));
//     }
// 
//     #[test]
//     fn write_empty_value_sequence() {
//         written_value_sequence_eq(&[], vec!())
//     }
// 
//     #[test]
//     fn write_basic_value_sequence() {
//         written_value_sequence_eq(&[0x22u8, 0x99, 0x88, 0x28], vec!(ApplicationValue(PrimitiveValue::Unsigned(0x9988)), ContextValue(2, PrimitiveValue::Boolean(false))))
//     }
// }

/// Call to write a sequenceable value to the writer, the tag for the value is written first
/// followed by the encoded value
pub fn write_sequenceable_value(writer: &mut Vec<u8>, value: SequenceableValue) {
    match value {
        SequenceableValue::ContextValue(context, value) => {
            let (lvt, content, _) = primitive_value_to_tag_value(value);
            write_tag(writer, Tag::Context(context, lvt));
            writer.extend(content);
        },
        SequenceableValue::ApplicationValue(value) => {
            let (lvt, content, primitive_type) = primitive_value_to_tag_value(value);
            write_tag(writer, Tag::Application(primitive_type, lvt));
            writer.extend(content);
        },
        SequenceableValue::ContextValueSequence(context, list) => {
            write_tag(writer, Tag::Open(context));
            for e in list {
                write_sequenceable_value(writer, e);
            }
           write_tag(writer, Tag::Close(context)); 
        },
    };
}

/// Call to write a PrimitiveValue and return the lvt portion of the tag
fn primitive_value_to_tag_value(value: ast::PrimitiveValue) -> (u32, Vec<u8>, u8) {
    use ast::PrimitiveValue;

    match value {
        PrimitiveValue::Null => (0, vec![], 0),
        PrimitiveValue::Boolean(b) => (b as u32, vec![], 1),
        PrimitiveValue::Unsigned(u) => {
            let mut uv: Vec<u8> = vec![u as u8];
            let mut t = u >> 8;
            println!("{} {}", u, t);
            while t > 0 {
                uv.push(t as u8);
                t = t >> 8;
            }
            uv.reverse();
            (uv.len() as u32, uv, 2)
        },
        _ => panic!("Not implemented"),
    }
}

#[cfg(test)]
mod write_sequenceable_value_tests {
    use super::write_sequenceable_value;
    use ast;
    use ast::PrimitiveValue;
    use ast::SequenceableValue;
    use ast::SequenceableValue::ContextValue;
    use ast::SequenceableValue::ApplicationValue;
    use ast::SequenceableValue::ContextValueSequence;

    fn write_array(sequenceable_value: SequenceableValue) -> Vec<u8> {
        let mut writer = Vec::new();
        write_sequenceable_value(&mut writer, sequenceable_value);
        writer
    }

    fn written_context_value_eq(data: &[u8], value: ast::SequenceableValue) {
        assert_eq!(data.to_vec(), write_array(value));
    }

    #[test]
    fn write_context_wrapped_application_value() {
        written_context_value_eq(&[0x3eu8, 0x21, 0x01, 0x3f], ContextValueSequence(3, vec!(ApplicationValue(PrimitiveValue::Unsigned(1)))));
    }

    #[test]
    fn write_context_sequence_of_application_values() {
        written_context_value_eq(&[0x5eu8, 0x21, 0x01, 0x21, 0x02, 0x21, 0x03, 0x5f], ContextValueSequence(5, vec!(ApplicationValue(PrimitiveValue::Unsigned(1)), ApplicationValue(PrimitiveValue::Unsigned(2)), ApplicationValue(PrimitiveValue::Unsigned(3)))));
    }

    #[test]
    fn write_context_null() {
        written_context_value_eq(&[0x18u8], ContextValue(1, PrimitiveValue::Null));
    }

    #[test]
    fn write_context_boolean() {
        written_context_value_eq(&[0x28u8], ContextValue(2, PrimitiveValue::Boolean(false)));
        written_context_value_eq(&[0x29u8], ContextValue(2, PrimitiveValue::Boolean(true)));
    }

    #[test]
    fn write_context_unsigned() {
        use ast::PrimitiveValue::Unsigned;
        written_context_value_eq(&[0x39u8, 200], ContextValue(3, Unsigned(200)));
        written_context_value_eq(&[0x3Au8, 0x99, 0x88], ContextValue(3, Unsigned(0x9988)));
        written_context_value_eq(&[0x3Bu8, 0x99, 0x88, 0x77], ContextValue(3, Unsigned(0x998877)));
        written_context_value_eq(&[0x3Cu8, 0x99, 0x88, 0x77, 0x66], ContextValue(3, Unsigned(0x99887766)));
    }

    fn written_application_value_eq(data: &[u8], value: PrimitiveValue) {
        assert_eq!(data.to_vec(), write_array(ast::SequenceableValue::ApplicationValue(value)));
    }

    #[test]
    fn write_null() {
        written_application_value_eq(&[0x00u8], PrimitiveValue::Null);
    }

    #[test]
    fn write_boolean() {
        written_application_value_eq(&[0x10u8], PrimitiveValue::Boolean(false));
        written_application_value_eq(&[0x11u8], PrimitiveValue::Boolean(true));
    }

    #[test]
    fn write_unsigned() {
        use ast::PrimitiveValue::Unsigned;
        written_application_value_eq(&[0x21u8, 200], Unsigned(200));
        written_application_value_eq(&[0x22u8, 0x99, 0x88], Unsigned(0x9988));
        written_application_value_eq(&[0x23u8, 0x99, 0x88, 0x77], Unsigned(0x998877));
        written_application_value_eq(&[0x24u8, 0x99, 0x88, 0x77, 0x66], Unsigned(0x99887766));
    }
}
 
#[derive(PartialEq, Debug)]
enum Tag {
    Application(u8, u32),
    Context(u8, u32),
    Open(u8),
    Close(u8),
}
 
/// Call to write a tag (Clause 20.2.1)
///
/// # Description of encoding
///
/// BACnet encoding encodes meta data into a preceding octet.
/// BACnet has 2 classes of tags, context tags and application tags. The class is
/// identifed by the 3rd bit of the tag octet, a `1` indicating a context tag.
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
fn write_tag(writer: &mut Vec<u8>, tag: Tag) {
    let (is_named, tag_number, is_context, lvt) = match tag {
        Tag::Open(context) => (true, context, true, 0x6),
        Tag::Close(context) => (true, context, true, 0x7),
        Tag::Application(tag, value) => (false, tag, false, value),
        Tag::Context(context, value) => (false, context, true, value),
    };
    let class_flag = if is_context { 0x8 } else { 0x0 };
    let tag_portion = if tag_number <= 14 {     // Clause 20.2.1.2
        tag_number
    } else {
        0b1111
    };
    let value_portion = if is_named || lvt <= 4 {           // Clause 20.2.1.3.1
        lvt
    } else {
        0b101
    };
    writer.push((tag_portion << 4) ^ class_flag ^ value_portion as u8);
    if tag_portion == 0b1111 {
        writer.extend(&[tag_number]);
    }
    if value_portion == 0b101 {
        if lvt <= 253 {
            writer.push(lvt as u8);
        } else if lvt <= 65535 {
            writer.extend(vec![254, (lvt >> 8) as u8, lvt as u8]);
        } else {
            writer.extend(vec![255, (lvt >> 24) as u8, (lvt >> 16) as u8, (lvt >> 8) as u8, lvt as u8]);
        }
    }
}

#[cfg(test)]
mod test_write_tag {
    use super::write_tag;
    use super::Tag;

    fn assert_tag_write(expected: &[u8], tag: Tag) {
        let mut data = Vec::new();
        write_tag(&mut data, tag);
        assert_eq!(expected.to_vec(), data);
    }
   
    #[test]
    fn open() {
        assert_tag_write(&[0x2eu8], Tag::Open(2));
    }

    #[test]
    fn close() {
        assert_tag_write(&[0x2fu8], Tag::Close(2));
    }

    #[test]
    fn application0() {
        assert_tag_write(&[0x24u8], Tag::Application(2, 4));
    }

    /// Extended tag number context tag.
    #[test]
    fn context1() {
        assert_tag_write(&[0xF9u8, 0x59], Tag::Context(0x59, 1));
    }
    
    /// Extended value (8-bit) application tag.
    #[test]
    fn application1() {
        assert_tag_write(&[0x05u8, 200], Tag::Application(0, 200));
    }
    
    /// Extended value (16-bit) application tag.
    #[test]
    fn application2() {
        assert_tag_write(&[0x05u8, 0xFE, 0x59, 0x59], Tag::Application(0, 0x5959));
    }
    
    /// Extended value (32-bit) application tag.
    #[test]
    fn application32 () {
        assert_tag_write(&[0x05u8, 0xFF, 0x59, 0x59, 0x59, 0x59], Tag::Application(0, 0x59595959));
    }
}

