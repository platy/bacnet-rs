//! Implementations of messages sent to and from services

use ast::ValueSequence;

trait ServiceMessage where Self: Sized {
    fn unmarshall(body: &ValueSequence) -> Result<Self, UnmarshallError>;
    fn marshall(&self) -> ValueSequence;
}

#[derive(Debug, PartialEq)]
enum UnmarshallError {
    RequiredValueNotProvided,
}

mod whois {
    use ast::ValueSequence;
    use ast::PrimitiveValue::Unsigned;
    use ast::SequenceableValue::ContextValue;
    use ast::get_context_value;
    use super::ServiceMessage;
    use super::UnmarshallError;

    #[derive(Debug, PartialEq)]
    struct Message {
        device_instance_low: u32,
        device_instance_high: u32,
    }

    impl ServiceMessage for Message {
        fn marshall(&self) -> ValueSequence {
            vec!(
                ContextValue(0, Unsigned(self.device_instance_low)), 
                ContextValue(1, Unsigned(self.device_instance_high)))
        }

        fn unmarshall(body: &ValueSequence) -> Result<Self, UnmarshallError> {
            match (get_context_value(body, 0), get_context_value(body, 1)) {
                (Some(&Unsigned(low)), Some(&Unsigned(high))) => Ok(Message {
                    device_instance_low: low,
                    device_instance_high: high,
                }),
                _ => Err(UnmarshallError::RequiredValueNotProvided),
            }
        }
    }
    
    #[cfg(test)]
    mod test {
        use super::Message;
        use super::super::ServiceMessage;
        use ast::PrimitiveValue::Unsigned;
        use ast::SequenceableValue::ContextValue;

        #[test]
        fn test_unmarshall_correct() {
            assert_eq!(Ok(Message { device_instance_low: 1, device_instance_high: 50000 }),
                       Message::unmarshall(&vec!(
                              ContextValue(0, Unsigned(1)), 
                              ContextValue(1, Unsigned(50000)))));
        }

        #[test]
        fn test_marshall_correct() {
            assert_eq!(vec!(
                    ContextValue(0, Unsigned(1)),
                    ContextValue(1, Unsigned(50000))),
                    Message { device_instance_low: 1, device_instance_high: 50000 }.marshall());
        }

        #[test]
        fn test_marshall_cycle() {
            let message = Message { device_instance_low: 1, device_instance_high: 50000 };
            assert_eq!(message, Message::unmarshall(&message.marshall()).unwrap());
        }
    }
}

mod iam {
    use ast::ValueSequence;
    use ast::PrimitiveValue::Unsigned;
    use ast::SequenceableValue::ApplicationValue;
    use ast::PrimitiveValue::ObjectId;
    use ast::PrimitiveValue::Enumerated;
    use ast::ObjectType;
    use super::ServiceMessage;
    use super::UnmarshallError;

    #[derive(Debug, PartialEq)]
    struct Message {
        device_instance: u32,
        max_apdu: u32,
        segmentation_support: u8,
        vendor_id: u32,
    }

    impl ServiceMessage for Message {
        fn marshall(&self) -> ValueSequence {
            vec!(
                ApplicationValue(ObjectId(ObjectType::Device, self.device_instance)),
                ApplicationValue(Unsigned(self.max_apdu)),
                ApplicationValue(Enumerated(self.segmentation_support as u32)),
                ApplicationValue(Unsigned(self.vendor_id)),
            )
        }

        fn unmarshall(body: &ValueSequence) -> Result<Self, UnmarshallError> {
            match (&body[0], &body[1], &body[2], &body[3]) {
                (&ApplicationValue(ObjectId(ObjectType::Device, device_instance)), 
                 &ApplicationValue(Unsigned(max_apdu)),
                 &ApplicationValue(Enumerated(segmentation_support)),
                 &ApplicationValue(Unsigned(vendor_id))) =>
                    Ok(Message {
                        device_instance: device_instance,
                        max_apdu: max_apdu,
                        segmentation_support: segmentation_support as u8,
                        vendor_id: vendor_id,
                    }),
                _ => Err(UnmarshallError::RequiredValueNotProvided),
            }
        }
    }
    
    #[cfg(test)]
    mod test {
        use super::Message;
        use super::super::ServiceMessage;
        use ast::PrimitiveValue::Unsigned;
        use ast::PrimitiveValue::Enumerated;
        use ast::PrimitiveValue::ObjectId;
        use ast::SequenceableValue::ApplicationValue;
        use ast::ObjectType;

        #[test]
        fn test_unmarshall_correct() {
            assert_eq!(Ok(Message { device_instance: 10, max_apdu: 1476, segmentation_support: 3, vendor_id: 1 }),
                       Message::unmarshall(&vec!(
                              ApplicationValue(ObjectId(ObjectType::Device, 10)), 
                              ApplicationValue(Unsigned(1476)),
                              ApplicationValue(Enumerated(3)),
                              ApplicationValue(Unsigned(1)))));
        }

        #[test]
        fn test_marshall_correct() {
            assert_eq!(vec!(
                    ApplicationValue(ObjectId(ObjectType::Device, 10)),
                    ApplicationValue(Unsigned(1476)),
                    ApplicationValue(Enumerated(3)),
                    ApplicationValue(Unsigned(1))),
                    Message { device_instance: 10, max_apdu: 1476, segmentation_support: 3, vendor_id: 1 }.marshall());
        }

        #[test]
        fn test_marshall_cycle() {
            let message = Message { device_instance: 10, max_apdu: 1476, segmentation_support: 3, vendor_id: 1 };
            assert_eq!(message, Message::unmarshall(&message.marshall()).unwrap());
        }
    }
}

