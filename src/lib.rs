#![forbid(unsafe_code)]

#[doc(hidden)]
pub mod __private {
    /// SplitMix64 mixing function.
    #[inline]
    pub const fn splitmix64(mut value: u64) -> u64 {
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);

        value ^ (value >> 31)
    }

    /// Hashes a string at compile time.
    pub const fn hash(data: &str) -> u64 {
        let bytes = data.as_bytes();

        let mut hash: u64 = 0;
        let mut i = 0;

        while i < data.len() {
            let byte = bytes[i].to_ascii_lowercase();

            hash = hash.wrapping_add(byte as u64);
            hash = hash.wrapping_add(hash << 8);
            hash ^= hash >> 11;

            i += 1;
        }

        hash = hash.wrapping_add(hash << 5);
        hash ^= hash >> 13;
        hash = hash.wrapping_add(hash << 10);

        hash
    }

    /// Generates a compile-time seed.
    pub const fn seed(file: &str, line: u32, column: u32) -> u64 {
        let hash = hash(file);

        splitmix64(
            hash.wrapping_add((line as u64).wrapping_mul(0x9e3779b97f4a7c15))
                .wrapping_add((column as u64).wrapping_mul(0xbf58476d1ce4e5b9)),
        )
    }

    /// Encrypts bytes at compile time.
    pub const fn encrypt<const N: usize>(data: &[u8], seed: u64) -> [u8; N] {
        let mut output = [0u8; N];
        let mut i = 0;

        while i < N {
            let block = i / 8;
            let offset = i % 8;

            let key = splitmix64(seed.wrapping_add(block as u64));
            let mask = (key >> (offset * 8)) as u8;

            output[i] = data[i] ^ mask;

            i += 1;
        }

        output
    }

    /// Decrypts bytes at runtime.
    #[inline]
    pub fn decrypt(data: &[u8], seed: u64) -> Result<String, std::string::FromUtf8Error> {
        let mut output = Vec::with_capacity(data.len());
        let mut key = 0u64;

        for (index, &byte) in data.iter().enumerate() {
            let block = index / 8;
            let offset = index % 8;

            if offset == 0 {
                key = splitmix64(seed.wrapping_add(block as u64));
            }

            let mask = (key >> (offset * 8)) as u8;
            output.push(byte ^ mask);
        }

        String::from_utf8(output)
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
        const SEED: u64 = $crate::__private::seed(file!(), line!(), column!());

        const ENCRYPTED: [u8; $value.len()] =
            $crate::__private::encrypt::<{ $value.len() }>($value.as_bytes(), SEED);

        $crate::__private::decrypt(&ENCRYPTED, SEED).unwrap()
    }};
}
