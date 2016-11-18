//! Iam messages are unconfirmed requests and can be broadcast on the network to update the network
//! to a device's current info or can be sent as a reply by the whois service.

use ast::ValueSequence;
use ast::PrimitiveValue::Unsigned;
use ast::SequenceableValue::ApplicationValue;
use ast::PrimitiveValue::ObjectId;
use ast::PrimitiveValue::Enumerated;
use ast::object_type;
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
            ApplicationValue(ObjectId(object_type::DEVICE, self.device_instance)),
            ApplicationValue(Unsigned(self.max_apdu)),
            ApplicationValue(Enumerated(self.segmentation_support as u32)),
            ApplicationValue(Unsigned(self.vendor_id)),
        )
    }

    fn unmarshall(body: &ValueSequence) -> Result<Self, UnmarshallError> {
        match (&body[0], &body[1], &body[2], &body[3]) {
            (&ApplicationValue(ObjectId(object_type::DEVICE, device_instance)), 
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
    use ast::object_type;

    #[test]
    fn test_unmarshall_correct() {
        assert_eq!(Ok(Message { device_instance: 10, max_apdu: 1476, segmentation_support: 3, vendor_id: 1 }),
                   Message::unmarshall(&vec!(
                          ApplicationValue(ObjectId(object_type::DEVICE, 10)), 
                          ApplicationValue(Unsigned(1476)),
                          ApplicationValue(Enumerated(3)),
                          ApplicationValue(Unsigned(1)))));
    }

    #[test]
    fn test_marshall_correct() {
        assert_eq!(vec!(
                ApplicationValue(ObjectId(object_type::DEVICE, 10)),
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


