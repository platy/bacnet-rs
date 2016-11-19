//! The Whois service is activated by unconfirmed whois messages for which this device resides in
//! the specified device ID range. The whois service should send out iam messages when activated

use super::ServiceMessage;
use super::UnmarshallError;

use object;
use service;
use ast::ValueSequence;
use ast::PrimitiveValue::Unsigned;
use ast::SequenceableValue::ContextValue;
use ast::get_context_value;

#[derive(Debug, PartialEq)]
struct Message {
    device_instance_low: u32,
    device_instance_high: u32,
}

fn handler(whois: Message, db: object::BacnetDB) -> Option<service::iam::Message> {
    let device = db.device();
	if whois.device_instance_low <= device.instance && whois.device_instance_high >= device.instance {
        Some(super::iam::Message::about(device))
    } else {
        None
    }
}

#[cfg(test)]
mod handler_test {
	use super::handler;
	use super::Message;
	use service::iam;
	use object::BacnetDB;
	use object::DeviceObject;
	
	const DEVICE: DeviceObject = DeviceObject {
		instance: 45,
		max_apdu_length_supported: 1000,
		vendor_identifier: 23,
		segmentation_supported: 3,	
	};
	fn test_db() -> BacnetDB {
        BacnetDB::new(DEVICE)
    }

	fn whois_range(low: u32, high: u32) -> Option<iam::Message> {
		handler(Message {
			device_instance_low: low,
			device_instance_high: high,
		}, test_db())
	}

	#[test]
	fn range_checks() {
		assert_eq!(None, whois_range(46, 100));
		assert_eq!(None, whois_range(0, 43));
		assert_eq!(Some(iam::Message::about(&DEVICE)), whois_range(45, 100));
		assert_eq!(Some(iam::Message::about(&DEVICE)), whois_range(1, 45));
	}
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
mod message {
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

