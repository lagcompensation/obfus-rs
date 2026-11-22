#![forbid(unsafe_code)]
#![no_std]

use core::fmt;
use core::ops::Deref;

/// SplitMix64 mixing function.
#[inline]
const fn splitmix64(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);

    value ^ (value >> 31)
}

#[doc(hidden)]
pub mod detail {
    use super::splitmix64;

    /// Hashes a string at compile time using FNV-1a 64-bit.
    pub const fn hash(data: &str) -> u64 {
        let bytes = data.as_bytes();

        let mut hash = 0xcbf29ce484222325u64;
        let mut i = 0;

        while i < bytes.len() {
            hash ^= bytes[i] as u64;
            hash = hash.wrapping_mul(0x100000001b3);

            i += 1;
        }

        hash
    }

    /// Generates a compile-time seed.
    pub const fn seed(file: &str, line: u32, column: u32, value: &str) -> u64 {
        let file_hash = hash(file);
        let value_hash = hash(value);

        splitmix64(
            file_hash
                .wrapping_add(value_hash.rotate_left(17))
                .wrapping_add((line as u64).wrapping_mul(0x9e3779b97f4a7c15))
                .wrapping_add((column as u64).wrapping_mul(0xbf58476d1ce4e5b9)),
        )
    }

    /// Encrypts bytes at compile time.
    pub const fn encrypt<const N: usize>(data: &[u8], seed: u64) -> [u8; N] {
        let mut output = [0u8; N];
        let mut i = 0;

        let mut key: u64 = 0;

        while i < N {
            let offset = i & 7;

            if offset == 0 {
                key = splitmix64(seed.wrapping_add((i >> 3) as u64));
            }

            output[i] = data[i] ^ (key >> (offset * 8)) as u8;

            i += 1;
        }

        output
    }
}

/// A heap-allocation-free obfuscated string.
pub struct ObfuscatedString<const N: usize> {
    data: [u8; N],
}

impl<const N: usize> ObfuscatedString<N> {
    /// Decrypts encrypted bytes at runtime.
    #[inline(never)]
    pub fn new(data: &[u8; N], seed: u64) -> Self {
        let mut output = [0u8; N];
        let mut key = 0u64;

        let mut i = 0;

        while i < N {
            let offset = i & 7;

            if offset == 0 {
                key = splitmix64(seed.wrapping_add((i >> 3) as u64));
            }

            output[i] = data[i] ^ (key >> (offset * 8)) as u8;

            i += 1;
        }

        Self {
            data: core::hint::black_box(output),
        }
    }

    /// Returns the string as a UTF-8 slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.data).unwrap()
    }

    /// Returns the underlying bytes.
    #[inline]
    pub const fn as_bytes(&self) -> &[u8; N] {
        &self.data
    }

    /// Returns the string length in bytes.
    #[inline]
    pub const fn len(&self) -> usize {
        N
    }

    /// Returns whether the string is empty.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        N == 0
    }
}

impl<const N: usize> Deref for ObfuscatedString<N> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<const N: usize> AsRef<str> for ObfuscatedString<N> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> fmt::Display for ObfuscatedString<N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<const N: usize> fmt::Debug for ObfuscatedString<N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl<const N: usize> PartialEq for ObfuscatedString<N> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl<const N: usize> PartialEq<str> for ObfuscatedString<N> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl<const N: usize> PartialEq<ObfuscatedString<N>> for str {
    #[inline]
    fn eq(&self, other: &ObfuscatedString<N>) -> bool {
        self == other.as_str()
    }
}

impl<const N: usize> PartialEq<&str> for ObfuscatedString<N> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl<const N: usize> PartialEq<ObfuscatedString<N>> for &str {
    #[inline]
    fn eq(&self, other: &ObfuscatedString<N>) -> bool {
        *self == other.as_str()
    }
}

/// Obfuscates a string literal at compile time.
///
/// The encrypted bytes are decrypted when the expression is evaluated.
///
/// # Example
///
/// ```
/// let message = obfus::obfstr!("Hello, World!");
///
/// assert_eq!(message, "Hello, World!");
/// ```
#[macro_export]
macro_rules! obfstr {
    ($value:literal) => {{
        const SEED: u64 = $crate::detail::seed(file!(), line!(), column!(), $value);

        const ENCRYPTED: [u8; $value.len()] =
            $crate::detail::encrypt::<{ $value.len() }>($value.as_bytes(), SEED);

        $crate::ObfuscatedString::new(&ENCRYPTED, SEED)
    }};
}