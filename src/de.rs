use eetf::Term;
use std::io::{self, Read};
use std::iter;
use std::slice::Iter;
use std::str;

use heck::CamelCase;

use num_traits::cast::{FromPrimitive, ToPrimitive};

use serde::de::{
    self, Deserialize, DeserializeOwned, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess,
    SeqAccess, VariantAccess, Visitor,
};

use error::{Error, Result};

use self::private::*;

pub struct Deserializer<'a> {
    term: &'a Term,
}

impl<'a> Deserializer<'a> {
    pub fn from_term(term: &'a Term) -> Self {
        Deserializer { term: term }
    }
}

// TODO: figure out if this is needed.
trait IntoEetfDeserializer {
    fn into_deserializer<'a>(&'a self) -> Deserializer<'a>;
}

impl IntoEetfDeserializer for Term {
    fn into_deserializer<'a>(&'a self) -> Deserializer<'a> {
        Deserializer::from_term(self)
    }
}

// impl<'de, 'a: 'de> From<&'a Term> for Deserializer<'de> {
//     fn from(term: &'a Term) -> Self {
//         Deserializer::from_term(term)
//     }
// }

pub fn from_reader<R, T>(reader: R) -> Result<T>
where
    R: Read,
    T: DeserializeOwned,
{
    let term = Term::decode(reader)?;
    let deserializer = Deserializer::from_term(&term);
    let t = T::deserialize(deserializer)?;
    Ok(t)
}

pub fn from_bytes<T>(bytes: &[u8]) -> Result<T>
where
    T: DeserializeOwned,
{
    let cursor = io::Cursor::new(bytes);

    from_reader(cursor)
}

// Implementation methods for deserializer that require a lifetime.
impl<'a> Deserializer<'a> {
    fn parse_integer<T>(&self) -> Result<T>
    where
        T: FromPrimitive,
    {
        match self.term {
            Term::FixInteger(fix_int) => {
                if let Some(num) = T::from_i32(fix_int.value) {
                    Ok(num)
                } else {
                    Err(Error::IntegerConvertError)
                }
            }
            Term::BigInteger(big_int) => {
                if let Some(num) = big_int.to_i64() {
                    if let Some(num) = T::from_i64(num) {
                        Ok(num)
                    } else {
                        Err(Error::IntegerConvertError)
                    }
                } else {
                    Err(Error::IntegerConvertError)
                }
            }
            _ => Err(Error::ExpectedFixInteger),
        }
    }

    fn parse_float<T>(&self) -> Result<T>
    where
        T: FromPrimitive,
    {
        match self.term {
            Term::Float(float) => if let Some(num) = T::from_f64(float.value) {
                Ok(num)
            } else {
                Err(Error::IntegerConvertError)
            },
            _ => Err(Error::ExpectedFloat),
        }
    }

    fn parse_binary(&self) -> Result<&[u8]> {
        match self.term {
            Term::Binary(binary) => Ok(&binary.bytes),
            _ => Err(Error::ExpectedBinary),
        }
    }

    fn parse_string(&self) -> Result<String> {
        match self.parse_binary() {
            Ok(bytes) => str::from_utf8(&bytes)
                .map(|s| s.to_string())
                .or(Err(Error::Utf8DecodeError)),
            Err(e) => Err(e),
        }
    }
}

impl<'de, 'a: 'de> de::Deserializer<'de> for Deserializer<'a> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::TypeHintsRequired)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.term {
            Term::Atom(b) => {
                if b.name == "true" {
                    visitor.visit_bool(true)
                } else if b.name == "false" {
                    visitor.visit_bool(false)
                } else {
                    Err(Error::InvalidBoolean)
                }
            }

            _ => Err(Error::ExpectedBoolean),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.parse_integer()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.parse_integer()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.parse_integer()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.parse_integer()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.parse_integer()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.parse_integer()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.parse_integer()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.parse_integer()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f32(self.parse_float()?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.parse_float()?)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.parse_string() {
            Err(Error::ExpectedBinary) => Err(Error::ExpectedChar),
            Err(other) => Err(other),
            Ok(string) => {
                if string.len() == 1 {
                    visitor.visit_char(string.chars().next().unwrap())
                } else {
                    Err(Error::ExpectedChar)
                }
            }
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.parse_string()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bytes(self.parse_binary()?)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bytes(self.parse_binary()?)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.term {
            Term::Atom(atom) => {
                if atom.name == "nil" {
                    visitor.visit_none()
                } else {
                    visitor.visit_some(self)
                }
            }
            other => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.term {
            Term::Atom(atom) => {
                if atom.name == "nil" {
                    visitor.visit_unit()
                } else {
                    Err(Error::ExpectedNil)
                }
            }
            other => Err(Error::ExpectedNil),
        }
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    // Deserialization of compound types like sequences and maps happens by
    // passing the visitor an "Access" object that gives it the ability to
    // iterate through the data contained in the sequence.
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.term {
            Term::List(list) => {
                let seq_deserializer = ListDeserializer::new(list.elements.iter());
                visitor.visit_seq(seq_deserializer)
                // TODO: Figure out how to call end here.
            }
            other => {
                eprintln!("{}", other);
                Err(Error::ExpectedList)
            }
        }
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.term {
            Term::Tuple(tuple) => {
                if tuple.elements.len() != len {
                    return Err(Error::WrongTupleLength);
                }
                let seq_deserializer = ListDeserializer::new(tuple.elements.iter());
                visitor.visit_seq(seq_deserializer)
                // TODO: Figure out how to call end here.
            }
            _ => Err(Error::ExpectedTuple),
        }
    }

    // Tuple structs look just like tuples in EETF.
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.term {
            Term::Map(map) => {
                let mut map_deserializer = MapDeserializer::new(map.entries.iter());
                visitor.visit_map(&mut map_deserializer).and_then(|result| {
                    match map_deserializer.end() {
                        Ok(()) => Ok(result),
                        Err(e) => Err(e),
                    }
                })
            }
            _ => Err(Error::ExpectedMap),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // TODO: So, think I need to somehow tell the MapDeserializer that it needs
        // to deserialize string keys...
        match self.term {
            Term::Map(map) => {
                let mut map_deserializer = MapDeserializer::new(map.entries.iter());
                visitor.visit_map(&mut map_deserializer).and_then(|result| {
                    match map_deserializer.end() {
                        Ok(()) => Ok(result),
                        Err(e) => Err(e),
                    }
                })
            }
            _ => Err(Error::ExpectedMap),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.term {
            Term::Atom(atom) => {
                // We have a unit variant.
                visitor.visit_enum(atom.name.to_camel_case().into_deserializer())
            }
            Term::Tuple(tuple) => match tuple.elements.as_slice() {
                [variant_term, value_term] => {
                    visitor.visit_enum(EnumDeserializer::new(&variant_term, &value_term))
                }
                _ => return Err(Error::MisSizedVariantTuple),
            },
            _ => Err(Error::ExpectedAtomOrTuple),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.term {
            Term::Atom(atom) => visitor.visit_string(atom.name.clone()),
            _ => Err(Error::ExpectedAtom),
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // Just skip over this by calling visit_unit.
        visitor.visit_unit()
    }
}

struct ListDeserializer<I>
where
    I: Iterator,
{
    iter: iter::Fuse<I>,
}

impl<I> ListDeserializer<I>
where
    I: Iterator,
{
    fn new(iter: I) -> Self {
        ListDeserializer { iter: iter.fuse() }
    }

    fn end(self) -> Result<()> {
        if self.iter.count() == 0 {
            Ok(())
        } else {
            Err(Error::TooManyItems)
        }
    }
}

impl<'de, 'a: 'de, I> SeqAccess<'de> for ListDeserializer<I>
where
    I: Iterator<Item = &'a Term>,
{
    type Error = Error;

    fn next_element_seed<V>(&mut self, seed: V) -> Result<Option<V::Value>>
    where
        V: de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(term) => seed.deserialize(Deserializer::from_term(term)).map(Some),
            None => Ok(None),
        }
    }
}

// TODO: Look at https://github.com/flavray/avro-rs/blob/master/src/de.rs#L50-L53
// and figure out if we can use it's ideas to simplify all this lifetime shit.

struct MapDeserializer<'de, I, T>
where
    I: Iterator<Item = T>,
    T: Pair<'de> + 'de,
    First<'de, I::Item>: 'de,
    Second<'de, I::Item>: 'de,
{
    items: iter::Fuse<I>,
    current_value: Option<&'de T::Second>,
}

impl<'de, I, T> MapDeserializer<'de, I, T>
where
    I: Iterator<Item = T>,
    T: Pair<'de>,
{
    fn new(iter: I) -> Self {
        MapDeserializer {
            items: iter.fuse(),
            current_value: None,
        }
    }

    fn end(self) -> Result<()> {
        if self.items.count() == 0 {
            Ok(())
        } else {
            Err(Error::TooManyItems)
        }
    }
}

impl<'a, 'de: 'a, I, T> MapAccess<'de> for &'a mut MapDeserializer<'de, I, T>
where
    I: Iterator<Item = T>,
    T: Pair<'de>,
    First<'de, I::Item>: IntoEetfDeserializer,
    Second<'de, I::Item>: IntoEetfDeserializer,
{
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        if let Some(_) = self.current_value {
            panic!("MapDeserializer.next_key_seed was called twice in a row")
        }

        match self.items.next() {
            Some(pair) => {
                let (key, val) = pair.split();
                self.current_value = Some(val);

                seed.deserialize(key.into_deserializer()).map(Some)
            }
            None => Ok(None),
        }
    }

    // TODO: implement next_entry_seed instead.

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        if let Some(value) = self.current_value {
            self.current_value = None;
            seed.deserialize(value.into_deserializer())
        } else {
            panic!("MapDeserializer.next_value_seed was called before next_key_seed")
        }
    }
}

struct EnumDeserializer<'de> {
    variant: &'de Term,
    term: &'de Term,
}

impl<'de> EnumDeserializer<'de> {
    fn new(variant: &'de Term, term: &'de Term) -> Self {
        EnumDeserializer {
            variant: variant,
            term: term,
        }
    }
}

// `EnumAccess` is provided to the `Visitor` to give it the ability to determine
// which variant of the enum is supposed to be deserialized.
//
// Note that all enum deserialization methods in Serde refer exclusively to the
// "externally tagged" enum representation.
impl<'de> EnumAccess<'de> for EnumDeserializer<'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        let val = seed.deserialize(VariantNameDeserializer::from_term(self.variant))?;
        Ok((val, self))
    }
}

// `VariantAccess` is provided to the `Visitor` to give it the ability to see
// the content of the single variant that it decided to deserialize.
impl<'de> VariantAccess<'de> for EnumDeserializer<'de> {
    type Error = Error;

    // If the `Visitor` expected this variant to be a unit variant, the input
    // should have been the plain string case handled in `deserialize_enum`.
    fn unit_variant(self) -> Result<()> {
        Err(Error::ExpectedAtom)
    }

    // Newtype variants are represented in JSON as `{ NAME: VALUE }` so
    // deserialize the value here.
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(Deserializer::from_term(self.term))
    }

    // Tuple variants are represented in JSON as `{ NAME: [DATA...] }` so
    // deserialize the sequence of data here.
    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let deserializer = Deserializer::from_term(self.term);
        de::Deserializer::deserialize_tuple(deserializer, len, visitor)
    }

    // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }` so
    // deserialize the inner map here.
    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let deserializer = Deserializer::from_term(self.term);
        de::Deserializer::deserialize_map(deserializer, visitor)
    }
}

struct VariantNameDeserializer<'a> {
    term: &'a Term,
}

impl<'a> VariantNameDeserializer<'a> {
    pub fn from_term(term: &'a Term) -> Self {
        VariantNameDeserializer { term: term }
    }
}

impl<'de, 'a: 'de> de::Deserializer<'de> for VariantNameDeserializer<'a> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.term {
            Term::Atom(atom) => visitor.visit_string(atom.name.to_camel_case()),
            _ => Err(Error::ExpectedAtom),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            bytes byte_buf option unit unit_struct newtype_struct seq tuple
            tuple_struct map struct enum identifier ignored_any
    }
}

mod private {
    // Some code I stole from serde.

    /// Avoid having to restate the generic types on `MapDeserializer`. The
    /// `Iterator::Item` contains enough information to figure out K and V.
    pub trait Pair<'a> {
        type First;
        type Second;
        fn split(self) -> &'a (Self::First, Self::Second);
    }

    impl<'a, A, B> Pair<'a> for &'a (A, B) {
        type First = A;
        type Second = B;
        fn split(self) -> &'a (A, B) {
            self
        }
    }

    pub type First<'a, T> = <T as Pair<'a>>::First;
    pub type Second<'a, T> = <T as Pair<'a>>::Second;
}

#[cfg(test)]
mod tests {
    use super::*;

    use eetf::{self, Term};
    use std::fmt::Debug;

    use serde::Serialize;

    // Helper function for tests. Runs things through our serializer then
    // decodes and returns.
    fn deserialize<T>(input: Term) -> T
    where
        T: DeserializeOwned,
    {
        let mut cursor = io::Cursor::new(vec![]);
        Term::encode(&input, &mut cursor).expect("encode failed");

        from_bytes(&cursor.into_inner()).expect("deserialize failed")
    }

    #[test]
    fn test_unsigned_ints_and_structs() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct TestStruct {
            unsigned8: u8,
            unsigned16: u16,
            unsigned32: u32,
            unsigned64: u64,
        }

        let result: TestStruct = deserialize(Term::Map(eetf::Map::from(vec![
            (
                Term::Atom(eetf::Atom::from("unsigned8")),
                Term::FixInteger(eetf::FixInteger::from(129)),
            ),
            (
                Term::Atom(eetf::Atom::from("unsigned16")),
                Term::FixInteger(eetf::FixInteger::from(65530)),
            ),
            (
                Term::Atom(eetf::Atom::from("unsigned32")),
                Term::BigInteger(eetf::BigInteger::from(65530)),
            ),
            (
                Term::Atom(eetf::Atom::from("unsigned64")),
                Term::BigInteger(eetf::BigInteger::from(65530)),
            ),
        ])));

        assert_eq!(
            result,
            TestStruct {
                unsigned8: 129,
                unsigned16: 65530,
                unsigned32: 65530,
                unsigned64: 65530,
            }
        )
    }

    #[test]
    fn test_signed_ints_and_tuple_structs() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct TestStruct(i8, i16, i32, i64);

        let result: TestStruct = deserialize(Term::Tuple(eetf::Tuple::from(vec![
            Term::FixInteger(eetf::FixInteger::from(-127)),
            Term::FixInteger(eetf::FixInteger::from(30000)),
            Term::FixInteger(eetf::FixInteger::from(65530)),
            Term::BigInteger(eetf::BigInteger::from(65530)),
        ])));
        assert_eq!(result, TestStruct(-127, 30000, 65530, 65530))
    }

    #[test]
    fn test_binaries_tuples_and_lists() {
        let result: (String, Vec<u8>) = deserialize(Term::Tuple(eetf::Tuple::from(vec![
            Term::Binary(eetf::Binary::from("ABCD".as_bytes())),
            Term::List(eetf::List::from(vec![
                Term::FixInteger(eetf::FixInteger::from(0)),
                Term::FixInteger(eetf::FixInteger::from(1)),
                Term::FixInteger(eetf::FixInteger::from(2)),
            ])),
        ])));

        assert_eq!(result, ("ABCD".to_string(), vec![0, 1, 2]))
    }

    #[test]
    fn test_option() {
        let nil_result: Option<u8> = deserialize(Term::Atom(eetf::Atom::from("nil")));
        let some_result: Option<u8> = deserialize(Term::FixInteger(eetf::FixInteger::from(0)));

        assert_eq!(nil_result, None);

        assert_eq!(some_result, Some(0));
    }

    #[test]
    fn test_unit_variant() {
        #[derive(Deserialize, Debug, PartialEq)]
        enum E {
            AnOption,
            AnotherOption,
        };

        let result: E = deserialize(Term::Atom(eetf::Atom::from("an_option")));

        assert_eq!(result, E::AnOption);
    }

    #[test]
    fn test_newtype_variant() {
        // Not 100% sure if this is a tuple variant or a newtype variant.
        // But whatever I guess?
        #[derive(Deserialize, Debug, PartialEq)]
        enum ErlResult {
            Ok(String),
        };

        let result: ErlResult = deserialize(Term::Tuple(eetf::Tuple::from(vec![
            Term::Atom(eetf::Atom::from("ok")),
            Term::Binary(eetf::Binary::from("test".as_bytes())),
        ])));

        assert_eq!(result, ErlResult::Ok("test".to_string()));
    }

    #[test]
    fn test_tuple_variant() {
        // Not 100% sure if this is a tuple variant or a newtype variant.
        // But whatever I guess?
        #[derive(Deserialize, Debug, PartialEq)]
        enum Testing {
            Ok(u8, u8),
        };

        let result: Testing = deserialize(Term::Tuple(eetf::Tuple::from(vec![
            Term::Atom(eetf::Atom::from("ok")),
            Term::Tuple(eetf::Tuple::from(vec![
                Term::FixInteger(eetf::FixInteger::from(1)),
                Term::FixInteger(eetf::FixInteger::from(2)),
            ])),
        ])));

        assert_eq!(result, Testing::Ok(1, 2));
    }
    // TODO: test actual maps, rather than structs.  Suspect they're broken.
}
