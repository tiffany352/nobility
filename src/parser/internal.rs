use crate::parser::ParseError;
use byteorder::{BigEndian, ByteOrder};

pub trait NbtParse<'a>: Sized {
    fn read(reader: &mut Reader<'a>) -> Result<Self, ParseError>;
}

macro_rules! primitive_impl {
    ($ty:ty, $size:expr, $func:ident) => {
        impl<'a> NbtParse<'a> for $ty {
            fn read(reader: &mut Reader<'a>) -> Result<Self, ParseError> {
                Ok(BigEndian::$func(reader.advance($size)?))
            }
        }
    };
}

primitive_impl!(i16, 2, read_i16);
primitive_impl!(i32, 4, read_i32);
primitive_impl!(i64, 8, read_i64);
primitive_impl!(f32, 4, read_f32);
primitive_impl!(f64, 8, read_f64);

impl<'a> NbtParse<'a> for &'a [u8] {
    fn read(reader: &mut Reader<'a>) -> Result<Self, ParseError> {
        let length = BigEndian::read_u32(reader.advance(4)?);
        Ok(reader.advance(length as usize)?)
    }
}

pub struct Reader<'a> {
    buffer: &'a [u8],
    pub position: usize,
}

impl<'a> Reader<'a> {
    pub(crate) fn new(buffer: &'a [u8]) -> Reader<'a> {
        Reader {
            buffer: buffer,
            position: 0,
        }
    }

    pub(crate) fn advance(&mut self, n: usize) -> Result<&'a [u8], ParseError> {
        if self.buffer.len() < self.position + n {
            Err(ParseError::EOF)
        } else {
            let slice = &self.buffer[self.position..self.position + n];
            self.position += n;
            Ok(slice)
        }
    }
}
