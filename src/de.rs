use std::io::{Read};
use std::str;
use std::iter;
use std::marker::PhantomData;
use eetf::{Term};

use num_traits::cast::FromPrimitive;

use serde::de::{
    self, Deserialize, DeserializeSeed, DeserializeOwned, EnumAccess, IntoDeserializer,
    MapAccess, SeqAccess, VariantAccess, Visitor,
};

use error::{Error, Result};

use self::private::*;


pub struct Deserializer<'a> {
    term: &'a Term
}

impl<'a> Deserializer<'a> {
    pub fn from_term(term: &'a Term) -> Self
    {
        Deserializer{ term: term }
    }
}

trait IntoEetfDeserializer {
    fn into_deserializer<'a>(&'a self) -> Deserializer<'a>;
}

impl IntoEetfDeserializer for Term {
    fn into_deserializer<'de>(&'de self) -> Deserializer<'de> {
        Deserializer::from_term(self)
    }
}

impl<'de, 'a: 'de> From<&'a Term> for Deserializer<'de> {
    fn from(term: &'a Term) -> Self {
        Deserializer::from_term(term)
    }
}

pub fn from_reader<R, T>(reader: R) -> Result<T>
    where R: Read, T: DeserializeOwned
{
    let term = Term::decode(reader)?;
    let deserializer = Deserializer::from_term(&term);
    let t = T::deserialize(&deserializer)?;
    Ok(t)
}

// Implementation methods for deserializer that require a lifetime.
impl<'de> Deserializer<'de> {
    fn parse_fix_integer<T>(&self) -> Result<T>
    where T: FromPrimitive
    {
        match self.term {
            Term::FixInteger(fix_int) =>
                if let Some(num) = T::from_i32(fix_int.value) {
                    Ok(num)
                } else {
                    Err(Error::IntegerConvertError)
                }
            _ =>
                Err(Error::ExpectedFixInteger)
        }
    }

    fn parse_float<T>(&self) -> Result<T>
    where T: FromPrimitive
    {
        match self.term {
            Term::Float(float) =>
                if let Some(num) = T::from_f64(float.value) {
                    Ok(num)
                } else {
                    Err(Error::IntegerConvertError)
                }
            _ =>
                Err(Error::ExpectedFloat)
        }
    }

    fn parse_binary(&self) -> Result<&[u8]> {
        match self.term {
            Term::Binary(binary) =>
                Ok(&binary.bytes),
            _ =>
                Err(Error::ExpectedBinary)
        }
    }

    fn parse_string(&self) -> Result<String> {
        match self.parse_binary() {
            Ok(bytes) => {
                str::from_utf8(&bytes)
                    .map(|s| { s.to_string()})
                    .or(Err(Error::Utf8DecodeError))
            }
            Err(e) => Err(e)
        }
    }
}

impl<'de> de::Deserializer<'de> for &'de Deserializer<'de> {
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

            _ =>
                Err(Error::ExpectedBoolean)
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.parse_fix_integer()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.parse_fix_integer()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.parse_fix_integer()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.parse_fix_integer()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.parse_fix_integer()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.parse_fix_integer()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.parse_fix_integer()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.parse_fix_integer()?)
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
            other =>
                visitor.visit_some(self)
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
            other =>
                Err(Error::ExpectedNil)
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value>
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
            }
            _ =>
                Err(Error::ExpectedList)
        }
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.term {
            Term::Tuple(tuple) => {
                if tuple.elements.len() != len {
                    return Err(Error::WrongTupleLength)
                }
                let seq_deserializer = ListDeserializer::new(tuple.elements.iter());
                visitor.visit_seq(seq_deserializer)
            }
            _ =>
                Err(Error::ExpectedTuple)
        }
    }

    // Tuple structs look just like sequences in EETF.
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.term {
            Term::Map(map) => {
                let map_deserializer = MapDeserializer::new(map.entries.iter());
                let result = visitor.visit_map(map_deserializer);
                match map_deserializer.end() {
                    Ok(()) => result,
                    Err(e) => Err(e)
                }
            }
            _ =>
                Err(Error::ExpectedMap)
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
        self.deserialize_map(visitor)
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
                visitor.visit_enum(atom.name.into_deserializer())
            }
            Term::Tuple(tuple) => {
                match tuple.elements.as_slice() {
                    [variant_term, value_term] => {
                        visitor.visit_enum(EnumDeserializer::new(&variant_term, &value_term))
                    }
                    _ => {
                        return Err(Error::MisSizedVariantTuple)
                    }
                }
                // New type variant has {variant, value}
                // Tuple variant has {variant, {tuple_data}}
                // struct variant has {variant, %{map_data}}
            }
            _ =>
                Err(Error::ExpectedAtomOrTuple)
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.term {
            Term::Atom(atom) =>
                visitor.visit_str(&atom.name),
            _ =>
                Err(Error::ExpectedAtom)
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct ListDeserializer<I>
    where I: Iterator
{
    iter: iter::Fuse<I>
}

impl<I> ListDeserializer<I>
    where I: Iterator
{
    fn new(iter: I) -> Self {
        ListDeserializer{iter: iter.fuse()}
    }

    fn end(&self) -> Result<()> {
        if self.iter.count() == 0 {
            Ok(())
        } else {
            Err(Error::TooManyItems)
        }
    }
}

impl<'de, 'a: 'de, I> SeqAccess<'de> for ListDeserializer<I>
where I: Iterator<Item = &'a Term>,
{
    type Error = Error;

    fn next_element_seed<V>(&mut self, seed: V) -> Result<Option<V::Value>>
    where
        V: de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(term) => {
                seed.deserialize(&Deserializer::from_term(term)).map(Some)
            }
            None => Ok(None)
        }
    }
}

struct MapDeserializer<'de, I, T>
where
    I: Iterator<Item = T>,
    T: Pair
{
    items: iter::Fuse<I>,
    current_value: Option<T::Second>,
    lifetime: PhantomData<&'de ()>,
}

impl<'de, I, T> MapDeserializer<'de, I, T>
where I: Iterator<Item = T>
    , T: Pair
{
    fn new(iter: I) -> Self {
        MapDeserializer { items: iter.fuse(), current_value: None, lifetime: PhantomData }
    }

    fn end(&self) -> Result<()> {
        if self.items.count() == 0 {
            Ok(())
        } else {
            Err(Error::TooManyItems)
        }
    }
}



impl<'de, I, T> MapAccess<'de> for MapDeserializer<'de, I, T>
where I: Iterator<Item = T>,
      T: Pair,
      First<I::Item>: IntoEetfDeserializer,
      Second<I::Item>: IntoEetfDeserializer

        // TODO: We might need an into serializer constraint on the pair.
        // But for now fuck it.
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
                let deserializer = key.into_deserializer();

                seed.deserialize(&deserializer).map(Some)
            }
            None => Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        if let Some(value) = self.current_value {
            self.current_value = None;
            seed.deserialize(&Deserializer::from_term(value))
        } else {
            panic!("MapDeserializer.next_value_seed was called before next_key_seed")
        }
    }
}

struct EnumDeserializer<'de> {
    variant: &'de Term,
    term: &'de Term
}

impl<'de> EnumDeserializer<'de> {
    fn new(variant: &'de Term, term: &'de Term) -> Self {
        EnumDeserializer{
            variant : variant,
            term : term,
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
        let val = seed.deserialize(&Deserializer::from_term(self.variant))?;
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
        seed.deserialize(&Deserializer::from_term(self.term))
    }

    // Tuple variants are represented in JSON as `{ NAME: [DATA...] }` so
    // deserialize the sequence of data here.
    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let deserializer = Deserializer::from_term(self.term);
        de::Deserializer::deserialize_tuple(&deserializer, len, visitor)
    }

    // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }` so
    // deserialize the inner map here.
    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let deserializer = Deserializer::from_term(self.term);
        de::Deserializer::deserialize_map(&deserializer, visitor)
    }
}

mod private {
    // Some code I stole from serde.

    /// Avoid having to restate the generic types on `MapDeserializer`. The
    /// `Iterator::Item` contains enough information to figure out K and V.
    pub trait Pair {
        type First;
        type Second;
        fn split(self) -> (Self::First, Self::Second);
    }

    impl<A, B> Pair for (A, B) {
        type First = A;
        type Second = B;
        fn split(self) -> (A, B) {
            self
        }
    }

    pub type First<T> = <T as Pair>::First;
    pub type Second<T> = <T as Pair>::Second;
}
