use num_bigint::BigInt;
use num_traits::cast::FromPrimitive;
use serde::ser::{self, Serialize};
use std::io;
use std::convert::TryFrom;

use heck::SnakeCase;

use eetf::{self, Term};

use error::{Error, Result};

/// Serializes a value into EETF using a Write
pub fn to_writer<T, W>(value: &T, writer: &mut W) -> Result<()>
where
    T: Serialize + ?Sized,
    W: io::Write + ?Sized,
{
    let serializer = Serializer {};
    let term = value.serialize(&serializer)?;
    match term.encode(writer) {
        Ok(_result) => Ok(()),
        Err(_error) => Err(Error::EncodeError("TODO".to_string())),
    }
}

/// Serializes a value into a EETF in a Vec of bytes
pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize + ?Sized,
{
    let mut cursor = io::Cursor::new(Vec::new());

    match to_writer(value, &mut cursor) {
        Ok(_) => Ok(cursor.into_inner()),
        Err(e) => Err(e),
    }
}

/// Serializes 
struct Serializer {}

struct SequenceSerializer {
    items: Vec<Term>,
}

struct NamedSequenceSerializer {
    name: Term,
    items: Vec<Term>,
}

struct MapSerializer {
    items: Vec<(Term, Term)>,
}

struct NamedMapSerializer {
    name: Term,
    items: Vec<(Term, Term)>,
}

impl<'a> ser::Serializer for &'a Serializer {
    // The output type produced by this `Serializer` during successful
    // serialization.
    type Ok = Term;

    // The error type when some error occurs during serialization.
    type Error = Error;

    // Associated types for keeping track of additional state while serializing
    // compound data structures like sequences and maps.
    type SerializeSeq = SequenceSerializer;
    type SerializeTuple = SequenceSerializer;
    type SerializeTupleStruct = SequenceSerializer;
    type SerializeTupleVariant = NamedSequenceSerializer;
    type SerializeMap = MapSerializer;
    type SerializeStruct = MapSerializer;
    type SerializeStructVariant = NamedMapSerializer;

    // The following 12 methods receive one of the primitive types of the data
    // model and map it to eetf
    fn serialize_bool(self, v: bool) -> Result<Term> {
        // TODO: Make this actually a boolean?
        Ok(Term::Atom(eetf::Atom::from(if v {
            "true"
        } else {
            "false"
        })))
    }

    // eetf has two kinds of integers: 32 bit ones and big ints.
    fn serialize_i8(self, v: i8) -> Result<Term> {
        self.serialize_i32(i32::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<Term> {
        self.serialize_i32(i32::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<Term> {
        Ok(Term::FixInteger(eetf::FixInteger { value: v }))
    }

    fn serialize_i64(self, v: i64) -> Result<Term> {
        let big_int = BigInt::from_i64(v).expect("TODO: Handle failure here");
        Ok(Term::BigInteger(eetf::BigInteger { value: big_int }))
    }

    fn serialize_u8(self, v: u8) -> Result<Term> {
        self.serialize_u16(u16::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<Term> {
        Ok(Term::FixInteger(eetf::FixInteger::from(v)))
    }

    // The eetf crate uses an i32 to encode FixIntegers, so for unsigned numbers
    // we use a BigInteger instead.
    fn serialize_u32(self, v: u32) -> Result<Term> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<Term> {
        let big_int = BigInt::from_u64(v).expect("TODO: Handle failure here");
        Ok(Term::BigInteger(eetf::BigInteger { value: big_int }))
    }

    fn serialize_f32(self, v: f32) -> Result<Term> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Term> {
        Ok(Term::Float(eetf::Float::try_from(v)?))
    }

    // Serialize a char as a single-character string.
    // TODO: Decide if this is a good idea.
    fn serialize_char(self, v: char) -> Result<Term> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Term> {
        Ok(Term::Binary(eetf::Binary::from(v.as_bytes())))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Term> {
        Ok(Term::Binary(eetf::Binary::from(v)))
    }

    // An absent optional is represented as the JSON `null`.
    fn serialize_none(self) -> Result<Term> {
        Ok(Term::Atom(eetf::Atom::from("nil")))
    }

    // At present optional is represented as just the contained value. Note that
    // this is a lossy representation. For example the values `Some(())` and
    // `None` both serialize as just `null`.
    // TODO: Decide if this makes sense.
    fn serialize_some<T>(self, value: &T) -> Result<Term>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // In Serde, unit means an anonymous value containing no data.
    // Map this to eetf as `nil`.
    fn serialize_unit(self) -> Result<Term> {
        // TODO: Decide if this is right.
        self.serialize_none()
    }

    // Unit struct means a named value containing no data.
    // We basically just treat this like nil for now.
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Term> {
        self.serialize_unit()
    }

    // When serializing a unit variant (or any other kind of variant), formats
    // can choose whether to keep track of it by index or by name. Binary
    // formats typically use the index of the variant and human-readable formats
    // typically use the name.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Term> {
        Ok(Term::Atom(eetf::Atom::from(variant.to_snake_case())))
    }

    // We treat newtype structs as insignificant wrappers around the data they
    // contain.
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Term>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // Note that newtype variant (and all of the other variant serialization
    // methods) refer exclusively to the "externally tagged" enum
    // representation.
    //
    // We serialize this to {value_name, value}, which hopefully allows results
    // to be serialized into fairly standard erlang ok/err tuples.
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Term>
    where
        T: ?Sized + Serialize,
    {
        let serialized_value = value.serialize(self)?;
        Ok(Term::Tuple(eetf::Tuple::from(vec![
            Term::Atom(eetf::Atom::from(variant.to_snake_case())),
            serialized_value,
        ])))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        let vec = match len {
            None => Vec::new(),
            Some(len) => Vec::with_capacity(len),
        };
        Ok(SequenceSerializer { items: vec })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        Ok(SequenceSerializer {
            items: Vec::with_capacity(len),
        })
    }

    // We treat tuple structs exactly like tuples for now.
    // TODO: Decide if this is a good idea.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_tuple(len)
    }

    // Tuple variants are represented in eetf as `{name, {data}}`. Again
    // this method is only responsible for the externally tagged representation.
    // TODO: decide if this is a good idea...
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(NamedSequenceSerializer {
            name: Term::Atom(eetf::Atom::from(variant.to_snake_case())),
            items: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        let vec = match len {
            None => Vec::new(),
            Some(len) => Vec::with_capacity(len),
        };
        Ok(MapSerializer { items: vec })
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        // TODO: decide how to do this....
        // do we want to tag things?
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(NamedMapSerializer {
            name: Term::Atom(eetf::Atom::from(variant.to_snake_case())),
            items: Vec::with_capacity(len),
        })
    }
}

impl<'a> ser::SerializeSeq for SequenceSerializer {
    type Ok = Term;
    type Error = Error;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let term_value = value.serialize(&Serializer {})?;
        self.items.push(term_value);
        Ok(())
    }

    fn end(self) -> Result<Term> {
        Ok(Term::List(eetf::List {
            elements: self.items,
        }))
    }
}

impl<'a> ser::SerializeTuple for SequenceSerializer {
    type Ok = Term;
    type Error = Error;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let term_value = value.serialize(&Serializer {})?;
        self.items.push(term_value);
        Ok(())
    }

    fn end(self) -> Result<Term> {
        Ok(Term::Tuple(eetf::Tuple {
            elements: self.items,
        }))
    }
}

impl<'a> ser::SerializeTupleStruct for SequenceSerializer {
    type Ok = Term;
    type Error = Error;

    // Serialize a single element of the sequence.
    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let term_value = value.serialize(&Serializer {})?;
        self.items.push(term_value);
        Ok(())
    }

    fn end(self) -> Result<Term> {
        Ok(Term::Tuple(eetf::Tuple {
            elements: self.items,
        }))
    }
}

impl<'a> ser::SerializeTupleVariant for NamedSequenceSerializer {
    type Ok = Term;
    type Error = Error;

    // Serialize a single element of the sequence.
    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let term_value = value.serialize(&Serializer {})?;
        self.items.push(term_value);
        Ok(())
    }

    fn end(self) -> Result<Term> {
        // TODO: rename items to elements.
        let serialized_data = Term::Tuple(eetf::Tuple {
            elements: self.items,
        });
        Ok(Term::Tuple(eetf::Tuple::from(vec![
            self.name,
            serialized_data,
        ])))
    }
}

impl<'a> ser::SerializeMap for MapSerializer {
    type Ok = Term;
    type Error = Error;

    // Serialize a single element of the sequence.
    fn serialize_key<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        panic!("Not Implemented")
    }

    fn serialize_value<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        panic!("Not Implemented")
    }

    fn serialize_entry<K: ?Sized, V: ?Sized>(&mut self, key: &K, value: &V) -> Result<()>
    where
        K: Serialize,
        V: Serialize,
    {
        let key_term = key.serialize(&Serializer {})?;
        let value_term = value.serialize(&Serializer {})?;
        self.items.push((key_term, value_term));
        Ok(())
    }

    fn end(self) -> Result<Term> {
        // TODO: rename items to entries.
        Ok(Term::Map(eetf::Map {
            entries: self.items,
        }))
    }
}

impl<'a> ser::SerializeStruct for MapSerializer {
    type Ok = Term;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let value_term = value.serialize(&Serializer {})?;
        self.items
            .push((Term::Atom(eetf::Atom::from(key)), value_term));
        Ok(())
    }

    fn end(self) -> Result<Term> {
        Ok(Term::Map(eetf::Map {
            entries: self.items,
        }))
    }
}

impl<'a> ser::SerializeStructVariant for NamedMapSerializer {
    type Ok = Term;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let value_term = value.serialize(&Serializer {})?;
        self.items
            .push((Term::Atom(eetf::Atom::from(key)), value_term));
        Ok(())
    }

    fn end(self) -> Result<Term> {
        let serialized_data = Term::Map(eetf::Map {
            entries: self.items,
        });
        Ok(Term::Tuple(eetf::Tuple::from(vec![
            self.name,
            serialized_data,
        ])))
    }
}

// TODO: More Tests
#[cfg(test)]
mod tests {
    use super::*;

    // Helper function for tests. Runs things through our serializer then
    // decodes and returns.
    fn serialize_and_decode<T>(data: T) -> Term
    where
        T: Serialize,
    {
        let bytes = to_bytes(&data).expect("serialize failed");
        Term::decode(io::Cursor::new(bytes)).expect("Decode failed")
    }

    #[test]
    fn test_unsigned_ints_and_structs() {
        #[derive(PartialEq, Serialize)]
        struct TestStruct {
            unsigned8: u8,
            unsigned16: u16,
            unsigned32: u32,
            unsigned64: u64,
        }

        let result = serialize_and_decode(TestStruct {
            unsigned8: 129,
            unsigned16: 65530,
            unsigned32: 65530,
            unsigned64: 65530,
        });
        assert_eq!(
            result,
            Term::Map(eetf::Map::from(vec![
                (
                    Term::Atom(eetf::Atom::from("unsigned8")),
                    Term::FixInteger(eetf::FixInteger::from(129))
                ),
                (
                    Term::Atom(eetf::Atom::from("unsigned16")),
                    Term::FixInteger(eetf::FixInteger::from(65530))
                ),
                (
                    Term::Atom(eetf::Atom::from("unsigned32")),
                    Term::BigInteger(eetf::BigInteger::from(65530))
                ),
                (
                    Term::Atom(eetf::Atom::from("unsigned64")),
                    Term::BigInteger(eetf::BigInteger::from(65530))
                )
            ]))
        )
    }

    #[test]
    fn test_signed_ints_and_tuple_structs() {
        #[derive(PartialEq, Serialize)]
        struct TestStruct(i8, i16, i32, i64);

        let result = serialize_and_decode(TestStruct(-127, 30000, 65530, 65530));
        assert_eq!(
            result,
            Term::Tuple(eetf::Tuple::from(vec![
                Term::FixInteger(eetf::FixInteger::from(-127)),
                Term::FixInteger(eetf::FixInteger::from(30000)),
                Term::FixInteger(eetf::FixInteger::from(65530)),
                Term::BigInteger(eetf::BigInteger::from(65530)),
            ]))
        )
    }

    #[test]
    fn test_binaries_tuples_and_lists() {
        let result = serialize_and_decode(("ABCD", vec![0, 1, 2]));
        assert_eq!(
            result,
            Term::Tuple(eetf::Tuple::from(vec![
                Term::Binary(eetf::Binary::from("ABCD".as_bytes())),
                Term::List(eetf::List::from(vec![
                    Term::FixInteger(eetf::FixInteger::from(0)),
                    Term::FixInteger(eetf::FixInteger::from(1)),
                    Term::FixInteger(eetf::FixInteger::from(2)),
                ]))
            ]))
        )
    }

    #[test]
    fn test_option() {
        let none: Option<u8> = None;
        let nil_result = serialize_and_decode(none);
        let some_result = serialize_and_decode(Some(0));

        assert_eq!(nil_result, Term::Atom(eetf::Atom::from("nil")));

        assert_eq!(some_result, Term::FixInteger(eetf::FixInteger::from(0)));
    }

    #[test]
    fn test_unit_variant() {
        #[derive(Serialize)]
        enum E {
            AnOption,
            AnotherOption,
        };

        let result = serialize_and_decode(E::AnOption);

        assert_eq!(result, Term::Atom(eetf::Atom::from("an_option")))
    }

    #[test]
    fn test_newtype_variant() {
        // Not 100% sure if this is a tuple variant or a newtype variant.
        // But whatever I guess?
        #[derive(Serialize)]
        enum ErlResult {
            Ok(String),
        };

        let result = serialize_and_decode(ErlResult::Ok("test".to_string()));

        assert_eq!(
            result,
            Term::Tuple(eetf::Tuple::from(vec![
                Term::Atom(eetf::Atom::from("ok")),
                Term::Binary(eetf::Binary::from("test".as_bytes())),
            ]))
        );
    }

    #[test]
    fn test_tuple_variant() {
        // Not 100% sure if this is a tuple variant or a newtype variant.
        // But whatever I guess?
        #[derive(Serialize)]
        enum Testing {
            Ok(u8, u8),
        };

        let result = serialize_and_decode(Testing::Ok(1, 2));

        assert_eq!(
            result,
            Term::Tuple(eetf::Tuple::from(vec![
                Term::Atom(eetf::Atom::from("ok")),
                Term::Tuple(eetf::Tuple::from(vec![
                    Term::FixInteger(eetf::FixInteger::from(1)),
                    Term::FixInteger(eetf::FixInteger::from(2)),
                ]))
            ]))
        );
    }
}
