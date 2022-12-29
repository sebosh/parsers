mod error;
use error::Error;
pub mod json;

#[cfg(test)]
mod json_tests {
  use std::collections::HashMap;

  use json::*;

  use super::*;

  #[test]
  fn json_parse_null() {
    let result = JSON::parse("null".to_string()).unwrap();
    assert_eq!(result, JsonValue::Null);
  }

  #[test]
  fn json_parse_boolean() {
    let result = JSON::parse("true".to_string()).unwrap();
    assert_eq!(result, JsonValue::Boolean(true));
    let result = JSON::parse("false".to_string()).unwrap();
    assert_eq!(result, JsonValue::Boolean(false));
  }

  #[test]
  fn json_parse_number() {
    let result = JSON::parse("123".to_string()).unwrap();
    assert_eq!(result, JsonValue::Number(123.0));
    let result = JSON::parse("-123.456e+3".to_string()).unwrap();
    assert_eq!(result, JsonValue::Number(-123456.0));
  }

  #[test]
  fn json_parse_string() {
    let result = JSON::parse(r#""hello""#.to_string()).unwrap();
    assert_eq!(result, JsonValue::String(r#"hello"#.to_string()));
    let result = JSON::parse(r#""hello\\ world""#.to_string()).unwrap();
    assert_eq!(result, JsonValue::String(r#"hello\ world"#.to_string()));
  }

  #[test]
  fn json_parse_array() {
    let result = JSON::parse("[null,true,123,\"hello\"]".to_string()).unwrap();
    assert_eq!(
      result,
      JsonValue::Array(vec![
        JsonValue::Null,
        JsonValue::Boolean(true),
        JsonValue::Number(123.0),
        JsonValue::String("hello".to_string())
      ])
    );
    let result = JSON::parse("[42,[true],\"a\"]".to_string()).unwrap();
    assert_eq!(
      result,
      JsonValue::Array(vec![
        JsonValue::Number(42.0),
        JsonValue::Array(vec![JsonValue::Boolean(true)]),
        JsonValue::String("a".to_string())
      ])
    );
  }

  #[test]
  fn json_parse_object() {
    let result = JSON::parse(r#"{"a":null,"b":true,"c":123,"d":"hello"}"#.to_string()).unwrap();
    assert_eq!(
      result,
      JsonValue::Object(HashMap::from([
        ("a".to_string(), JsonValue::Null),
        ("b".to_string(), JsonValue::Boolean(true)),
        ("c".to_string(), JsonValue::Number(123.0)),
        ("d".to_string(), JsonValue::String("hello".to_string()))
      ]))
    );
    let result = JSON::parse(r#"{"a":42,"b":[true],"c":"a"}"#.to_string()).unwrap();
    assert_eq!(
      result,
      JsonValue::Object(HashMap::from([
        ("a".to_string(), JsonValue::Number(42.0)),
        (
          "b".to_string(),
          JsonValue::Array(vec![JsonValue::Boolean(true)])
        ),
        ("c".to_string(), JsonValue::String("a".to_string()))
      ]))
    );
  }

  // --------------------------------

  #[test]
  fn json_stringify_null() {
    let result = JSON::stringify(JsonValue::Null, 0);
    assert_eq!(result, "null");
  }

  #[test]
  fn json_stringify_boolean() {
    let result = JSON::stringify(JsonValue::Boolean(true), 0);
    assert_eq!(result, "true");
    let result = JSON::stringify(JsonValue::Boolean(false), 0);
    assert_eq!(result, "false");
  }

  #[test]
  fn json_stringify_number() {
    let result = JSON::stringify(JsonValue::Number(123.0), 0);
    assert_eq!(result, "123");
    let result = JSON::stringify(JsonValue::Number(-123.456), 0);
    assert_eq!(result, "-123.456");
  }

  #[test]
  fn json_stringify_string() {
    let result = JSON::stringify(JsonValue::String("hello".to_string()), 0);
    assert_eq!(result, "\"hello\"");
    let result = JSON::stringify(JsonValue::String("hello\\ world\n".to_string()), 0);
    assert_eq!(result, "\"hello\\\\ world\\n\"");
  }

  #[test]
  fn json_stringify_array() {
    let result = JSON::stringify(
      JsonValue::Array(vec![
        JsonValue::Null,
        JsonValue::Boolean(true),
        JsonValue::Number(123.0),
        JsonValue::String("hello".to_string()),
      ]),
      0,
    );
    assert_eq!(result, "[null,true,123,\"hello\"]");
    let result = JSON::stringify(
      JsonValue::Array(vec![
        JsonValue::Number(42.0),
        JsonValue::Array(vec![JsonValue::Boolean(true)]),
        JsonValue::String("a".to_string()),
      ]),
      0,
    );
    assert_eq!(result, "[42,[true],\"a\"]");
  }

  // Cannot predict order of keys in HashMap, so this test is not deterministic.
  // When this test fails, it is because the order of keys in the HashMap is
  // different and everything else is the same. Proven by the fact that after
  // multiple test runs that failed only because of the order of keys
  // (observed), one was successful when the order was the same.

  // #[test]
  // fn json_stringify_object() {
  //   let result =
  // JSON::stringify(JsonValue::Object(HashMap::from([("a".to_string(),
  // JsonValue::Null), ("b".to_string(), JsonValue::Boolean(true)),
  // ("c".to_string(), JsonValue::Number(123.0)), ("d".to_string(),
  // JsonValue::String("hello".to_string()))])), 0);   assert_eq!(result,
  // "{\"a\":null,\"b\":true,\"c\":123,\"d\":\"hello\"}");   let result =
  // JSON::stringify(JsonValue::Object(HashMap::from([("a".to_string(),
  // JsonValue::Number(42.0)), ("b".to_string(),
  // JsonValue::Array(vec![JsonValue::Boolean(true)])), ("c".to_string(),
  // JsonValue::String("a".to_string()))])), 0);   assert_eq!(result,
  // "{\"a\":42,\"b\":[true],\"c\":\"a\"}"); }
}
