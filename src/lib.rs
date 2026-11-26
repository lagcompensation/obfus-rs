#![forbid(unsafe_code)]
#![no_std]

use core::fmt;
use core::marker::PhantomData;
use core::ops::Deref;

use zeroize::Zeroize;

/// SplitMix64 mixing function.
#[inline]
#[must_use]
const fn splitmix64(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);

    value ^ (value >> 31)
}

#[doc(hidden)]
pub mod detail {
    use super::splitmix64;
    use core::marker::PhantomData;

    /// Hashes a string at compile time using FNV-1a 64-bit.
    #[must_use]
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
    #[must_use]
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

    /// Encrypts a byte slice at compile time (used by `obfstr!`).
    #[must_use]
    pub const fn encrypt<const N: usize>(data: &[u8], seed: u64) -> [u8; N] {
        let mut output = [0u8; N];
        let mut i = 0;

        let mut key = 0u64;

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

    /// Encrypts an owned byte array at compile time (used by
    /// `obfval!`). XOR is its own inverse, so the same function
    /// both encrypts (at compile time) and decrypts (at run time).
    #[must_use]
    pub const fn encrypt_array<const N: usize>(mut data: [u8; N], seed: u64) -> [u8; N] {
        let mut i = 0;
        let mut key = 0u64;

        while i < N {
            let offset = i & 7;

            if offset == 0 {
                key = splitmix64(seed.wrapping_add((i >> 3) as u64));
            }

            data[i] ^= (key >> (offset * 8)) as u8;

            i += 1;
        }

        data
    }

    /// Consumes a Copy value and returns a type-level marker for it.
    ///
    /// Used by `obfval!` to preserve the concrete type of obfuscated
    /// value without storing the original value in `PhantomData`.
    #[must_use]
    pub const fn phantom_of<T: Copy>(value: T) -> PhantomData<T> {
        let _ = value;
        PhantomData
    }
}

mod sealed {
    pub trait Sealed {}
}

/// This trait is sealed and can only be implemented for built-in
/// integer types supported by this crate.
///
/// Sealing preserves the invariant that `N` is exactly equal to
/// `size_of::<Self>()` and that every possible byte pattern is valid.
pub trait Obfuscatable<const N: usize>: Copy + sealed::Sealed {
    /// Reconstructs the value from its native-endian byte representation.
    fn from_ne_bytes(bytes: [u8; N]) -> Self;
}

macro_rules! impl_obfuscatable {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl sealed::Sealed for $ty {}

            impl Obfuscatable<{ core::mem::size_of::<$ty>() }> for $ty {
                #[inline]
                fn from_ne_bytes(bytes: [u8; core::mem::size_of::<$ty>()]) -> Self {
                    <$ty>::from_ne_bytes(bytes)
                }
            }
        )+
    };
}

impl_obfuscatable!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize,
);

/// A heap-allocation-free obfuscated string.
///
/// Internally, all string-producing operations rely on the decrypted
/// bytes being valid UTF-8.
///
/// Note: intentionally does not implement `Clone` — cloning would
/// create a second copy of the decrypted plaintext in memory that
/// this crate cannot track or zeroize on your behalf.
#[must_use = "obfuscated string is immediately dropped if unused"]
pub struct ObfuscatedString<const N: usize> {
    data: [u8; N],
}

impl<const N: usize> Drop for ObfuscatedString<N> {
    #[inline]
    fn drop(&mut self) {
        self.data.zeroize();
    }
}

impl<const N: usize> ObfuscatedString<N> {
    /// Decrypts encrypted bytes at runtime.
    ///
    /// Primarily intended to be called by the `obfstr!` macro.
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
    ///
    /// # Panics
    ///
    /// Panics if the decrypted bytes are not valid UTF-8.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.data).unwrap()
    }

    /// Returns the underlying bytes.
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; N] {
        &self.data
    }

    /// Returns the string length in bytes.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        N
    }

    /// Returns whether the string is empty.
    #[inline]
    #[must_use]
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ObfuscatedString").field("len", &N).finish()
    }
}

impl<const N: usize, const M: usize> PartialEq<ObfuscatedString<M>> for ObfuscatedString<N> {
    #[inline]
    fn eq(&self, other: &ObfuscatedString<M>) -> bool {
        if N != M {
            return false;
        }

        self.as_str() == other.as_str()
    }
}

impl<const N: usize> Eq for ObfuscatedString<N> {}

impl<const N: usize> core::hash::Hash for ObfuscatedString<N> {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        core::hash::Hash::hash(self.as_str(), state);
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

/// A heap-allocation-free obfuscated numeric value.
///
/// The value is encrypted during compile-time evaluation, reducing
/// the likelihood of plaintext values appearing directly in the binary.
///
/// Note: intentionally does not implement `Clone` — cloning would
/// create a second copy of the decrypted plaintext in memory that
/// this crate cannot track or zeroize on your behalf. `get()` also
/// returns a plaintext copy that this crate cannot zeroize for you.
#[must_use = "obfuscated value is immediately dropped if unused"]
pub struct ObfuscatedValue<T, const N: usize> {
    data: [u8; N],
    seed: u64,
    _marker: PhantomData<T>,
}

impl<T, const N: usize> Drop for ObfuscatedValue<T, N> {
    #[inline]
    fn drop(&mut self) {
        self.data.zeroize();
        self.seed.zeroize();
    }
}

impl<T, const N: usize> ObfuscatedValue<T, N>
where
    T: Obfuscatable<N>,
{
    /// Wraps already-encrypted bytes together with the seed needed to
    /// decrypt them.
    ///
    /// Primarily intended to be called by the `obfval!` macro, with
    /// bytes produced by [`detail::encrypt_array`] at compile time.
    #[inline(never)]
    pub const fn new(data: [u8; N], seed: u64, _marker: PhantomData<T>) -> Self {
        Self {
            data: core::hint::black_box(data),
            seed,
            _marker: PhantomData,
        }
    }

    /// Returns the decrypted value.
    ///
    /// The returned value is a plaintext copy and is not automatically
    /// zeroized — avoid storing or logging it longer than necessary.
    #[inline]
    #[must_use]
    pub fn get(&self) -> T {
        // XOR is its own inverse: running the same keystream-based
        // XOR a second time recovers the original plaintext bytes.
        let decrypted = detail::encrypt_array(self.data, self.seed);
        T::from_ne_bytes(core::hint::black_box(decrypted))
    }
}

impl<T, const N: usize> fmt::Debug for ObfuscatedValue<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ObfuscatedValue").field("len", &N).finish()
    }
}

impl<T, const N: usize> PartialEq for ObfuscatedValue<T, N>
where
    T: Obfuscatable<N> + PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl<T, const N: usize> Eq for ObfuscatedValue<T, N> where T: Obfuscatable<N> + Eq {}

impl<T, const N: usize> core::hash::Hash for ObfuscatedValue<T, N>
where
    T: Obfuscatable<N> + core::hash::Hash,
{
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.get().hash(state);
    }
}

impl<T, const N: usize> PartialEq<T> for ObfuscatedValue<T, N>
where
    T: Obfuscatable<N> + PartialEq,
{
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.get() == *other
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

/// Obfuscates an integer expression at compile time.
///
/// Supports all signed and unsigned integer types. The expression is
/// evaluated and encrypted during compile-time evaluation, reducing
/// the likelihood of plaintext values appearing directly in the
/// compiled binary.
///
/// # Examples
///
/// ```
/// use obfus::obfval;
///
/// let unsigned = obfval!(123u32);
/// let signed = obfval!(-123i64);
///
/// assert_eq!(unsigned.get(), 123u32);
/// assert_eq!(signed.get(), -123i64);
/// ```
///
/// # Notes
///
/// This macro provides obfuscation, not cryptographic protection. The
/// obfuscated value can still be recovered through runtime analysis
/// (e.g. inspecting the result of `get()` with a debugger).
#[macro_export]
macro_rules! obfval {
    ($value:expr) => {{
        const SEED: u64 = $crate::detail::seed(file!(), line!(), column!(), stringify!($value));

        let (encrypted, marker) = const {
            let value = $value;
            let plain = value.to_ne_bytes();
            let encrypted = $crate::detail::encrypt_array(plain, SEED);
            let marker = $crate::detail::phantom_of(value);

            (encrypted, marker)
        };

        $crate::ObfuscatedValue::new(encrypted, SEED, marker)
    }};
}