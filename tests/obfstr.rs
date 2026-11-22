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
    assert_eq!(obfstr!("1234567"), "1234567");
    assert_eq!(obfstr!("12345678"), "12345678");
    assert_eq!(obfstr!("123456789"), "123456789");
    assert_eq!(obfstr!("0123456789abcdef"), "0123456789abcdef");
    assert_eq!(obfstr!("0123456789abcdefg"), "0123456789abcdefg");
}

#[test]
fn multiple_on_same_line() {
    let (a, b) = (obfstr!("alpha_key"), obfstr!("beta_key"));

    assert_eq!(a, "alpha_key");
    assert_eq!(b, "beta_key");
}