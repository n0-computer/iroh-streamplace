uniffi::setup_scaffolding!();

pub mod key;
pub mod receiver;
pub mod sender;
pub mod utils;

const ALPN: &[u8] = b"/iroh/streamplace/1";
