pub mod ast;
pub mod parse;
pub mod service;

// fn handle_whois(buf: &[u8]) -> String {
//     String::from("WHOIS")
// }
// 
// fn handle_unconfirmed(buf: &[u8]) -> String {
//     match buf.split_first() {
//         Some((&8, whois_buf)) => handle_whois(whois_buf),
//         Some((unconfirmed_service, _)) => format!("Unrecognised service choice : {}", unconfirmed_service),
//         None => String::from("End of APDU!")
//     }
// }
// 
// fn handle_apdu(buf: &[u8]) -> String {
//     match buf.split_first() {
//         Some((&0b00010000u8, unconfirmed_buf)) => handle_unconfirmed(unconfirmed_buf),
//         Some((header, _)) => format!("Unrecognised header : {}", header),
//         None => String::from("Empty APDU!")
//     }
// }
// 
// #[cfg(test)]
// mod tests {
//     use handle_apdu;
// 
//     #[test]
//     fn handle_whois() {
//         let whois_buf: &[u8] = &[0x10, 0x08, 0x09, 0x01, 0x1a, 0xc3, 0x50];
//         println!("whoising :");
//         assert_eq!(handle_apdu(whois_buf), "WHOIS");
//     }
// }
// 
