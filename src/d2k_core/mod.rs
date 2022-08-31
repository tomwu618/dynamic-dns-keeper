use std::collections::HashMap;

pub use self::function::Function;
pub use self::cloudflare::Cloudflare;

pub mod function;
pub mod cloudflare;
mod Record;

pub mod d2k_core {}

