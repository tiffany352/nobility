use crate::bin_decode::array::{IntArray, LongArray, NbtArray};
use crate::bin_decode::Tag;
use crate::bin_decode::{
    read_byte_array, read_type, Compound, NbtParse, NbtString, ParseError, Reader,
};
use crate::TagType;
use byteorder::{BigEndian, ByteOrder};
use core::ops::Deref;
use core::ops::Index;
use core::slice::Iter as SliceIter;
use std::fmt;

/// Implementation for lists whose elements do not have a fixed size.
#[derive(Clone)]
pub struct NbtList<T> {
    entries: Vec<T>,
}

impl<'a, T> NbtParse<'a> for NbtList<T>
where
    T: NbtParse<'a>,
{
    fn read(reader: &mut Reader<'a>) -> Result<Self, ParseError> {
        let length = BigEndian::read_u32(reader.advance(4)?);
        let mut entries = vec![];
        entries.reserve(length as usize);
        for _index in 0..length {
            entries.push(T::read(reader)?);
        }
        Ok(NbtList { entries })
    }
}

impl<T> NbtList<T> {
    /// Returns the number of elements in the list.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// If len() > index >= 0, returns Some, otherwise returns None.
    pub fn get(&self, index: usize) -> Option<&T> {
        self.entries.get(index)
    }

    /// Converts into the underlying vector.
    pub fn into_vec(self) -> Vec<T> {
        self.entries
    }

    /// Returns an iterator over the elements of the list.
    pub fn iter(&self) -> SliceIter<T> {
        self.entries.iter()
    }
}

impl<T> Deref for NbtList<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.entries
    }
}

impl<T> Index<usize> for NbtList<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl<T> fmt::Debug for NbtList<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut builder = fmt.debug_list();
        builder.entries(self.iter());
        builder.finish()
    }
}

// Complex lists
/// A TAG_List of TAG_Compound.
pub type CompoundList<'a> = NbtList<Compound<'a>>;
/// A TAG_List of TAG_String.
pub type StringList<'a> = NbtList<NbtString<'a>>;
/// A TAG_List of TAG_List. This is a nested list. The inner lists can
/// each have distinct element types.
pub type ListList<'a> = NbtList<List<'a>>;
/// A TAG_List of TAG_Int_Array.
pub type IntArrayList<'a> = NbtList<IntArray<'a>>;
/// A TAG_List of TAG_Long_Array.
pub type LongArrayList<'a> = NbtList<LongArray<'a>>;
/// A TAG_List of TAG_Byte_Array.
pub type ByteArrayList<'a> = NbtList<&'a [u8]>;

// Primitive lists
/// A TAG_List of TAG_Short.
pub type ShortList<'a> = NbtArray<'a, i16>;
/// A TAG_List of TAG_Int.
pub type IntList<'a> = NbtArray<'a, i32>;
/// A TAG_List of TAG_Long.
pub type LongList<'a> = NbtArray<'a, i64>;
/// A TAG_List of TAG_Float.
pub type FloatList<'a> = NbtArray<'a, f32>;
/// A TAG_List of TAG_Double.
pub type DoubleList<'a> = NbtArray<'a, f64>;

/// An enum that represents all possible list types.
#[derive(Clone, Debug)]
pub enum List<'a> {
    Byte(&'a [u8]),
    Short(ShortList<'a>),
    Int(IntList<'a>),
    Long(LongList<'a>),
    Float(FloatList<'a>),
    Double(DoubleList<'a>),
    ByteArray(ByteArrayList<'a>),
    String(StringList<'a>),
    Compound(CompoundList<'a>),
    List(ListList<'a>),
    IntArray(IntArrayList<'a>),
    LongArray(LongArrayList<'a>),
}

impl<'a> NbtParse<'a> for List<'a> {
    fn read(reader: &mut Reader<'a>) -> Result<Self, ParseError> {
        let tag = read_type(reader)?;
        match tag {
            TagType::End => {
                let length = BigEndian::read_u32(reader.advance(4)?);
                // Some implementations will generate an End tag when
                // serializing an empty list. In this case,
                // implementations should treat it as an empty byte
                // array.
                if length == 0 {
                    Ok(List::Byte(&[]))
                } else {
                    Err(ParseError::UnexpectedEndTag)
                }
            }
            TagType::Byte => read_byte_array(reader).map(List::Byte),
            TagType::Short => Ok(List::Short(ShortList::read(reader)?)),
            TagType::Int => Ok(List::Int(IntList::read(reader)?)),
            TagType::Long => Ok(List::Long(LongList::read(reader)?)),
            TagType::Float => Ok(List::Float(FloatList::read(reader)?)),
            TagType::Double => Ok(List::Double(DoubleList::read(reader)?)),
            TagType::ByteArray => Ok(List::ByteArray(ByteArrayList::read(reader)?)),
            TagType::String => Ok(List::String(StringList::read(reader)?)),
            TagType::List => Ok(List::List(ListList::read(reader)?)),
            TagType::Compound => Ok(List::Compound(CompoundList::read(reader)?)),
            TagType::IntArray => Ok(List::IntArray(IntArrayList::read(reader)?)),
            TagType::LongArray => Ok(List::LongArray(LongArrayList::read(reader)?)),
        }
    }
}

impl<'a> List<'a> {
    /// Returns the number of elements in the list.
    pub fn len(&self) -> usize {
        match self {
            List::Byte(list) => list.len(),
            List::Short(list) => list.len(),
            List::Int(list) => list.len(),
            List::Long(list) => list.len(),
            List::Float(list) => list.len(),
            List::Double(list) => list.len(),
            List::ByteArray(list) => list.len(),
            List::String(list) => list.len(),
            List::Compound(list) => list.len(),
            List::List(list) => list.len(),
            List::IntArray(list) => list.len(),
            List::LongArray(list) => list.len(),
        }
    }

    /// Allows the list to be indexed without creating a case for each
    /// possible list type. The return value is wrapped in a Tag.
    ///
    /// Note that for complex lists, a clone will be performed. This
    /// happens for Compound, Int Array, and Long Array lists.
    pub fn get(&self, index: usize) -> Option<Tag<'a>> {
        match self {
            List::Byte(list) => list.get(index).map(|&v| Tag::Byte(v as i8)),
            List::Short(list) => list.get(index).map(Tag::Short),
            List::Int(list) => list.get(index).map(Tag::Int),
            List::Long(list) => list.get(index).map(Tag::Long),
            List::Float(list) => list.get(index).map(Tag::Float),
            List::Double(list) => list.get(index).map(Tag::Double),
            List::ByteArray(list) => list.get(index).map(|v| Tag::ByteArray(v)),
            List::String(list) => list.get(index).map(|v| Tag::String(*v)),
            List::Compound(list) => list.get(index).map(|v| Tag::Compound(v.clone())),
            List::List(list) => list.get(index).map(|v| Tag::List(v.clone())),
            List::IntArray(list) => list.get(index).map(|v| Tag::IntArray(v.clone())),
            List::LongArray(list) => list.get(index).map(|v| Tag::LongArray(v.clone())),
        }
    }

    /// Returns an iterator over the elements of the list, yielding a Tag.
    pub fn iter(&self) -> ListIter<'_> {
        ListIter {
            list: self,
            index: 0,
        }
    }
}

/// Iterator over the contents of [List], wrapped as a [Tag].
pub struct ListIter<'a> {
    list: &'a List<'a>,
    index: usize,
}

impl<'a> Iterator for ListIter<'a> {
    type Item = Tag<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.list.get(self.index);
        self.index += 1;
        result
    }
}
