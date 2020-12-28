use crate::bin_decode::{NbtParse, ParseError, Reader};
use byteorder::{BigEndian, ByteOrder};
use core::marker::PhantomData;
use std::fmt;

/// Common representation for TAG_Int_Array, TAG_Long_Array, and
/// TAG_List with elements of fixed size (Byte, Short, Int, Long, Float,
/// Double).
#[derive(Clone, Copy)]
pub struct NbtArray<'a, T> {
    data: &'a [u8],
    _phantom: PhantomData<T>,
}

mod internal {
    use byteorder::{BigEndian, ByteOrder};
    use std::fmt::Debug;

    pub trait NbtPrimitive: Debug + Copy {
        const SIZE: usize;

        fn read(data: &[u8]) -> Self;
    }

    macro_rules! create_impl {
        ($ty:ty, $size:expr, $func:ident) => {
            impl NbtPrimitive for $ty {
                const SIZE: usize = $size;
                fn read(data: &[u8]) -> Self {
                    BigEndian::$func(data)
                }
            }
        };
    }

    create_impl!(i16, 2, read_i16);
    create_impl!(i32, 4, read_i32);
    create_impl!(i64, 8, read_i64);
    create_impl!(f32, 4, read_f32);
    create_impl!(f64, 8, read_f64);
}

use internal::NbtPrimitive;

impl<'a, T> NbtParse<'a> for NbtArray<'a, T>
where
    T: NbtPrimitive,
{
    fn read(reader: &mut Reader<'a>) -> Result<Self, ParseError> {
        let length = BigEndian::read_u32(reader.advance(4)?);
        let data = reader.advance(length as usize * T::SIZE)?;
        Ok(NbtArray {
            data,
            _phantom: PhantomData,
        })
    }
}

impl<'a, T> NbtArray<'a, T>
where
    T: NbtPrimitive,
{
    /// Returns the number of elements in the array.
    pub fn len(&self) -> usize {
        self.data.len() / T::SIZE
    }

    /// Returns an element at the index if it's in the range 0..len(),
    /// or None.
    pub fn get(&self, index: usize) -> Option<T> {
        if index < self.len() {
            let start = index * T::SIZE;
            Some(T::read(&self.data[start..start + T::SIZE]))
        } else {
            None
        }
    }

    /// Creates a Vec of the contents of this array.
    pub fn to_vec(&self) -> Vec<T> {
        let mut v = vec![];
        v.reserve(self.len());
        for i in 0..self.len() {
            v.push(self.get(i).unwrap())
        }
        v
    }

    /// Returns an iterator over the elements of the array.
    pub fn iter(&self) -> NbtArrayIter<'a, T> {
        NbtArrayIter {
            array: *self,
            index: 0,
        }
    }
}

impl<'a, T> fmt::Debug for NbtArray<'a, T>
where
    T: NbtPrimitive,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut builder = fmt.debug_list();
        builder.entries(self.iter());
        builder.finish()
    }
}

/// Iterator over the contents of [NbtArray], yielding the element type.
pub struct NbtArrayIter<'a, T> {
    array: NbtArray<'a, T>,
    index: usize,
}

impl<'a, T> Iterator for NbtArrayIter<'a, T>
where
    T: NbtPrimitive,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.array.get(self.index);
        self.index += 1;
        result
    }
}

/// TAG_Int_Array, represented using [NbtArray].
pub type IntArray<'a> = NbtArray<'a, i32>;
/// TAG_Long_Array, represented using [NbtArray].
pub type LongArray<'a> = NbtArray<'a, i64>;
