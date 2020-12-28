use crate::parser::{NbtParse, ParseError, Reader};
use byteorder::{BigEndian, ByteOrder};
use cesu8::{from_java_cesu8, Cesu8DecodingError};
use std::borrow::Cow;
use std::fmt;

/// NBT stores strings in a format called ["Modified UTF-8"][1], which
/// requires some special handling.
///
/// [1]: https://docs.oracle.com/javase/8/docs/api/java/io/DataInput.html#modified-utf-8
#[derive(Copy, Clone)]
pub struct NbtString<'a> {
    data: &'a [u8],
}

impl<'a> NbtParse<'a> for NbtString<'a> {
    fn read(reader: &mut Reader<'a>) -> Result<Self, ParseError> {
        let length = BigEndian::read_u16(reader.advance(2)?);
        let data = reader.advance(length as usize)?;
        Ok(NbtString { data })
    }
}

impl<'a> NbtString<'a> {
    pub fn decode(&self) -> Result<Cow<str>, Cesu8DecodingError> {
        from_java_cesu8(self.data)
    }
}

impl<'a, T> PartialEq<T> for NbtString<'a>
where
    T: AsRef<str>,
{
    fn eq(&self, other: &T) -> bool {
        if let Ok(result) = self.decode() {
            result == other.as_ref()
        } else {
            false
        }
    }
}

impl<'a> fmt::Debug for NbtString<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if let Ok(result) = self.decode() {
            write!(fmt, "NbtString({:?})", result)
        } else {
            write!(fmt, "NbtString({:?})", self.data)
        }
    }
}
