#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_op_in_unsafe_fn)]

use core::{fmt, hash, ops, str};

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::{borrow::Cow, boxed::Box, string::String};

#[derive(Copy, Eq, PartialEq, Clone, Debug)]
pub struct InvalidLength {
    expected: usize,
    actual: usize,
}

impl fmt::Display for InvalidLength {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Expected string with {} bytes, but got {}",
            self.expected, self.actual
        )
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InvalidLength {}

#[derive(Copy, Clone)]
pub struct Str<const N: usize> {
    v: [u8; N],
}

impl<const N: usize> Str<N> {
    /// Extracts a string slice containing the entire `Str`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use str_array::Str;
    /// let s: Str<3> = Str::try_new("foo").unwrap();
    ///
    /// assert_eq!("foo", s.as_str());
    /// ```
    #[inline]
    #[must_use]
    pub const fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.v) }
    }

    // TODO: Make const when `&mut self` in const fn stabilizes.
    /// Converts a `Str` into a mutable string slice.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use str_array::Str;
    /// let mut s: Str<6> = Str::try_new("foobar").unwrap();
    /// let s_mut_str = s.as_mut_str();
    ///
    /// s_mut_str.make_ascii_uppercase();
    ///
    /// assert_eq!("FOOBAR", s_mut_str);
    /// ```
    #[inline]
    #[must_use]
    pub fn as_mut_str(&mut self) -> &mut str {
        unsafe { str::from_utf8_unchecked_mut(&mut self.v) }
    }
    /// Converts a `Str` into a byte array.
    ///
    /// This consumes the `Str`, so we do not need to copy its contents.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use str_array::Str;
    /// let s = Str::try_from(b"hello").unwrap();
    /// let bytes = s.into_bytes();
    ///
    /// assert_eq!([104, 101, 108, 108, 111], bytes);
    /// ```
    #[inline]
    #[must_use = "`self` will be dropped if the result is not used"]
    pub const fn into_bytes(self) -> [u8; N] {
        self.v
    }

    /// Returns a byte slice of this `Str`'s contents.
    ///
    /// The inverse of this method is [`from_utf8`].
    ///
    /// [`from_utf8`]: Str::from_utf8
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use str_array::Str;
    /// let s: Str<5> = Str::try_from("hello").unwrap();
    ///
    /// assert_eq!(&[104, 101, 108, 108, 111], s.as_bytes());
    /// ```
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        &self.v
    }

    /// Returns the length of this `Str`, in bytes, not [`char`]s or
    /// graphemes. In other words, it might not be what a human considers the
    /// length of the string.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use str_array::Str;
    /// let a: Str<3> = Str::try_from("foo").unwrap();
    /// assert_eq!(a.len(), 3);
    ///
    /// let fancy_f: Str<4> = Str::try_from("ƒoo").unwrap();
    /// assert_eq!(fancy_f.len(), 4);
    /// assert_eq!(fancy_f.chars().count(), 3);
    /// ```
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        N
    }

    /// Returns `true` if this `Str` has a length of zero, and `false` otherwise.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use str_array::Str;
    /// let v = Str::try_from([]).unwrap();
    /// assert!(v.is_empty());
    ///
    /// let v = Str::try_from(b"Hello World").unwrap();
    /// assert!(!v.is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Converts an array of bytes to a `Str`.
    ///
    /// A Str ([`Str`]) is made of an array of bytes ([`u8; N`]),
    /// so this function converts between the two.
    /// Not all byte arrays are valid `Str`s, however: `Str`
    /// requires that it is valid UTF-8. `from_utf8()` checks to ensure that
    /// the bytes are valid UTF-8, and then does the conversion.
    ///
    /// If you are sure that the byte slice is valid UTF-8, and you don't want
    /// to incur the overhead of the validity check, there is an unsafe version
    /// of this function, [`from_utf8_unchecked`], which has the same behavior
    /// but skips the check.
    ///
    /// The inverse of this method is [`into_bytes`].
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the slice is not UTF-8 with a description as to why the
    /// provided bytes are not UTF-8. The vector you moved in is also included.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use str_array::Str;
    /// // some bytes, in an array
    /// let sparkle_heart = [240, 159, 146, 150];
    ///
    /// // We know these bytes are valid, so we'll use `unwrap()`.
    /// let sparkle_heart = Str::from_utf8(sparkle_heart).unwrap();
    ///
    /// assert_eq!("💖", sparkle_heart);
    /// ```
    ///
    /// Incorrect bytes:
    ///
    /// ```
    /// # use str_array::Str;
    /// // some invalid bytes, in a vector
    /// let sparkle_heart = [0, 159, 146, 150];
    ///
    /// assert!(Str::from_utf8(sparkle_heart).is_err());
    /// ```
    ///
    /// See the docs for [`FromUtf8Error`] for more details on what you can do
    /// with this error.
    ///
    /// [`from_utf8_unchecked`]: alloc::string::String::from_utf8_unchecked
    /// [`array`]: prim@array "array"
    /// [`&str`]: prim@str "&str"
    /// [`into_bytes`]: Str::into_bytes
    #[inline]
    pub const fn from_utf8(v: [u8; N]) -> Result<Self, str::Utf8Error> {
        if let Err(e) = run_utf8_validation(&v) {
            return Err(e);
        }
        Ok(unsafe { Self::from_utf8_unchecked_internal(v) })
    }

    #[inline(always)]
    const unsafe fn from_utf8_unchecked_internal(v: [u8; N]) -> Self {
        Self { v }
    }

    /// Converts a slice of bytes to a Str<N> without checking
    /// that the string contains valid UTF-8.
    ///
    /// See the safe version, [`from_utf8`], for more information.
    ///
    /// # Safety
    ///
    /// The bytes passed in must be valid UTF-8.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use str_array::Str;
    ///
    /// // some bytes, in an array
    /// let sparkle_heart = [240, 159, 146, 150];
    ///
    /// let sparkle_heart = unsafe {
    ///     Str::from_utf8_unchecked(sparkle_heart)
    /// };
    ///
    /// assert_eq!("💖", sparkle_heart);
    /// ```
    #[must_use]
    #[inline]
    pub const unsafe fn from_utf8_unchecked(v: [u8; N]) -> Self {
        if cfg!(debug_assertions) {
            match Self::from_utf8(v) {
                Ok(v) => v,
                // TODO: replace with full error panic when stabilized in const fn.
                Err(_e) => panic!("Invalid UTF-8"),
            }
        } else {
            // SAFETY: the caller must guarantee that the bytes `v` are valid UTF-8.
            unsafe { Self::from_utf8_unchecked_internal(v) }
        }
    }

    #[inline]
    pub const fn try_new(str: &str) -> Result<Self, InvalidLength> {
        let bytes = str.as_bytes();
        if bytes.len() != N {
            return Err(InvalidLength {
                expected: N,
                actual: bytes.len(),
            });
        }
        let mut array = [0u8; N];
        // We use `while` because `copy_from_slice` is not const fn yet.
        let mut i = 0;
        while i < N {
            array[i] = bytes[i];
            i += 1;
        }
        // Safety: str is guaranteed to be valid UTF-8.
        Ok(unsafe { Self::from_utf8_unchecked(array) })
    }
}

#[inline(always)]
const fn run_utf8_validation(v: &[u8]) -> Result<(), str::Utf8Error> {
    match str::from_utf8(v) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

impl<const N: usize> AsRef<str> for Str<N> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> AsMut<str> for Str<N> {
    #[inline]
    fn as_mut(&mut self) -> &mut str {
        self.as_mut_str()
    }
}

impl<const N: usize> AsRef<[u8]> for Str<N> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.v
    }
}

impl<const N: usize> TryFrom<&str> for Str<N> {
    type Error = InvalidLength;
    /// Try to convert a `&str` into a [`Str<N>`].
    ///
    /// The result will fail if the string's length is not exactly `N`.
    #[inline]
    fn try_from(v: &str) -> Result<Self, Self::Error> {
        Self::try_new(v)
    }
}

impl<const N: usize> TryFrom<&mut str> for Str<N> {
    type Error = InvalidLength;
    /// Try to convert a `&mut str` into a [`Str<N>`].
    ///
    /// The result will fail if the string's length is not exactly `N`.
    #[inline]
    fn try_from(v: &mut str) -> Result<Self, Self::Error> {
        Self::try_new(v)
    }
}

#[cfg(feature = "alloc")]
impl<const N: usize> TryFrom<String> for Str<N> {
    type Error = InvalidLength;
    /// Try to convert a `String` into a [`Str<N>`].
    ///
    /// The result will fail if the string's length is not exactly `N`.
    #[inline]
    fn try_from(v: String) -> Result<Self, Self::Error> {
        Self::try_new(&v)
    }
}

#[cfg(feature = "alloc")]
impl<const N: usize> TryFrom<&String> for Str<N> {
    type Error = InvalidLength;
    /// Try to convert a `&String` into a [`Str<N>`].
    ///
    /// The result will fail if the string's length is not exactly `N`.
    #[inline]
    fn try_from(v: &String) -> Result<Self, Self::Error> {
        Self::try_new(v)
    }
}
#[cfg(feature = "alloc")]
impl<const N: usize> TryFrom<&mut String> for Str<N> {
    type Error = InvalidLength;
    /// Try to convert a `&mut String` into a [`Str<N>`].
    ///
    /// The result will fail if the string's length is not exactly `N`.
    #[inline]
    fn try_from(v: &mut String) -> Result<Self, Self::Error> {
        Self::try_new(v)
    }
}

#[cfg(feature = "alloc")]
impl<const N: usize> TryFrom<Box<str>> for Str<N> {
    type Error = InvalidLength;
    /// Try to convert a `Box<str>` into a [`Str<N>`].
    ///
    /// The result will fail if the string's length is not exactly `N`.
    #[inline]
    fn try_from(v: Box<str>) -> Result<Self, Self::Error> {
        Self::try_new(&v)
    }
}

#[cfg(feature = "alloc")]
impl<const N: usize> TryFrom<Cow<'_, str>> for Str<N> {
    type Error = InvalidLength;
    /// Try to Convert a clone-on-write string `Cow<'_, str>` into a [`Str<N>`].
    ///
    /// The result will fail if the string's length is not exactly `N`.
    #[inline]
    fn try_from(v: Cow<'_, str>) -> Result<Self, Self::Error> {
        Self::try_new(&v)
    }
}

impl<const N: usize> TryFrom<[u8; N]> for Str<N> {
    type Error = str::Utf8Error;
    #[inline]
    fn try_from(v: [u8; N]) -> Result<Self, Self::Error> {
        Self::from_utf8(v)
    }
}

impl<const N: usize> TryFrom<&[u8; N]> for Str<N> {
    type Error = str::Utf8Error;
    #[inline]
    fn try_from(v: &[u8; N]) -> Result<Self, Self::Error> {
        Self::from_utf8(*v)
    }
}

#[cfg(feature = "alloc")]
impl<const N: usize> From<Str<N>> for String {
    #[inline]
    fn from(v: Str<N>) -> Self {
        String::from(v.as_str())
    }
}

#[cfg(feature = "alloc")]
impl<const N: usize> From<Str<N>> for Box<str> {
    #[inline]
    fn from(v: Str<N>) -> Self {
        unsafe { alloc::str::from_boxed_utf8_unchecked(Box::new(v.v)) }
    }
}

// str::Owned == String, can we still do this somehow?
// #[cfg(feature = "alloc")]
// impl<const N: usize> From<Str<N>> for Cow<'_, str> {
//     fn from(v: Str<N>) -> Self {
//         Cow::Owned(v)
//     }
// }

#[cfg(feature = "alloc")]
impl<'a, const N: usize> From<&'a Str<N>> for Cow<'a, str> {
    #[inline]
    fn from(v: &'a Str<N>) -> Self {
        Cow::Borrowed(v.as_str())
    }
}

// impl<const N: usize> TryFrom<&[u8]> for Str<N> {
//     type Error = str::Utf8Error;
//     fn try_from(v: [u8; N]) -> Result<Self, Self::Error> {
//         Self::from_utf8(v)
//     }
// }

macro_rules! impl_eq {
    ($other: ty) => {
        #[allow(unused_lifetimes)]
        #[allow(clippy::extra_unused_lifetimes)]
        impl<'a, const N: usize> PartialEq<$other> for Str<N> {
            #[inline]
            fn eq(&self, other: &$other) -> bool {
                <str as PartialEq>::eq(self.as_ref(), other.as_ref())
            }
        }

        #[allow(unused_lifetimes)]
        #[allow(clippy::extra_unused_lifetimes)]
        impl<'a, const N: usize> PartialEq<Str<N>> for $other {
            #[inline]
            fn eq(&self, other: &Str<N>) -> bool {
                <str as PartialEq>::eq(self.as_ref(), other.as_ref())
            }
        }
    };
}

impl_eq! { str }
impl_eq! { &'a str }
#[cfg(feature = "alloc")]
impl_eq! { Cow<'a, str> }
#[cfg(feature = "alloc")]
impl_eq! { String }
#[cfg(feature = "alloc")]
impl_eq! { &'a String }
#[cfg(feature = "alloc")]
impl_eq! { Box<str> }

impl<const N: usize> fmt::Display for Str<N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

// Requires specialization
// #[cfg(feature = "alloc")]
// impl<const N: usize> ToString for Str<N> {
//     #[inline]
//     fn to_string(&self) -> String {
//         String::from(self)
//     }
// }

impl<const N: usize> fmt::Debug for Str<N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl<const N: usize> hash::Hash for Str<N> {
    #[inline]
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        (self.as_str()).hash(hasher)
    }
}

impl<const N: usize> ops::Index<ops::Range<usize>> for Str<N> {
    type Output = str;

    #[inline]
    fn index(&self, index: ops::Range<usize>) -> &str {
        ops::Index::index(self.as_str(), index)
    }
}
impl<const N: usize> ops::Index<ops::RangeTo<usize>> for Str<N> {
    type Output = str;

    #[inline]
    fn index(&self, index: ops::RangeTo<usize>) -> &str {
        ops::Index::index(self.as_str(), index)
    }
}
impl<const N: usize> ops::Index<ops::RangeFrom<usize>> for Str<N> {
    type Output = str;

    #[inline]
    fn index(&self, index: ops::RangeFrom<usize>) -> &str {
        ops::Index::index(self.as_str(), index)
    }
}
impl<const N: usize> ops::Index<ops::RangeFull> for Str<N> {
    type Output = str;

    #[inline]
    fn index(&self, index: ops::RangeFull) -> &str {
        ops::Index::index(self.as_str(), index)
    }
}
impl<const N: usize> ops::Index<ops::RangeInclusive<usize>> for Str<N> {
    type Output = str;

    #[inline]
    fn index(&self, index: ops::RangeInclusive<usize>) -> &str {
        ops::Index::index(self.as_str(), index)
    }
}
impl<const N: usize> ops::Index<ops::RangeToInclusive<usize>> for Str<N> {
    type Output = str;

    #[inline]
    fn index(&self, index: ops::RangeToInclusive<usize>) -> &str {
        ops::Index::index(self.as_str(), index)
    }
}

impl<const N: usize> ops::IndexMut<ops::Range<usize>> for Str<N> {
    #[inline]
    fn index_mut(&mut self, index: ops::Range<usize>) -> &mut str {
        ops::IndexMut::index_mut(self.as_mut_str(), index)
    }
}
impl<const N: usize> ops::IndexMut<ops::RangeTo<usize>> for Str<N> {
    #[inline]
    fn index_mut(&mut self, index: ops::RangeTo<usize>) -> &mut str {
        ops::IndexMut::index_mut(self.as_mut_str(), index)
    }
}
impl<const N: usize> ops::IndexMut<ops::RangeFrom<usize>> for Str<N> {
    #[inline]
    fn index_mut(&mut self, index: ops::RangeFrom<usize>) -> &mut str {
        ops::IndexMut::index_mut(self.as_mut_str(), index)
    }
}
impl<const N: usize> ops::IndexMut<ops::RangeFull> for Str<N> {
    #[inline]
    fn index_mut(&mut self, index: ops::RangeFull) -> &mut str {
        ops::IndexMut::index_mut(self.as_mut_str(), index)
    }
}
impl<const N: usize> ops::IndexMut<ops::RangeInclusive<usize>> for Str<N> {
    #[inline]
    fn index_mut(&mut self, index: ops::RangeInclusive<usize>) -> &mut str {
        ops::IndexMut::index_mut(self.as_mut_str(), index)
    }
}
impl<const N: usize> ops::IndexMut<ops::RangeToInclusive<usize>> for Str<N> {
    #[inline]
    fn index_mut(&mut self, index: ops::RangeToInclusive<usize>) -> &mut str {
        ops::IndexMut::index_mut(self.as_mut_str(), index)
    }
}

impl<const N: usize> ops::Deref for Str<N> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> ops::DerefMut for Str<N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut str {
        self.as_mut_str()
    }
}
