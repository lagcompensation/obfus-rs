use obfus::obfstr;
use obfus::obfval;

#[test]
fn basic_string() {
    let value = obfstr!("Hello, World!");

    assert_eq!(value, "Hello, World!");
}

#[test]
fn unicode_string() {
    let value = obfstr!("Привет, мир! Привіт, світе!");

    assert_eq!(value, "Привет, мир! Привіт, світе!");
}

#[test]
fn empty_string() {
    let value = obfstr!("");

    assert_eq!(value, "");
}

#[test]
fn special_characters() {
    let value = obfstr!("!@#$%^&*()_+-=[]{}");

    assert_eq!(value, "!@#$%^&*()_+-=[]{}");
}

#[test]
fn multiline_string() {
    let value = obfstr!("First line\nSecond line");

    assert_eq!(value, "First line\nSecond line");
}

#[test]
fn block_boundaries() {
    let seven_bytes = obfstr!("1234567");
    let eight_bytes = obfstr!("12345678");
    let nine_bytes = obfstr!("123456789");
    let sixteen_bytes = obfstr!("0123456789abcdef");
    let seventeen_bytes = obfstr!("0123456789abcdefg");

    assert_eq!(seven_bytes, "1234567");
    assert_eq!(eight_bytes, "12345678");
    assert_eq!(nine_bytes, "123456789");
    assert_eq!(sixteen_bytes, "0123456789abcdef");
    assert_eq!(seventeen_bytes, "0123456789abcdefg");
}

#[test]
fn multiple_on_same_line() {
    let (a, b) = (obfstr!("alpha_key"), obfstr!("beta_key"));

    assert_eq!(a, "alpha_key");
    assert_eq!(b, "beta_key");
}

#[test]
fn identical_strings_at_different_call_sites() {
    let (a, b) = (obfstr!("secret"), obfstr!("secret"));

    assert_eq!(a, "secret");
    assert_eq!(b, "secret");
    assert_eq!(a, b);
}

#[test]
fn multiple_call_sites() {
    let (a, b, c) = (
        obfstr!("same_string"),
        obfstr!("same_string"),
        obfstr!("different_string"),
    );

    assert_eq!(a, "same_string");
    assert_eq!(b, "same_string");
    assert_eq!(c, "different_string");
}

#[test]
fn hash_consistent_with_eq() {
    use core::hash::{Hash, Hasher};

    fn hash_of<T: Hash>(value: &T) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    let a = obfstr!("same content");
    let b = obfstr!("same content");

    assert_eq!(a, b);
    assert_eq!(hash_of(&a), hash_of(&b));
}

#[test]
fn different_length_not_equal() {
    let short = obfstr!("abc");
    let long = obfstr!("abcdef");

    assert_ne!(short, long);
}

#[test]
fn obfval_roundtrip_u32() {
    let value = obfval!(123u32);
    assert_eq!(value.get(), 123u32);
    assert_eq!(value, 123u32);
}

#[test]
fn obfval_roundtrip_i64_negative() {
    let value = obfval!(-123i64);
    assert_eq!(value.get(), -123i64);
}

#[test]
fn obfval_const_expr() {
    const X: u16 = 7 * 6;
    let value = obfval!(X);
    assert_eq!(value.get(), 42u16);
}

#[test]
fn obfval_eq_between_instances() {
    let a = obfval!(999u64);
    let b = obfval!(999u64);
    assert_eq!(a, b);
}
