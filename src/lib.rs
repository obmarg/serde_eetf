extern crate eetf;
extern crate num_bigint;
extern crate num_traits;
extern crate serde;

// We need serde_derive to derive serializers/deserializers in our tests.
#[cfg(test)]
#[macro_use]
extern crate serde_derive;

//mod de;
mod error;
mod ser;

//pub use de::{from_str, Deserializer};
pub use error::{Error, Result};
pub use ser::{to_writer, Serializer};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
