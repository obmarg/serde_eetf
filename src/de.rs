use eetf::Term;
use std::io::{self, Read};
use std::str;

use num_traits::cast::ToPrimitive;

use serde::de::{self, DeserializeOwned, Visitor};
use serde::private::de::size_hint;

use crate::error::{Error, Result};

/// Deserializes an `eetf::Term`
///
/// Generally you should use the from_bytes or from_reader functions instead.
pub struct Deserializer {
    term: Term,
}

impl Deserializer {
    pub fn new(term: Term) -> Self {
        Deserializer { term }
    }
}

impl<'de> de::IntoDeserializer<'de, Error> for Deserializer {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self {
        self
    }
}

// impl From<Term> for Deserializer {
//     fn from(term: Term) -> Self {
//         Deserializer::new(term)
//     }
// }

/// Deserializes some EETF from a Read
pub fn from_reader<R, T>(reader: R) -> Result<T>
where
    R: Read,
    T: DeserializeOwned,
{
    let term = Term::decode(reader)?;
    let deserializer = Deserializer::new(term);
    let t = de::Deserialize::deserialize(deserializer)?;
    Ok(t)
}

/// Deserializes some EETF from a slice of bytes.
pub fn from_bytes<T>(bytes: &[u8]) -> Result<T>
where
    T: DeserializeOwned,
{
    let cursor = io::Cursor::new(bytes);

    from_reader(cursor)
}

// Implementation methods for deserializer that require a lifetime.
impl Deserializer {
    fn deserialize_integer<'de, V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.term {
            Term::FixInteger(v) => visitor.visit_i32(v.value),
            Term::BigInteger(v) => visitor.visit_i64(v.to_i64().ok_or(Error::IntegerConvertError)?),
            _ => Err(Error::ExpectedFixInteger),
        }
    }
}

fn visit_term_seq<'de, V>(term: Vec<Term>, visitor: V) -> Result<V::Value>
where
    V: Visitor<'de>,
{
    let mut deserializer = SeqDeserializer::new(term);
    let value = visitor.visit_seq(&mut deserializer)?;
    Ok(value)
}

fn visit_term_map<'de, V>(term: Vec<(Term, Term)>, visitor: V) -> Result<V::Value>
where
    V: Visitor<'de>,
{
    let mut deserializer = MapDeserializer::new(term);
    let value = visitor.visit_map(&mut deserializer)?;
    Ok(value)
}

impl<'de> de::Deserializer<'de> for Deserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.term {
            Term::Atom(v) => visitor.visit_string(v.name),
            Term::FixInteger(v) => visitor.visit_i32(v.value),
            Term::BigInteger(_v) => unimplemented!(),
            Term::Float(v) => visitor.visit_f64(v.value),
            Term::Pid(_v) => unimplemented!(),
            Term::Port(_v) => unimplemented!(),
            Term::Reference(_v) => unimplemented!(),
            Term::ExternalFun(_v) => unimplemented!(),
            Term::InternalFun(_v) => unimplemented!(),
            Term::Binary(v) => visitor.visit_byte_buf(v.bytes),
            Term::BitBinary(_v) => unimplemented!(),
            Term::List(v) => visit_term_seq(v.elements, visitor),
            Term::ImproperList(_v) => unimplemented!(),
            Term::Tuple(v) => visit_term_seq(v.elements, visitor),
            Term::Map(v) => visit_term_map(v.entries, visitor),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.term {
            Term::Atom(b) => match b.name.as_ref() {
                "true" => visitor.visit_bool(true),
                "false" => visitor.visit_bool(false),
                _ => Err(Error::InvalidBoolean),
            },
            _ => Err(Error::ExpectedBoolean),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_integer(visitor)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_integer(visitor)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_integer(visitor)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_integer(visitor)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_integer(visitor)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_integer(visitor)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_integer(visitor)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_integer(visitor)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_f64(visitor)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.term {
            Term::Float(v) => visitor.visit_f64(v.value),
            _ => Err(Error::ExpectedFloat),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.term {
            Term::Atom(v) => visitor.visit_string(v.name),
            Term::Binary(v) => visitor.visit_byte_buf(v.bytes),
            _ => Err(Error::ExpectedBinary),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.term {
            Term::Binary(v) => visitor.visit_byte_buf(v.bytes),
            _ => Err(Error::ExpectedBinary),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.term {
            Term::Atom(v) if &v.name == "nil" => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.term {
            Term::Atom(v) if &v.name == "nil" => visitor.visit_unit(),
            _ => Err(Error::ExpectedNil),
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
            Term::List(v) => visit_term_seq(v.elements, visitor),
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
            Term::Tuple(v) => {
                if v.elements.len() != len {
                    return Err(Error::WrongTupleLength);
                }
                visit_term_seq(v.elements, visitor)
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
            Term::Map(v) => visit_term_map(v.entries, visitor),
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
        match self.term {
            Term::List(v) => visit_term_seq(v.elements, visitor),
            Term::Map(v) => visit_term_map(v.entries, visitor),
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
        let (variant, value) = match self.term {
            Term::Map(value) => {
                let mut iter = value.entries.into_iter();
                let (variant, value) = match iter.next() {
                    Some(v) => v,
                    None => {
                        return Err(de::Error::invalid_value(
                            de::Unexpected::Map,
                            &"map with a single key",
                        ));
                    }
                };
                // enums are encoded in json as maps with a single key:value pair
                if iter.next().is_some() {
                    return Err(de::Error::invalid_value(
                        de::Unexpected::Map,
                        &"map with a single key",
                    ));
                }
                (variant, Some(value))
            }
            s @ Term::Atom(_) => (s, None),
            _ => {
                return Err(Error::ExpectedAtomOrTuple);
            }
        };

        visitor.visit_enum(EnumDeserializer::new(variant, value))
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

struct SeqDeserializer {
    iter: <Vec<Term> as IntoIterator>::IntoIter,
}

impl SeqDeserializer {
    fn new(vec: Vec<Term>) -> Self {
        SeqDeserializer {
            iter: vec.into_iter(),
        }
    }
}

impl<'de> de::Deserializer<'de> for SeqDeserializer {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let len = self.iter.len();
        if len == 0 {
            visitor.visit_unit()
        } else {
            let ret = visitor.visit_seq(&mut self)?;
            let remaining = self.iter.len();
            if remaining == 0 {
                Ok(ret)
            } else {
                Err(de::Error::invalid_length(len, &"fewer elements in array"))
            }
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de> de::SeqAccess<'de> for SeqDeserializer {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(Deserializer::new(value)).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        size_hint::from_bounds(&self.iter)
    }
}

struct MapDeserializer {
    iter: <Vec<(Term, Term)> as IntoIterator>::IntoIter,
    value: Option<Term>,
}

impl MapDeserializer {
    fn new(map: Vec<(Term, Term)>) -> Self {
        MapDeserializer {
            iter: map.into_iter(),
            value: None,
        }
    }
}

impl<'de> de::MapAccess<'de> for MapDeserializer {
    type Error = Error;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(Deserializer::new(key)).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(Deserializer::new(value)),
            None => Err(de::Error::custom("value is missing")),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        size_hint::from_bounds(&self.iter)
    }
}

impl<'de> de::Deserializer<'de> for MapDeserializer {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

pub struct EnumDeserializer {
    variant: Term,
    value: Option<Term>,
}

impl EnumDeserializer {
    pub fn new(variant: Term, value: Option<Term>) -> EnumDeserializer {
        EnumDeserializer { variant, value }
    }
}

impl<'de> de::EnumAccess<'de> for EnumDeserializer {
    type Error = Error;
    type Variant = VariantDeserializer;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        let visitor = VariantDeserializer { value: self.value };
        seed.deserialize(Deserializer::new(self.variant))
            .map(|v| (v, visitor))
    }
}

pub struct VariantDeserializer {
    value: Option<Term>,
}

impl<'de> de::VariantAccess<'de> for VariantDeserializer {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        match self.value {
            Some(value) => de::Deserialize::deserialize(Deserializer::new(value)),
            None => Ok(()),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.value {
            Some(value) => seed.deserialize(Deserializer::new(value)),
            None => Err(Error::ExpectedTuple),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Some(Term::Tuple(v)) => {
                de::Deserializer::deserialize_any(SeqDeserializer::new(v.elements), visitor)
            }
            _ => Err(Error::ExpectedTuple),
        }
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Some(Term::Map(v)) => {
                de::Deserializer::deserialize_any(MapDeserializer::new(v.entries), visitor)
            }
            Some(Term::List(v)) => {
                de::Deserializer::deserialize_any(SeqDeserializer::new(v.elements), visitor)
            }
            _ => Err(Error::ExpectedMap),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use eetf::{self, Term};
    use std::convert::TryFrom;

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
    fn test_buffered_adjacently_tagged() {
        #[derive(Serialize, Deserialize, Debug)]
        #[serde(tag = "type", content = "data")]
        enum E {
            Float(f32),
        }

        let result: E = deserialize(Term::Map(eetf::Map::from(vec![
            // Content must come first to trigger serde's buffering
            (
                Term::Atom(eetf::Atom::from("data")),
                Term::Float(eetf::Float::try_from(159.1).unwrap()),
            ),
            (
                Term::Atom(eetf::Atom::from("type")),
                Term::Atom(eetf::Atom::from("Float")),
            ),
        ])));

        match result {
            E::Float(f) => assert_eq!(f, 159.1),
        }
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
        #[serde(rename_all = "snake_case")]
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
        #[serde(rename_all = "snake_case")]
        enum ErlResult {
            Ok(String),
        };

        let result: ErlResult = deserialize(Term::Map(eetf::Map::from(vec![(
            Term::Atom(eetf::Atom::from("ok")),
            Term::Binary(eetf::Binary::from("test".as_bytes())),
        )])));

        assert_eq!(result, ErlResult::Ok("test".to_string()));
    }

    #[test]
    fn test_tuple_variant() {
        // Not 100% sure if this is a tuple variant or a newtype variant.
        // But whatever I guess?
        #[derive(Deserialize, Debug, PartialEq)]
        #[serde(rename_all = "snake_case")]
        enum Testing {
            Ok(u8, u8),
        };

        let result: Testing = deserialize(Term::Map(eetf::Map::from(vec![(
            Term::Atom(eetf::Atom::from("ok")),
            Term::Tuple(eetf::Tuple::from(vec![
                Term::FixInteger(eetf::FixInteger::from(1)),
                Term::FixInteger(eetf::FixInteger::from(2)),
            ])),
        )])));

        assert_eq!(result, Testing::Ok(1, 2));
    }
    // TODO: test actual maps, as well as structs.  Suspect they're broken.
    // some quickcheck based roundtrip tests would also be great.
}
