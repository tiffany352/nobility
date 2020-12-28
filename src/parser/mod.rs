use crate::TagType;
use byteorder::{BigEndian, ByteOrder};
use flate2::read::GzDecoder;
use std::io::Error as IoError;
use std::io::Read;

mod array;
mod compound;
mod internal;
mod list;
mod string;

pub use array::IntArray;
pub use array::LongArray;
pub use compound::Compound;
pub(crate) use internal::{NbtParse, Reader};
pub use list::List;
pub use string::NbtString;

/// Failures which can occur while parsing an NBT document.
#[derive(Debug)]
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
    IncorrectStartTag { tag: u8 },
}

#[derive(Clone, Debug)]
pub enum Tag<'a> {
    Byte(u8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(&'a [u8]),
    String(NbtString<'a>),
    IntArray(IntArray<'a>),
    LongArray(LongArray<'a>),
    List(List<'a>),
    Compound(Compound<'a>),
}

impl<'a> Tag<'a> {
    pub(crate) fn read(tag: TagType, reader: &mut Reader<'a>) -> Result<Tag<'a>, ParseError> {
        match tag {
            TagType::End => Err(ParseError::UnexpectedEndTag),
            TagType::Byte => Ok(Tag::Byte(reader.advance(1)?[0])),
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

    /// If this tag is a string, returns it. Otherwise, returns None. No coercion is performed.
    pub fn as_string(&self) -> Option<NbtString<'a>> {
        if let Tag::String(value) = self {
            Some(*value)
        } else {
            None
        }
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

pub struct Document {
    data: Vec<u8>,
}

impl Document {
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

    pub fn parse<'a>(&'a self) -> Result<(NbtString<'a>, Compound<'a>), ParseError> {
        let mut reader = Reader::new(&self.data);
        let tag = read_type(&mut reader)?;
        if tag != TagType::Compound {
            return Err(ParseError::IncorrectStartTag { tag: tag as u8 });
        }
        let name = NbtString::read(&mut reader)?;
        let root = Compound::read(&mut reader)?;
        Ok((name, root))
    }
}
