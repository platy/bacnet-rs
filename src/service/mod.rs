//! Implementations of bacnet services

use ast::ValueSequence;
use ast::ApduHeader;
use object::BacnetDB;
mod whois;
mod iam;

pub fn handle_apdu(header: ApduHeader, body: &ValueSequence, db: &BacnetDB) -> Option<ValueSequence> {
    match header {
        ApduHeader::UnconfirmedReq { service: choice } =>
            unconfirmed_service(choice)(body, db),
        _ => panic!("Unsupported request type"),
    }
}

fn unconfirmed_service(choice: u8) -> UnconfirmedHandler {
    match choice {
        8 => whois::handler,
        _ => panic!("Not suporting this service"),
    }
}

trait ServiceMessage where Self: Sized {
    fn choice() -> u8;
    fn unmarshall(body: &ValueSequence) -> Result<Self, UnmarshallError>;
    fn marshall(&self) -> ValueSequence;
}

#[derive(Debug, PartialEq)]
enum UnmarshallError {
    RequiredValueNotProvided,
}

/// An unconfirmed service must accept a service message, it also has access to the 
/// bacnet object database and has the option to send an unconfirmed message in response
type UnconfirmedHandler = fn(&ValueSequence, &BacnetDB) -> Option<ValueSequence>;

// trait UnconfirmedService {
// 	fn service_choice() -> u8;
// 	fn handle(&ValueSequence, &BacnetDB) -> Option<()>;
// }

