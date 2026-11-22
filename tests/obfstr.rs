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
