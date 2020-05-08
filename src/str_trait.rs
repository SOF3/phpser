use std::str;

/// Represents a string of data, either owned or referenced.
///
/// The data can either be a raw byte string (`[u8]`) or a UTF-8-checked string (`str`),
/// and can be used either in the boxed type (`Vec<u8>`/`String`)
/// or as a reference (`[u8]`/`str`).
///
/// # Safety
/// See the safety sections in each method.
pub unsafe trait Str<'de>: 'de + Sized {
    /// Gets the length of the string.
    fn len(&self) -> usize;

    /// Express the string as a slice of bytes
    fn as_bytes(&self) -> &[u8];

    /// Returns whether the string is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets the character at offset `i`.
    /// Returns `None` if `i+1` is not a boundary.
    ///
    /// # Safety
    /// The offset `i` must be a position that
    /// the implementation previously inferred as a boundary
    /// and less than the result of `len()`.
    ///
    /// The implementation must not return `Some`
    /// unless `i+1` is also a boundary.
    unsafe fn get_u8_char(&self, i: usize) -> Option<u8>;

    /// Clones the characters from offset `i` to offset `j`.
    /// Returns `None` if `j` is not a boundary.
    ///
    /// # Safety
    /// The offset `i` must be a position that
    /// the implementation previously inferred as a boundary
    ///
    /// `i` and `j` must be less than the result of `len()`.
    ///
    /// The implementation must not return `Some`
    /// unless `i+1` is also a boundary.
    unsafe fn clone_slice(&self, i: usize, j: usize) -> Option<Self>;

    /// Finds the offset of the first occurrence of the ASCII character `char`
    /// after (but not including) offset `i`.
    ///
    /// # Safety
    /// The offset `i` should be a boundary,
    /// although this does not affect the implementation
    /// for UTF-8 strings.
    ///
    /// `i` must be less than the result of `len()`.
    ///
    /// `char` must be an ASCII character.
    ///
    /// If the returned value is `Some`,
    /// it must contain a boundary.
    unsafe fn find(&self, i: usize, char: u8) -> Option<usize>;

    /// Takes the subslice in bytes `i..`.
    ///
    /// # Safety
    /// The offset `i` must be a boundary,
    /// and hence must be `<= self.len()`.
    unsafe fn range_from(&self, i: usize) -> Self;

    /// Takes the subslice in bytes `i..j`.
    ///
    /// # Safety
    /// The offsets `i` and `j` must be boundaries,
    /// and hence must be `<= self.len()`.
    unsafe fn range(&self, i: usize, j: usize) -> Self;
}

unsafe impl<'de> Str<'de> for &'de str {
    fn len(&self) -> usize {
        str::len(*self)
    }

    fn as_bytes(&self) -> &[u8] {
        str::as_bytes(*self)
    }

    unsafe fn get_u8_char(&self, i: usize) -> Option<u8> {
        // safety assertions
        debug_assert!(i < self.len());
        debug_assert!(self.is_char_boundary(i));

        if self.is_char_boundary(i + 1) {
            Some(*self.as_bytes().get_unchecked(i))
        } else {
            None
        }
    }

    unsafe fn clone_slice(&self, i: usize, j: usize) -> Option<Self> {
        // safety assertions
        debug_assert!(i < self.len());
        debug_assert!(self.is_char_boundary(i));

        if self.is_char_boundary(j) {
            let bytes = str::as_bytes(*self).get_unchecked(i..j);
            Some(str::from_utf8_unchecked(bytes)) // checked above
        } else {
            None
        }
    }

    unsafe fn find(&self, i: usize, char: u8) -> Option<usize> {
        self.as_bytes().find(i, char)
    }

    unsafe fn range_from(&self, i: usize) -> Self {
        debug_assert!(i <= self.len());
        debug_assert!(self.is_char_boundary(i));

        self.get_unchecked(i..)
    }

    unsafe fn range(&self, i: usize, j: usize) -> Self {
        debug_assert!(i <= j);
        debug_assert!(j <= self.len());
        debug_assert!(self.is_char_boundary(i));
        debug_assert!(self.is_char_boundary(j));

        self.get_unchecked(i..j)
    }
}

unsafe impl<'de> Str<'de> for String {
    fn len(&self) -> usize {
        str::len(self.as_str())
    }

    fn as_bytes(&self) -> &[u8] {
        str::as_bytes(self.as_str())
    }

    unsafe fn get_u8_char(&self, i: usize) -> Option<u8> {
        self.as_str().get_u8_char(i)
    }

    unsafe fn clone_slice(&self, i: usize, j: usize) -> Option<Self> {
        self.as_str().clone_slice(i, j).map(|s| s.to_string())
    }

    unsafe fn find(&self, i: usize, char: u8) -> Option<usize> {
        self.as_bytes().find(i, char)
    }

    unsafe fn range_from(&self, i: usize) -> Self {
        self.as_str().range_from(i).to_string()
    }

    unsafe fn range(&self, i: usize, j: usize) -> Self {
        self.as_str().range(i, j).to_string()
    }
}

unsafe impl<'de> Str<'de> for &'de [u8] {
    fn len(&self) -> usize {
        <[u8]>::len(*self)
    }

    fn as_bytes(&self) -> &[u8] {
        *self
    }

    unsafe fn get_u8_char(&self, i: usize) -> Option<u8> {
        // safety assertions
        debug_assert!(i < self.len());

        Some(*self.get_unchecked(i))
    }

    unsafe fn clone_slice(&self, i: usize, j: usize) -> Option<Self> {
        // safety assertions
        debug_assert!(i < self.len());

        Some(self.get_unchecked(i..j))
    }

    unsafe fn find(&self, i: usize, char: u8) -> Option<usize> {
        // safety assertions
        debug_assert!(i < self.len());

        let slice = self.get_unchecked((i + 1)..);
        // It is safe to add 1 even for UTF-8 safety,
        // provided that `char` is an ASCII character.

        let index = slice.iter().position(|&other| char == other);
        index.map(|index| i + 1 + index)
    }

    unsafe fn range_from(&self, i: usize) -> Self {
        self.get_unchecked(i..)
    }

    unsafe fn range(&self, i: usize, j: usize) -> Self {
        self.get_unchecked(i..j)
    }
}

unsafe impl<'de> Str<'de> for Vec<u8> {
    fn len(&self) -> usize {
        <[u8]>::len(self.as_slice())
    }

    fn as_bytes(&self) -> &[u8] {
        self.as_slice()
    }

    unsafe fn get_u8_char(&self, i: usize) -> Option<u8> {
        self.as_slice().get_u8_char(i)
    }

    unsafe fn clone_slice(&self, i: usize, j: usize) -> Option<Self> {
        self.as_slice().clone_slice(i, j).map(|s| s.to_vec())
    }

    unsafe fn find(&self, i: usize, char: u8) -> Option<usize> {
        self.as_slice().find(i, char)
    }

    unsafe fn range_from(&self, i: usize) -> Self {
        self.get_unchecked(i..).to_vec()
    }

    unsafe fn range(&self, i: usize, j: usize) -> Self {
        self.get_unchecked(i..j).to_vec()
    }
}

