use obfus::obfstr;

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
