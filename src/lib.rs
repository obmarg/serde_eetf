//! Erlang external term format support for serde.  This allows a rust app to
//! easily talk to erlang, elixir or any other BEAM language with little
//! boilerplate.
//!
//! ```rust

//! # #[macro_use]
//! # extern crate serde_derive;
//! # extern crate serde_eetf;
//! # fn main() {
//! use serde_eetf::{to_bytes, from_bytes};
//!
//! #[derive(Deserialize, Serialize, PartialEq, Debug)]
//! struct Test {
//!     x: i8
//! }
//!
//! let input = Test{ x: 8 };
//! // Here we convert our Test struct into some bytes that could be sent to
//! // Erlang.
//! let bytes = to_bytes(&input).unwrap();
//!
//! // When we receive a response from Erlang we can convert it back to our Test
//! // struct.
//! let result: Test = from_bytes(&bytes).unwrap();
//!
//! # assert_eq!(result, input);
//! # }
//! ```

extern crate eetf;
extern crate heck;
extern crate num_bigint;
extern crate num_traits;
#[macro_use]
extern crate serde;

// We need serde_derive to derive serializers/deserializers in our tests.
#[cfg(test)]
#[macro_use]
extern crate serde_derive;

mod de;
mod error;
mod ser;

pub use crate::de::{from_bytes, from_reader, Deserializer};
pub use crate::error::{Error, Result};
pub use crate::ser::{to_bytes, to_writer};
