use crate::parser::read_type;
use crate::parser::{NbtParse, NbtString, ParseError, Reader, Tag};
use crate::TagType;
use core::ops::Deref;
use core::ops::Index;
use core::slice::Iter as SliceIter;
use std::fmt;

#[derive(Clone)]
pub struct Entry<'a> {
    pub name: NbtString<'a>,
    pub value: Tag<'a>,
}

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
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn into_vec(self) -> Vec<Entry<'a>> {
        self.entries
    }

    pub fn find_first_key(&self, key: &str) -> Option<&Entry<'a>> {
        for entry in &self.entries {
            if entry.name == key {
                return Some(entry);
            }
        }
        None
    }

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
