//! A big part of BACnet is its object database
pub struct DeviceObject {
    pub instance: u32,
    pub max_apdu_length_supported: u32,
    pub segmentation_supported: u8,
    pub vendor_identifier: u32,
}

pub mod object_type {
    pub const DEVICE: u16 = 8;
}

#[derive(PartialEq, Debug)]
pub struct ObjectId(pub u16, pub u32);

pub struct BacnetDB {
	device: DeviceObject,
}

impl BacnetDB {
    pub fn new(device: DeviceObject) -> BacnetDB {
        BacnetDB {
            device: device,
        }
    }

	pub fn device(&self) -> &DeviceObject {
		&self.device
	}
}

