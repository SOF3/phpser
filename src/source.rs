use std::convert::TryInto;
use std::io::{self, BufRead, Read};
use std::str;

use crate::*;

/// Represents a data source for a `Str`.
///
/// This is analogous to `Read`,
/// except with arbitrary, potentially non-static return type `S: Str`,
/// and provides methods optimized for usage in parsing.
pub trait Source<'de, S: Str<'de>> {
    /// Returns the number of bytes already read from the source.
    fn offset(&self) -> usize;

    /// Returns the maximum possible number of bytes in the source.
    ///
    /// This value is used for security reasons,
    /// to avoid out-of-memory error from arbitrarily large numbers
    /// as requested by the serialization.
    fn limit(&self) -> usize;

    /// Reads one byte from the source.
    ///
    /// # Errors
    /// If the byte is not an ASCII character,
    /// this method returns `Error::BadEncoding`.
    ///
    /// If the source has already ended,
    /// this method MUST return `Error::UnexpectedEof`
    /// (instead of `io::ErrorKind::UnexpectedEof`).
    ///
    /// If an IO error occurs,
    /// the error is returned directly wrapped in `IoError::Io`.
    fn read_u8_char(&mut self) -> IoResult<u8>;

    /// Reads `n` *bytes* from the source.
    ///
    /// # Errors
    /// If the nth byte does ont terminate a character boundary,
    /// this method should return `Error::BadEncoding`.
    ///
    /// If the source has less remaining characters available than `n`,
    /// this method MUST return `Error::UnexpectedEof`
    /// (instead of `io::ErrorKind::UnexpectedEof`).
    ///
    /// If an IO error occurs,
    /// the error is returned directly wrapped in `IoError::Io`.
    fn read_str(&mut self, n: usize) -> IoResult<S>;

    /// Reads the source until the byte `byte`.
    ///
    /// This method consumes the slice before `byte` AND `byte` itself,
    /// but only returns the slice before `byte`.
    ///
    /// # Safety
    /// `byte` must be a valid ASCII character.
    unsafe fn read_until(&mut self, byte: u8) -> IoResult<S>;
}

impl<'t, 'de, S, T> Source<'de, S> for &'t mut T
where
    S: Str<'de>,
    T: Source<'de, S>,
{
    fn offset(&self) -> usize {
        <T as Source<'de, S>>::offset(&**self)
    }

    fn limit(&self) -> usize {
        <T as Source<'de, S>>::limit(&**self)
    }

    fn read_u8_char(&mut self) -> IoResult<u8> {
        <T as Source<'de, S>>::read_u8_char(&mut **self)
    }

    fn read_str(&mut self, n: usize) -> IoResult<S> {
        <T as Source<'de, S>>::read_str(&mut **self, n)
    }

    unsafe fn read_until(&mut self, byte: u8) -> IoResult<S> {
        <T as Source<'de, S>>::read_until(&mut **self, byte)
    }
}

/// Reads an `io::Read` into a `Value<Vec<u8>>`.
pub struct ByteReader<R: Read> {
    read: io::BufReader<io::Take<R>>,
    offset: usize,
    limit: usize,
}

impl<R: Read> ByteReader<R> {
    /// Creates a new `ByteReader`.
    ///
    /// The `read` does not need to be buffered;
    /// the implementation would automatically buffer it.
    ///
    /// The `limit` value is used to avoid allocating arbitrary large chunks of memory
    /// as requested by the serialization.
    pub fn new(read: R, limit: usize) -> Self {
        Self {
            read: io::BufReader::new(
                read.take(
                    limit
                        .try_into()
                        .expect("Limit greater than u64::MAX_VALUE is not supported"),
                ),
            ),
            offset: 0,
            limit,
        }
    }
}

impl<'de, R: Read> Source<'de, Vec<u8>> for ByteReader<R> {
    fn offset(&self) -> usize {
        self.offset
    }

    fn limit(&self) -> usize {
        self.limit
    }

    fn read_u8_char(&mut self) -> IoResult<u8> {
        let mut buf = [0u8];
        self.read.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn read_str(&mut self, n: usize) -> IoResult<Vec<u8>> {
        if n > self.limit {
            return Err(Error::UnexpectedEof.into());
        }

        let mut buf = vec![0u8; n];
        self.read.read_exact(&mut buf)?;
        Ok(buf)
    }

    unsafe fn read_until(&mut self, byte: u8) -> IoResult<Vec<u8>> {
        let mut vec = vec![];
        let _ = self.read.read_until(byte, &mut vec)?;
        Ok(vec)
    }
}

/// Reads an `io::Read` into a `Value<String>`.
pub struct StringReader<R: Read> {
    read: io::BufReader<io::Take<R>>,
    offset: usize,
    limit: usize,
}
impl<R: Read> StringReader<R> {
    /// Creates a new `StringReader`.
    ///
    /// The `read` does not need to be buffered;
    /// the implementation would automatically buffer it.
    ///
    /// The `limit` value is used to avoid allocating arbitrary large chunks of memory
    /// as requested by the serialization.
    pub fn new(read: R, limit: usize) -> Self {
        Self {
            read: io::BufReader::new(
                read.take(
                    limit
                        .try_into()
                        .expect("Limit greater than u64::MAX_VALUE is not supported"),
                ),
            ),
            offset: 0,
            limit,
        }
    }
}
impl<'de, R: Read> Source<'de, String> for StringReader<R> {
    fn offset(&self) -> usize {
        self.offset
    }

    fn limit(&self) -> usize {
        self.limit
    }

    fn read_u8_char(&mut self) -> IoResult<u8> {
        let mut buf = [0u8];
        self.read.read_exact(&mut buf)?;
        let _ = str::from_utf8(&buf).map_err(|_| Error::BadEncoding(self.offset))?;
        Ok(buf[0])
    }

    fn read_str(&mut self, n: usize) -> IoResult<String> {
        if n > self.limit {
            return Err(Error::UnexpectedEof.into());
        }

        let mut buf = vec![0u8; n];
        self.read.read_exact(&mut buf)?;
        let string = String::from_utf8(buf).map_err(|_| Error::BadEncoding(self.offset))?;
        Ok(string)
    }

    unsafe fn read_until(&mut self, byte: u8) -> IoResult<String> {
        let mut vec = vec![];
        let _ = self.read.read_until(byte, &mut vec)?;
        let string = String::from_utf8(vec).map_err(|_| Error::BadEncoding(self.offset))?;
        Ok(string)
    }
}
