//! Decoder for the NBT binary format. This module is based around the
//! idea that you won't store the objects, instead they will be walked
//! through to build up some other data structure.
//!
//! As a result, almost all of the types here use borrows into the
//! original data buffer, rather than copying into a Vec. The main
//! exception is bookkeeping where necessary, such as when parsing
//! Compound tags.
//!
//! # Example
//!
//! ```rust
//! # use std::error::Error;
//! #
//! # fn main() -> Result<(), Box<dyn Error>> {
//! use nobility::bin_decode::Document;
//! use std::fs::File;
//! use std::io::Read;
//!
//! let mut file = File::open("files/hello_world.nbt")?;
//! let mut data = vec![];
//! file.read_to_end(&mut data)?;
//! let cursor = std::io::Cursor::new(data);
//!
//! // Either copies the data (plaintext) or decompresses it (gzip).
//! let doc = Document::load(cursor)?;
//!
//! // Returns the root tag's name, and the root tag (always a Compound tag).
//! // Both of these are borrowing the data inside the Document.
//! let (name, root) = doc.parse()?;
//! println!("name: {}", name.decode()?);
//! println!("{:#?}", root);
//! #
//! # Ok(())
//! # }
//! ```

use crate::TagType;
use byteorder::{BigEndian, ByteOrder};
use flate2::read::GzDecoder;
use std::fmt;
use std::io::Error as IoError;
use std::io::Read;

mod array;
mod compound;
mod internal;
mod list;
mod string;

pub use array::{IntArray, LongArray, NbtArray, NbtArrayIter};
pub use compound::{Compound, Entry};
pub(crate) use internal::{NbtParse, Reader};
pub use list::{
    ByteArrayList, CompoundList, DoubleList, FloatList, IntArrayList, IntList, List, ListIter,
    ListList, LongArrayList, LongList, NbtList, ShortList, StringList,
};
pub use string::NbtString;

/// Failures which can occur while parsing an NBT document.
#[derive(Debug)]
#[non_exhaustive]
pub enum ParseError {
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
    UnknownTag { tag: u8, offset: usize },
    /// This happens when we found a TAG_End where we shouldn't
    /// have. TAG_End is only supposed to be found after having a
    /// TAG_Compound, to terminate it. Places we can find this include
    /// as the root tag of a document and inside of a List.
    UnexpectedEndTag,
    /// This library assumes that NBT documents always have a root
    /// TAG_Compound, and if this invariant fails this error will be
    /// generated.
    IncorrectStartTag { tag: TagType },
}

impl fmt::Display for ParseError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::EOF => write!(fmt, "Unexpected end of file"),
            ParseError::UnknownTag { tag, offset } => {
                write!(fmt, "Unknown tag {} at offset {:#x}", tag, offset)
            }
            ParseError::UnexpectedEndTag => write!(fmt, "Unexpected end tag in document"),
            ParseError::IncorrectStartTag { tag } => {
                write!(
                    fmt,
                    "Document starts with tag {:?}, it should only start with Compound.",
                    tag
                )
            }
        }
    }
}

impl std::error::Error for ParseError {}

/// Representation for all values that a tag can be.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Tag<'a> {
    /// A small i8 integer.
    Byte(i8),
    /// An i16 integer.
    Short(i16),
    /// An i32 integer.
    Int(i32),
    /// An i64 integer.
    Long(i64),
    /// An f32 number.
    Float(f32),
    /// An f64 number.
    Double(f64),
    /// An array of raw bytes.
    ByteArray(&'a [u8]),
    /// A string containing CESU-8 encoded text.
    String(NbtString<'a>),
    /// An array of i32.
    IntArray(IntArray<'a>),
    /// An array of i64.
    LongArray(LongArray<'a>),
    /// An array which can only contain a single type. The type can be
    /// any tag, including a nested list. When lists are nested, the
    /// inner lists do not have to be the same type.
    List(List<'a>),
    /// A list of key/value pairs, creating a dictionary.
    Compound(Compound<'a>),
}

impl<'a> Tag<'a> {
    pub(crate) fn read(tag: TagType, reader: &mut Reader<'a>) -> Result<Tag<'a>, ParseError> {
        match tag {
            TagType::End => Err(ParseError::UnexpectedEndTag),
            TagType::Byte => Ok(Tag::Byte(reader.advance(1)?[0] as i8)),
            TagType::Short => Ok(Tag::Short(BigEndian::read_i16(reader.advance(2)?))),
            TagType::Int => Ok(Tag::Int(BigEndian::read_i32(reader.advance(4)?))),
            TagType::Long => Ok(Tag::Long(BigEndian::read_i64(reader.advance(8)?))),
            TagType::Float => Ok(Tag::Float(BigEndian::read_f32(reader.advance(4)?))),
            TagType::Double => Ok(Tag::Double(BigEndian::read_f64(reader.advance(8)?))),
            TagType::String => NbtString::read(reader).map(Tag::String),
            TagType::List => List::read(reader).map(Tag::List),
            TagType::Compound => Compound::read(reader).map(Tag::Compound),
            TagType::ByteArray => read_byte_array(reader).map(Tag::ByteArray),
            TagType::IntArray => IntArray::read(reader).map(Tag::IntArray),
            TagType::LongArray => LongArray::read(reader).map(Tag::LongArray),
        }
    }

    /// Returns the type that represents this tag.
    pub fn tag_type(&self) -> TagType {
        match self {
            Tag::Byte(_) => TagType::Byte,
            Tag::Short(_) => TagType::Short,
            Tag::Int(_) => TagType::Int,
            Tag::Long(_) => TagType::Long,
            Tag::Float(_) => TagType::Float,
            Tag::Double(_) => TagType::Double,
            Tag::ByteArray(_) => TagType::ByteArray,
            Tag::String(_) => TagType::String,
            Tag::List(_) => TagType::List,
            Tag::Compound(_) => TagType::Compound,
            Tag::IntArray(_) => TagType::IntArray,
            Tag::LongArray(_) => TagType::LongArray,
        }
    }

    /// If this tag is a string, returns it. Otherwise, returns None. No coercion is performed.
    pub fn as_string(&self) -> Option<NbtString<'a>> {
        if let Tag::String(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    /// If this tag is a byte array, returns it. Otherwise, returns None.
    pub fn as_byte_array(&self) -> Option<&[u8]> {
        if let Tag::ByteArray(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// If this tag is a [Compound], returns it. Otherwise, returns None.
    pub fn as_compound(&self) -> Option<&Compound<'a>> {
        if let Tag::Compound(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// If this tag is a [List], returns it. Otherwise, returns None.
    pub fn as_list(&self) -> Option<&List<'a>> {
        if let Tag::List(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Attempts to coerce the tag to an integer. Byte, Short, Int, and
    /// Long will return a value, other tags will return None.
    pub fn to_i64(&self) -> Option<i64> {
        match *self {
            Tag::Byte(value) => Some(value as i64),
            Tag::Short(value) => Some(value as i64),
            Tag::Int(value) => Some(value as i64),
            Tag::Long(value) => Some(value),
            _ => None,
        }
    }

    /// Attempts to coerce the tag to a f64. Byte, Short, Int, Long,
    /// Float, and Double will return a value, other tags will return
    /// None.
    pub fn to_f64(&self) -> Option<f64> {
        match *self {
            Tag::Byte(value) => Some(value as f64),
            Tag::Short(value) => Some(value as f64),
            Tag::Int(value) => Some(value as f64),
            Tag::Long(value) => Some(value as f64),
            Tag::Float(value) => Some(value as f64),
            Tag::Double(value) => Some(value),
            _ => None,
        }
    }

    /// Attempts to coerce the tag to a f32. Byte, Short, Int, Long,
    /// Float, and Double will return a value, other tags will return
    /// None.
    pub fn to_f32(&self) -> Option<f32> {
        match *self {
            Tag::Byte(value) => Some(value as f32),
            Tag::Short(value) => Some(value as f32),
            Tag::Int(value) => Some(value as f32),
            Tag::Long(value) => Some(value as f32),
            Tag::Float(value) => Some(value),
            Tag::Double(value) => Some(value as f32),
            _ => None,
        }
    }

    /// If the tag is in the 1.16+ UUID format (IntArray of length 4),
    /// returns it as big endian bytes. Otherwise, returns None.
    pub fn to_uuid_bytes(&self) -> Option<[u8; 16]> {
        if let Tag::IntArray(array) = self {
            if array.len() == 4 {
                let mut buf = [0; 16];
                BigEndian::write_i32(&mut buf[0..4], array.get(0).unwrap());
                BigEndian::write_i32(&mut buf[4..8], array.get(1).unwrap());
                BigEndian::write_i32(&mut buf[8..12], array.get(2).unwrap());
                BigEndian::write_i32(&mut buf[12..16], array.get(3).unwrap());
                return Some(buf);
            }
        }
        None
    }

    /// Similar to [Tag::to_uuid_bytes], but returns a [uuid::Uuid]. Requires the `uuid` feature.
    #[cfg(feature = "uuid")]
    pub fn to_uuid(&self) -> Option<uuid::Uuid> {
        self.to_uuid_bytes().map(uuid::Uuid::from_bytes)
    }
}

pub(crate) fn read_type(reader: &mut Reader<'_>) -> Result<TagType, ParseError> {
    let offset = reader.position;
    match reader.advance(1)?[0] {
        0 => Ok(TagType::End),
        1 => Ok(TagType::Byte),
        2 => Ok(TagType::Short),
        3 => Ok(TagType::Int),
        4 => Ok(TagType::Long),
        5 => Ok(TagType::Float),
        6 => Ok(TagType::Double),
        7 => Ok(TagType::ByteArray),
        8 => Ok(TagType::String),
        9 => Ok(TagType::List),
        10 => Ok(TagType::Compound),
        11 => Ok(TagType::IntArray),
        12 => Ok(TagType::LongArray),
        tag => Err(ParseError::UnknownTag { tag, offset }),
    }
}

fn read_byte_array<'a>(reader: &mut Reader<'a>) -> Result<&'a [u8], ParseError> {
    let len = BigEndian::read_u32(reader.advance(4)?);
    Ok(reader.advance(len as usize)?)
}

/// Represents an NBT document and is the owner of the data contained in
/// it. All other decoder types are borrows of the data stored in this.
///
/// # Example
///
/// ```rust
/// # use std::error::Error;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
/// use nobility::bin_decode::Document;
/// # let input = Document::doctest_demo();
///
/// // Either copies the data (plaintext) or decompresses it (gzip). Accepts
/// // any implementation of Read.
/// let doc = Document::load(input)?;
///
/// // Returns the root tag's name, and the root tag (always a Compound tag).
/// // Both of these are borrowing the data inside the Document.
/// let (name, root) = doc.parse()?;
/// # let _ = (name, root);
/// # Ok(())
/// # }
/// ```

#[derive(Clone, PartialEq)]
pub struct Document {
    data: Vec<u8>,
}

impl Document {
    #[doc(hidden)]
    pub fn doctest_demo() -> impl Read + Clone {
        use std::fs::File;

        let mut file = File::open("files/hello_world.nbt").expect("File should exist");
        let mut data = vec![];
        file.read_to_end(&mut data).unwrap();
        std::io::Cursor::new(data)
    }

    /// Loads a document from any source implementing Read. Sources that
    /// are compressed with gzip will be automatically decompressed,
    /// otherwise the data will just be copied.
    ///
    /// # Errors
    ///
    /// Errors from this function are either from the input [Read]
    /// object or from [GzDecoder].
    pub fn load<R: Read + Clone>(mut input: R) -> Result<Document, IoError> {
        let mut decoder = GzDecoder::new(input.clone());
        let mut data = vec![];
        if decoder.header().is_some() {
            // Valid gzip stream
            decoder.read_to_end(&mut data)?;
        } else {
            // Not a gzip stream
            input.read_to_end(&mut data)?;
        }
        Ok(Document { data })
    }

    /// Parses the document and returns the name and contents of the
    /// root tag.
    ///
    /// # Errors
    ///
    /// The only cases that this should return an error are:
    ///
    /// 1. The input is not an NBT document. This will likely generate
    ///    [ParseError::IncorrectStartTag].
    /// 2. The input is an NBT document, but is malformed/corrupted, or
    ///    in the Bedrock edition version of the format.
    /// 2. The document is compressed using something other than
    ///    gzip. This will likely generate
    ///    [ParseError::IncorrectStartTag].
    /// 3. The specification has changed due to a new Minecraft version.
    ///    This will likely generate [ParseError::UnknownTag].
    /// 4. There's a bug in the parser.
    pub fn parse(&self) -> Result<(NbtString, Compound), ParseError> {
        let mut reader = Reader::new(&self.data);
        let tag = read_type(&mut reader)?;
        if tag != TagType::Compound {
            return Err(ParseError::IncorrectStartTag { tag });
        }
        let name = NbtString::read(&mut reader)?;
        let root = Compound::read(&mut reader)?;
        Ok((name, root))
    }
}

impl fmt::Debug for Document {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Document({} B buffer)", self.data.len() / 1000)
    }
}
