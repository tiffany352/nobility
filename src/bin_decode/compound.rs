use crate::bin_decode::read_type;
use crate::bin_decode::{NbtParse, NbtString, ParseError, Reader, Tag};
use crate::TagType;
use core::ops::Deref;
use core::ops::Index;
use core::slice::Iter as SliceIter;
use std::fmt;

/// Represents an entry into a [Compound], with a name and a value.
#[derive(Clone)]
pub struct Entry<'a> {
    pub name: NbtString<'a>,
    pub value: Tag<'a>,
}

/// Represents TAG_Compound, a list of key/value pairs.
#[derive(Clone)]
pub struct Compound<'a> {
    entries: Vec<Entry<'a>>,
}

impl<'a> NbtParse<'a> for Compound<'a> {
    fn read(reader: &mut Reader<'a>) -> Result<Self, ParseError> {
        let mut entries = vec![];
        loop {
            let tag = read_type(reader)?;
            if tag == TagType::End {
                break;
            }
            let name = NbtString::read(reader)?;
            let value = Tag::read(tag, reader)?;
            entries.push(Entry { name, value });
        }
        Ok(Compound { entries })
    }
}

impl<'a> Compound<'a> {
    /// Returns the number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Converts into a Vec of key/value pairs.
    pub fn into_vec(self) -> Vec<Entry<'a>> {
        self.entries
    }

    /// Searches for the first key that matches the input, and returns
    /// it if it exists.
    pub fn find_first_key(&self, key: &str) -> Option<&Entry<'a>> {
        for entry in &self.entries {
            if entry.name == key {
                return Some(entry);
            }
        }
        None
    }

    /// Returns an iterator over the entries.
    pub fn iter(&self) -> SliceIter<Entry<'a>> {
        self.entries.iter()
    }
}

impl<'a> Deref for Compound<'a> {
    type Target = [Entry<'a>];

    fn deref(&self) -> &Self::Target {
        &self.entries
    }
}

impl<'a> Index<usize> for Compound<'a> {
    type Output = Entry<'a>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl<'a> fmt::Debug for Compound<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut builder = fmt.debug_struct("Compound");
        for entry in &self.entries {
            let name = format!("{:?}", entry.name);
            builder.field(&name, &entry.value);
        }
        builder.finish()
    }
}
