use super::client::urlencoded;

#[test]
fn test_urlencoded_basic() {
    assert_eq!(urlencoded("hello"), "hello");
    assert_eq!(urlencoded("rust-skill"), "rust-skill");
    assert_eq!(urlencoded("my_skill"), "my_skill");
    assert_eq!(urlencoded("v1.0.0"), "v1.0.0");
    // Unicode: multi-byte characters must be percent-encoded per byte
    assert_eq!(urlencoded("caf\u{00e9}"), "caf%C3%A9");
    assert_eq!(urlencoded("\u{1f680}"), "%F0%9F%9A%80");
}

#[test]
fn test_urlencoded_spaces() {
    assert_eq!(urlencoded("hello world"), "hello%20world");
    assert_eq!(urlencoded("my skill name"), "my%20skill%20name");
}

#[test]
fn test_urlencoded_special_chars() {
    assert_eq!(urlencoded("a&b"), "a%26b");
    assert_eq!(urlencoded("key=value"), "key%3Dvalue");
    assert_eq!(urlencoded("1+1"), "1%2B1");
    assert_eq!(urlencoded("section#2"), "section%232");
}
