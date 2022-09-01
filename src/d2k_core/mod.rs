use std::collections::HashMap;

pub use self::function::Function;
pub use self::cloudflare::Cloudflare;
pub use self::record::Record;

pub mod function;
pub mod cloudflare;
pub mod record;

pub mod d2k_core {}

