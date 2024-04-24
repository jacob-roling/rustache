use serde::{ser, Serialize};
use std::result::Result as StdResult;
use std::{collections::HashMap, fmt::Display};
use thiserror::Error;

use super::{to_value, Value};

#[derive(Debug, Error)]
pub enum Error {
    #[error("unsupported type: {0}")]
    UnsupportedType(String),
    #[error("key is not string")]
    KeyIsNotString,
    #[error("missing elements")]
    MissingElements,
    #[error("{0}")]
    Message(String),
}

impl serde::ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

pub type Result<T> = StdResult<T, Error>;

#[derive(Default)]
pub struct Encoder;

impl Encoder {
    pub fn new() -> Encoder {
        Encoder::default()
    }
}

impl serde::Serializer for Encoder {
    type Ok = Value;
    type Error = Error;
    type SerializeSeq = SerializeVec;
    type SerializeTuple = SerializeVec;
    type SerializeTupleStruct = SerializeVec;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeMap;
    type SerializeStructVariant = SerializeStructVariant;

    fn serialize_bool(self, v: bool) -> Result<Value> {
        return Ok(Value::Bool(v));
    }

    fn serialize_i8(self, v: i8) -> Result<Value> {
        return Ok(Value::String(v.to_string()));
    }

    fn serialize_i16(self, v: i16) -> Result<Value> {
        return Ok(Value::String(v.to_string()));
    }

    fn serialize_i32(self, v: i32) -> Result<Value> {
        return Ok(Value::String(v.to_string()));
    }

    fn serialize_i64(self, v: i64) -> Result<Value> {
        return Ok(Value::String(v.to_string()));
    }

    fn serialize_u8(self, v: u8) -> Result<Value> {
        return Ok(Value::String(v.to_string()));
    }

    fn serialize_u16(self, v: u16) -> Result<Value> {
        return Ok(Value::String(v.to_string()));
    }

    fn serialize_u32(self, v: u32) -> Result<Value> {
        return Ok(Value::String(v.to_string()));
    }

    fn serialize_u64(self, v: u64) -> Result<Value> {
        return Ok(Value::String(v.to_string()));
    }

    fn serialize_f32(self, v: f32) -> Result<Value> {
        return Ok(Value::String(v.to_string()));
    }

    fn serialize_f64(self, v: f64) -> Result<Value> {
        return Ok(Value::String(v.to_string()));
    }

    fn serialize_char(self, v: char) -> Result<Value> {
        return Ok(Value::String(v.to_string()));
    }

    fn serialize_str(self, v: &str) -> Result<Value> {
        return Ok(Value::String(v.to_string()));
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Value> {
        let vec = v.iter().map(|&b| Value::String(b.to_string())).collect();
        Ok(Value::Vec(vec))
    }

    fn serialize_none(self) -> Result<Value> {
        return Ok(Value::None);
    }

    fn serialize_some<T>(self, value: &T) -> Result<Value>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Value> {
        Ok(Value::None)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Value> {
        Ok(Value::None)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Value> {
        Ok(Value::String(variant.to_string()))
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Value>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Value>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(SerializeTupleVariant {
            name: String::from(variant),
            vec: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(SerializeMap {
            map: HashMap::with_capacity(len.unwrap_or(0)),
            next_key: None,
        })
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(SerializeStructVariant {
            name: String::from(variant),
            map: HashMap::with_capacity(len),
        })
    }
}

#[doc(hidden)]
pub struct SerializeVec {
    vec: Vec<Value>,
}

#[doc(hidden)]
pub struct SerializeTupleVariant {
    name: String,
    vec: Vec<Value>,
}

#[doc(hidden)]
pub struct SerializeMap {
    map: HashMap<String, Value>,
    next_key: Option<String>,
}

#[doc(hidden)]
pub struct SerializeStructVariant {
    name: String,
    map: HashMap<String, Value>,
}

impl ser::SerializeSeq for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.vec.push(to_value(&value)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Vec(self.vec))
    }
}

impl ser::SerializeTuple for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleStruct for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleVariant for SerializeTupleVariant {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.vec.push(to_value(&value)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        let mut object = HashMap::new();

        object.insert(self.name, Value::Vec(self.vec));

        Ok(Value::Object(object))
    }
}

impl ser::SerializeMap for SerializeMap {
    type Ok = Value;
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
    where
        T: Serialize,
    {
        match to_value(key)? {
            Value::String(s) => {
                self.next_key = Some(s);
                Ok(())
            }
            _ => Err(Error::KeyIsNotString),
        }
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        // Taking the key should only fail if this gets called before
        // serialize_key, which is a bug in the library.
        let key = self.next_key.take().ok_or(Error::MissingElements)?;
        self.map.insert(key, to_value(&value)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Object(self.map))
    }
}

impl ser::SerializeStruct for SerializeMap {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        ser::SerializeMap::serialize_key(self, key)?;
        ser::SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> Result<Value> {
        ser::SerializeMap::end(self)
    }
}

impl ser::SerializeStructVariant for SerializeStructVariant {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.map.insert(String::from(key), to_value(&value)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        let mut object = HashMap::new();

        object.insert(self.name, Value::Object(self.map));

        Ok(Value::Object(object))
    }
}
