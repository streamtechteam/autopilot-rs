use super::*;

fn parse_json(input: &str) -> serde_json::Value {
    serde_json::from_str(&parse(input)).unwrap()
}

#[test]
fn removes_line_and_block_comments() {
    let value = parse_json(
        r#"{
            // line comment
            "a": 1, /* block comment */
            "b": 2
        }"#,
    );

    assert_eq!(value["a"], 1);
    assert_eq!(value["b"], 2);
}

#[test]
fn removes_trailing_commas_in_objects_and_arrays() {
    let value = parse_json(r#"{"a": [1, 2, 3,], "b": {"c": true,},}"#);

    assert_eq!(value["a"], serde_json::json!([1, 2, 3]));
    assert_eq!(value["b"]["c"], true);
}

#[test]
fn preserves_comment_markers_inside_strings() {
    let value = parse_json(
        r#"{
            "url": "https://example.com/a//b",
            "text": "not /* a comment */ here"
        }"#,
    );

    assert_eq!(value["url"], "https://example.com/a//b");
    assert_eq!(value["text"], "not /* a comment */ here");
}

#[test]
fn handles_escaped_quotes_inside_strings() {
    let value = parse_json(r#"{"text": "a \"quoted\" value", "next": 1}"#);

    assert_eq!(value["text"], "a \"quoted\" value");
    assert_eq!(value["next"], 1);
}

#[test]
fn returns_the_input_when_there_is_nothing_to_strip() {
    assert_eq!(parse("{\"a\":1}"), "{\"a\":1}");
}
