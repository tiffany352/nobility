use crate::bin_decode::read_type;
use crate::bin_decode::{NbtParse, NbtString, ParseError, Reader, Tag};
use crate::TagType;
use core::ops::Index;
use core::slice::Iter as SliceIter;
use std::fmt;

/// Represents an entry into a [Compound], with a name and a value.
#[derive(Clone, PartialEq)]
pub struct Entry<'a> {
    name: NbtString<'a>,
    value: Tag<'a>,
}

impl<'a> Entry<'a> {
    pub fn name(&self) -> &NbtString<'a> {
        &self.name
    }

    pub fn value(&self) -> &Tag<'a> {
        &self.value
    }
}

/// Represents TAG_Compound, a list of key/value pairs. The order of the
/// entries is the same that they appear in the file, although this usually
/// is not significant.
///
/// # Example
///
/// ```rust
/// # use std::error::Error;
/// # use nobility::bin_decode::Document;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
/// # let input = Document::doctest_demo();
/// # let doc = Document::load(input)?;
/// # let (_name, compound) = doc.parse()?;
/// #
/// if let Some(entry) = compound.find_first_key("Health") {
///     if let Some(health) = entry.value().to_i64() {
///         println!("Player has {} health", health);
///     }
/// }
/// #
/// # Ok(())
/// # }
/// ```
#[derive(Clone, PartialEq)]
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

    /// Returns true if there are no entries.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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

    pub fn entries(&self) -> &[Entry<'a>] {
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
        let mut builder = fmt.debug_map();
        for entry in &self.entries {
            builder.entry(&entry.name, &entry.value);
        }
        builder.finish()
    }
}
