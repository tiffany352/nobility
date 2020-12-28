//! NBT decoding library.
//!
//! Currently only supports decoding into a tag tree. In the future,
//! this may support encoding and zero-allocation decoding.

extern crate byteorder;

use byteorder::{BigEndian, ReadBytesExt};
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::mem;
use std::str::from_utf8;

/// Failures which can occur while parsing an NBT document.
#[derive(Debug)]
pub enum Error {
    /// End of file happens when the document is truncated, i.e. we were
    /// expecting some data to follow after something, and then the file
    /// ended instead. In particular, this can happen when:
    ///
    /// - Any primitive type is not followed by enough bytes to
    /// construct the primitive type (TAG_Byte, TAG_Short, TAG_Int,
    /// TAG_Long, TAG_Float, TAG_Double).
    ///
    /// - A TAG_Byte_Array, TAG_String, or TAG_Int_Array is not followed
    /// by as many elements as it says it is.
    ///
    /// - A TAG_List does not have as many elements as it says it does, or
    /// we get an EOF while attempting to parse an element.
    ///
    /// - A TAG_Compound does not have a TAG_End to terminate it, or we
    /// get an EOF while attempting to parse a tag.
    EOF,
    /// This happens when there is an unknown tag type in the
    /// stream. This can happen if Mojang adds new tag types, if a
    /// document has third party tag types, if the file is corrupted, or
    /// if there's a bug in the library.
    UnknownTag,
    /// This happens when we found a TAG_End where we shouldn't
    /// have. TAG_End is only supposed to be found after having a
    /// TAG_Compound, to terminate it. Places we can find this include
    /// as the root tag of a document and inside of a List.
    UnexpectedEndTag,
    /// This should never happen. It is when ByteOrder is passed an
    /// insufficiently large buffer to parse, or when an std::io::Error
    /// happens (which should never happen, because we don't touch
    /// std::io).
    ByteOrderError(byteorder::Error),
    /// This happens when we attempt to parse a TAG_String, and it turns
    /// out to be invalid UTF-8. NBT specifies that TAG_String contain
    /// UTF-8 text, so any document with invalid UTF-8 is malformed, and
    /// so we reject it.
    Utf8Error(std::str::Utf8Error),
}

/// Straight mapping of the tags stored in the actual NBT documents.
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TagType {
    End = 0,
    Byte = 1,
    Short = 2,
    Int = 3,
    Long = 4,
    Float = 5,
    Double = 6,
    ByteArray = 7,
    String = 8,
    List = 9,
    Compound = 10,
    IntArray = 11,
}

const MAX_TYPE: u8 = 12;

impl TagType {
    pub fn from_raw(ty: u8) -> Option<TagType> {
        if ty < MAX_TYPE {
            Some(unsafe { mem::transmute::<u8, TagType>(ty) })
        } else {
            None
        }
    }

    pub fn to_raw(&self) -> u8 {
        unsafe { mem::transmute::<TagType, u8>(*self) }
    }
}

/// Represents a TAG_Int_Array type.
///
/// We have to handle these specially because integers are stored in
/// big endian in NBT.
#[derive(PartialEq)]
pub struct IntArray<'a>(&'a [u8]);

impl<'a> IntArray<'a> {
    pub fn len(&self) -> usize {
        self.0.len() / 4
    }

    pub fn get(&self, index: usize) -> Option<i32> {
        if index < self.len() {
            (&self.0[index * 4..index * 4 + 4])
                .read_i32::<BigEndian>()
                .ok()
        } else {
            None
        }
    }

    pub fn to_vec(&self) -> Vec<i32> {
        let mut v = vec![];
        v.reserve(self.len());
        for i in 0..self.len() {
            v.push(self.get(i).unwrap())
        }
        v
    }
}

/// An enumeration that represents all possible values that can be
/// encoded in an NBT document.
///
/// Only List and Compound use heap allocations. Most data is borrowed
/// from the buffer passed to `decode()`.
///
/// TAG_End is omitted because it is not a proper value type. It is
/// simply a terminator for TAG_Compound.
///
/// All integer values are signed, because NBT's original
/// implementation language is Java, which does not have unsigned
/// integers.
#[derive(PartialEq)]
pub enum Tag<'a> {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    /// An array of arbitrary binary data.
    ByteArray(&'a [u8]),
    /// A UTF-8 encoded string.
    String(&'a str),
    /// A list may only contain one type. Unfortunately, this is
    /// difficult to express without code duplication. It may be taken
    /// as an invariant that all of the Tags in a list will be the same
    /// type.
    ///
    /// One thing to note is that lists inside of a list are not
    /// required to be the same type. For example, you could have a list
    /// containing a list of ints, and a list of bytes.
    List(Vec<Tag<'a>>),
    /// The NBT specification states that no two tags within a
    /// TAG_Compound may have the same name, and does not define any
    /// kind of ordering. This makes a HashMap an ideal data structure.
    Compound(HashMap<&'a str, Tag<'a>>),
    /// It is not clear why this type exists, as its purpose is served
    /// perfectly well by a List of Int. It doesn't seem to have been
    /// added before the List tag, since it has a higher tag value.
    IntArray(IntArray<'a>),
}

impl<'a> Tag<'a> {
    pub fn get_type(&self) -> TagType {
        match self {
            &Tag::Byte(_) => TagType::Byte,
            &Tag::Short(_) => TagType::Short,
            &Tag::Int(_) => TagType::Int,
            &Tag::Long(_) => TagType::Long,
            &Tag::Float(_) => TagType::Float,
            &Tag::Double(_) => TagType::Double,
            &Tag::ByteArray(_) => TagType::ByteArray,
            &Tag::String(_) => TagType::String,
            &Tag::List(_) => TagType::List,
            &Tag::Compound(_) => TagType::Compound,
            &Tag::IntArray(_) => TagType::IntArray,
        }
    }

    /// A convenience function for converting any of NBT's integer types
    /// (TAG_Byte, TAG_Short, TAG_Int, TAG_Long) into an i64.
    pub fn to_integer(&self) -> Option<i64> {
        match self {
            &Tag::Byte(v) => Some(v as i64),
            &Tag::Short(v) => Some(v as i64),
            &Tag::Int(v) => Some(v as i64),
            &Tag::Long(v) => Some(v as i64),
            _ => None,
        }
    }
}

impl<'a> Debug for Tag<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            &Tag::Byte(v) => write!(fmt, "{}i8", v),
            &Tag::Short(v) => write!(fmt, "{}i16", v),
            &Tag::Int(v) => write!(fmt, "{}i32", v),
            &Tag::Long(v) => write!(fmt, "{}i64", v),
            &Tag::Float(v) => write!(fmt, "{}f32", v),
            &Tag::Double(v) => write!(fmt, "{}f64", v),
            &Tag::ByteArray(v) => write!(fmt, "[{} bytes]", v.len()),
            &Tag::String(s) => write!(fmt, "'{}'", s),
            &Tag::List(ref v) => {
                let mut debug = fmt.debug_list();
                for e in v.iter() {
                    debug.entry(e);
                }
                debug.finish()
            }
            &Tag::Compound(ref h) => {
                let mut debug = fmt.debug_map();
                let mut pairs: Vec<_> = h.iter().collect();
                pairs.sort_by(|&(a, _), &(b, _)| a.cmp(b));
                for &(k, v) in pairs.iter() {
                    debug.entry(k, v);
                }
                debug.finish()
            }
            &Tag::IntArray(ref v) => write!(fmt, "[{} ints]", v.len()),
        }
    }
}

struct Reader<'a> {
    buffer: &'a [u8],
    position: usize,
}

impl<'a> Reader<'a> {
    fn new(buffer: &'a [u8]) -> Reader<'a> {
        Reader {
            buffer: buffer,
            position: 0,
        }
    }

    fn advance(&mut self, n: usize) -> Result<&'a [u8], Error> {
        if self.buffer.len() < self.position + n {
            Err(Error::EOF)
        } else {
            let slice = &self.buffer[self.position..self.position + n];
            self.position += n;
            Ok(slice)
        }
    }
}

fn decode_value<'a>(tag: TagType, reader: &mut Reader<'a>) -> Result<Tag<'a>, Error> {
    match tag {
        TagType::End => Err(Error::UnexpectedEndTag),
        TagType::Byte => reader.advance(1).map(|b| Tag::Byte(b[0] as i8)),
        TagType::Short => reader.advance(2).and_then(|mut v| {
            Ok(Tag::Short(try!(v
                .read_i16::<BigEndian>()
                .map_err(Error::ByteOrderError))))
        }),
        TagType::Int => reader.advance(4).and_then(|mut v| {
            Ok(Tag::Int(try!(v
                .read_i32::<BigEndian>()
                .map_err(Error::ByteOrderError))))
        }),
        TagType::Long => reader.advance(8).and_then(|mut v| {
            Ok(Tag::Long(try!(v
                .read_i64::<BigEndian>()
                .map_err(Error::ByteOrderError))))
        }),
        TagType::Float => reader.advance(4).and_then(|mut v| {
            Ok(Tag::Float(try!(v
                .read_f32::<BigEndian>()
                .map_err(Error::ByteOrderError))))
        }),
        TagType::Double => reader.advance(8).and_then(|mut v| {
            Ok(Tag::Double(try!(v
                .read_f64::<BigEndian>()
                .map_err(Error::ByteOrderError))))
        }),
        TagType::ByteArray => {
            let len = {
                let arr = reader.advance(4);
                try!(arr.and_then(|mut v| v.read_u32::<BigEndian>().map_err(Error::ByteOrderError)))
            };
            Ok(Tag::ByteArray(try!(reader.advance(len as usize))))
        }
        TagType::String => {
            let len = {
                let arr = reader.advance(2);
                try!(arr.and_then(|mut v| v.read_u16::<BigEndian>().map_err(Error::ByteOrderError)))
            };
            Ok(Tag::String(try!(reader
                .advance(len as usize)
                .and_then(|v| from_utf8(v).map_err(Error::Utf8Error)))))
        }
        TagType::List => {
            let mut list = vec![];
            let tag = try!(reader
                .advance(1)
                .and_then(|v| TagType::from_raw(v[0]).ok_or(Error::UnknownTag)));
            let len = try!(reader
                .advance(4)
                .and_then(|mut v| v.read_u32::<BigEndian>().map_err(Error::ByteOrderError)));
            list.reserve(len as usize);
            for _ in 0..len {
                list.push(try!(decode_value(tag, reader)));
            }
            Ok(Tag::List(list))
        }
        TagType::Compound => {
            let mut compound = HashMap::new();
            loop {
                match decode_full(reader) {
                    Ok(None) => break,
                    Ok(Some((key, value))) => {
                        compound.insert(key, value);
                    }
                    Err(e) => return Err(e),
                }
            }
            Ok(Tag::Compound(compound))
        }
        TagType::IntArray => {
            let len = {
                let arr = reader.advance(4);
                try!(arr.and_then(|mut v| v.read_u32::<BigEndian>().map_err(Error::ByteOrderError)))
            };
            Ok(Tag::IntArray(IntArray(try!(
                reader.advance(len as usize * 4)
            ))))
        }
    }
}

fn decode_full<'a>(reader: &mut Reader<'a>) -> Result<Option<(&'a str, Tag<'a>)>, Error> {
    let tag = try!(reader
        .advance(1)
        .and_then(|v| TagType::from_raw(v[0]).ok_or(Error::UnknownTag)));
    if tag == TagType::End {
        return Ok(None);
    }
    let name_len = try!(reader
        .advance(2)
        .and_then(|mut v| v.read_u16::<BigEndian>().map_err(Error::ByteOrderError)));
    let name = try!(reader
        .advance(name_len as usize)
        .and_then(|v| from_utf8(v).map_err(Error::Utf8Error)));
    Ok(Some((name, try!(decode_value(tag, reader)))))
}

/// Decode a buffer into an NBT tag tree.
///
/// For whatever reason, the name of a tag is an intrinsic property of
/// its value, which is why this function returns `(&str, Tag)`. All
/// NBT documents have a root tag, which is almost always a
/// TAG_Compound, which contains the entire document. This tag's name
/// is usually specific to the type of document. For example,
/// WorldEdit schematics are always named "Schematic".
///
/// This function can fail for any number of reasons, and will return
/// an Error in that case. This parser should be able to handle
/// anything you can throw at it, so if it panics, that is a bug.
///
/// Future versions may support allocation-free parsing, using a
/// visitor instead of returning a tag tree.
pub fn decode<'a>(buffer: &'a [u8]) -> Result<(&'a str, Tag<'a>), Error> {
    let mut reader = Reader::new(buffer);
    decode_full(&mut reader).and_then(|v| v.ok_or(Error::UnexpectedEndTag))
}
