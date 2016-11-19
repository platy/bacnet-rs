//! Implementations of messages sent to and from services

use ast::ValueSequence;
use object::BacnetDB;
pub mod whois;
pub mod iam;

trait ServiceMessage where Self: Sized {
    fn unmarshall(body: &ValueSequence) -> Result<Self, UnmarshallError>;
    fn marshall(&self) -> ValueSequence;
}

#[derive(Debug, PartialEq)]
enum UnmarshallError {
    RequiredValueNotProvided,
}

/// An unconfirmed service must accept a service message, it also has access to the 
/// bacnet object database and has the option to send an unconfirmed message in response
//type UnconfirmedHandler = fn(ServiceMessage, BacnetDB) -> Option<ServiceMessage>;

trait UnconfirmedService<Req: ServiceMessage, Res: ServiceMessage> {
	fn service_choice() -> u8;
	fn handle(Req, BacnetDB) -> Option<Res>;
}

