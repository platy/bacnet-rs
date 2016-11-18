//! Implementations of messages sent to and from services

use ast::ValueSequence;
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

