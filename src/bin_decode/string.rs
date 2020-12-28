use crate::bin_decode::{NbtParse, ParseError, Reader};
use byteorder::{BigEndian, ByteOrder};
use cesu8::{from_java_cesu8, Cesu8DecodingError};
use std::borrow::Cow;
use std::fmt;

/// NBT stores strings in Java's modified version of [CESU-8][2] called
/// ["Modified UTF-8"][1]. As a result, documents can't be decoded into
/// &str references in all cases. This type stores a reference to a
/// potentially invalid string in this format.
///
/// [1]:
/// https://docs.oracle.com/javase/8/docs/api/java/io/DataInput.html#modified-utf-8
/// [2]: https://en.wikipedia.org/wiki/CESU-8
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
    /// Attempts to parse the string into UTF-8 using the cesu8 crate.
    /// An error will be returned if this fails, which should only
    /// happen if the data contained is invalid CESU-8.
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