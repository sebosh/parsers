use std::{collections::HashMap, ops::Index};

use super::Error;

#[derive(Debug, PartialEq, Clone)]
pub enum JsonValue {
  Null,
  String(String),
  Number(f64),
  Boolean(bool),
  Array(Vec<JsonValue>),
  Object(HashMap<String, JsonValue>),
}

impl Index<usize> for JsonValue {
  type Output = JsonValue;

  fn index(&self, index: usize) -> &Self::Output {
    match self {
      JsonValue::Array(arr) => &arr[index],
      _ => panic!("not an array"),
    }
  }
}

impl Index<&str> for JsonValue {
  type Output = JsonValue;

  fn index(&self, key: &str) -> &Self::Output {
    match self {
      JsonValue::Object(obj) => &obj[key],
      _ => panic!("not an object"),
    }
  }
}

#[derive(PartialEq, Clone)]
enum JsonToken {
  Null { pos: usize },
  String { val: String, pos: usize },
  Number { val: f64, pos: usize },
  Boolean { val: bool, pos: usize },
  Colon { pos: usize },
  Comma { pos: usize },
  LeftBracket { pos: usize },
  RightBracket { pos: usize },
  LeftBrace { pos: usize },
  RightBrace { pos: usize },
  Eof { pos: usize },
}

struct JsonLexer {
  json:  String,
  index: usize,
}

impl JsonLexer {
  pub fn new(json: String) -> Self {
    Self {
      json:  json.trim().to_string(),
      index: 0,
    }
  }

  fn advance(&mut self) -> Option<char> {
    self.index += 1;
    self.json.chars().nth(self.index)
  }

  fn current(&self) -> Option<char> { self.json.chars().nth(self.index) }

  fn make_string(&mut self) -> Result<JsonToken, Error> {
    let start = self.index;
    let mut result = String::new();
    while let Some(c) = self.advance() {
      match c {
        '"' => {
          self.advance();
          break;
        },
        '\\' => match self.advance() {
          Some('"') => result.push('"'),
          Some('\\') => result.push('\\'),
          Some('/') => result.push('/'),
          Some('b') => result.push('\x08'),
          Some('f') => result.push('\x0C'),
          Some('n') => result.push('\n'),
          Some('r') => result.push('\r'),
          Some('t') => result.push('\t'),
          Some('u') => todo!("unicode escape sequences"),
          Some(c) => {
            return Err(Error::json(
              format!("Invalid escape sequence '\\{}'", c),
              self.index - 1,
            ))
          },
          None => {
            return Err(Error::json(
              "Unexpected end of string".to_string(),
              self.index,
            ))
          },
        },
        _ => result.push(c),
      }
    }
    Ok(JsonToken::String {
      val: result,
      pos: start,
    })
  }

  fn make_number(&mut self) -> Result<JsonToken, Error> {
    let start = self.index;
    let mut result = String::new();
    while let Some(c) = self.current() {
      match c {
        '0'..='9' | '.' | '-' | '+' | 'e' | 'E' => {
          result.push(c);
          self.advance();
        },
        _ => break,
      }
    }
    match result.parse::<f64>() {
      Ok(n) => Ok(JsonToken::Number { val: n, pos: start }),
      Err(_) => Err(Error::json(format!("Invalid number '{}'", result), start)),
    }
  }

  fn make_keyword(&mut self) -> Result<JsonToken, Error> {
    let start = self.index;
    let mut result = String::new();
    while let Some(c) = self.current() {
      match c {
        'a'..='z' => {
          result.push(c);
          self.advance();
        },
        _ => break,
      }
    }
    match result.as_str() {
      "null" => Ok(JsonToken::Null { pos: start }),
      "true" => Ok(JsonToken::Boolean {
        val: true,
        pos: start,
      }),
      "false" => Ok(JsonToken::Boolean {
        val: false,
        pos: start,
      }),
      _ => Err(Error::json(format!("Unexpected '{}'", result), start)),
    }
  }

  fn make_symbol(&mut self) -> Result<JsonToken, Error> {
    let start = self.index;
    let result = match self.current() {
      Some(':') => JsonToken::Colon { pos: start },
      Some(',') => JsonToken::Comma { pos: start },
      Some('[') => JsonToken::LeftBracket { pos: start },
      Some(']') => JsonToken::RightBracket { pos: start },
      Some('{') => JsonToken::LeftBrace { pos: start },
      Some('}') => JsonToken::RightBrace { pos: start },
      Some(c) => return Err(Error::json(format!("Unexpected '{}'", c), start)),
      None => JsonToken::Eof { pos: start },
    };
    self.advance();
    Ok(result)
  }

  pub fn lex(&mut self) -> Result<Vec<JsonToken>, Error> {
    let mut result = vec![];
    while let Some(c) = self.current() {
      match c {
        ' ' | '\t' | '\n' | '\r' => {
          while self.current().map(|c| c.is_whitespace()).unwrap_or(false) {
            self.advance();
          }
        },
        '"' => result.push(self.make_string()?),
        '0'..='9' | '-' => result.push(self.make_number()?),
        'f'..='t' => result.push(self.make_keyword()?),
        ':' | ',' | '[' | ']' | '{' | '}' => result.push(self.make_symbol()?),
        _ => return Err(Error::json(format!("Unexpected '{}'", c), self.index)),
      }
    }
    Ok(result)
  }
}

struct JsonParser {
  json:   String,
  tokens: Vec<JsonToken>,
  index:  usize,
}

impl JsonParser {
  pub fn new(json: String) -> Self {
    Self {
      json:   json.trim().to_string(),
      tokens: vec![],
      index:  0,
    }
  }

  fn advance(&mut self) -> Option<JsonToken> {
    self.index += 1;
    self.tokens.get(self.index).cloned()
  }

  fn current(&self) -> Option<JsonToken> { self.tokens.get(self.index).cloned() }

  fn parse_object(&mut self) -> Result<JsonValue, Error> {
    let mut result = HashMap::new();
    while let Some(token) = self.advance() {
      match token {
        JsonToken::RightBrace { .. } => return Ok(JsonValue::Object(result)),
        JsonToken::String { val, pos } => {
          match self.advance() {
            Some(JsonToken::Colon { .. }) => (),
            Some(
              JsonToken::Boolean { pos, .. }
              | JsonToken::Comma { pos, .. }
              | JsonToken::Eof { pos, .. }
              | JsonToken::LeftBrace { pos, .. }
              | JsonToken::LeftBracket { pos, .. }
              | JsonToken::Null { pos, .. }
              | JsonToken::Number { pos, .. }
              | JsonToken::RightBrace { pos, .. }
              | JsonToken::RightBracket { pos, .. }
              | JsonToken::String { pos, .. },
            ) => return Err(Error::json("Expected ':'".to_string(), pos)),
            None => unreachable!(),
          }
          if result.contains_key(&val) {
            return Err(Error::json(format!("Duplicate key '{}'", val), pos));
          }
          self.advance();
          let value = self.parse_value()?;
          result.insert(val, value);
          match self.advance() {
            Some(JsonToken::Comma { .. }) => (),
            Some(JsonToken::RightBrace { .. }) => return Ok(JsonValue::Object(result)),
            _ => return Err(Error::json("Expected ',' or '}'".to_string(), self.index)),
          }
        },
        JsonToken::Null { pos, .. }
        | JsonToken::Number { pos, .. }
        | JsonToken::Boolean { pos, .. }
        | JsonToken::Colon { pos, .. }
        | JsonToken::Comma { pos, .. }
        | JsonToken::Eof { pos, .. }
        | JsonToken::LeftBrace { pos, .. }
        | JsonToken::LeftBracket { pos, .. }
        | JsonToken::RightBracket { pos, .. } => {
          return Err(Error::json("Expected string".to_string(), pos))
        },
      }
    }

    Ok(JsonValue::Object(result))
  }

  fn parse_array(&mut self) -> Result<JsonValue, Error> {
    let mut result = Vec::new();
    while let Some(token) = self.advance() {
      match token {
        JsonToken::RightBracket { .. } => return Ok(JsonValue::Array(result)),
        JsonToken::Colon { pos } | JsonToken::Comma { pos } => {
          return Err(Error::json("Expected a value".to_string(), pos))
        },
        _ => {
          let value = self.parse_value()?;
          result.push(value);
          match self.advance() {
            Some(JsonToken::Comma { .. }) => (),
            Some(JsonToken::RightBracket { .. }) => return Ok(JsonValue::Array(result)),
            _ => return Err(Error::json("Expected ',' or ']'".to_string(), self.index)),
          }
        },
      }
    }

    Ok(JsonValue::Array(result))
  }

  fn parse_value(&mut self) -> Result<JsonValue, Error> {
    let val = match self.current() {
      Some(JsonToken::String { val, .. }) => JsonValue::String(val),
      Some(JsonToken::Number { val, .. }) => JsonValue::Number(val),
      Some(JsonToken::Boolean { val, .. }) => JsonValue::Boolean(val),
      Some(JsonToken::Null { .. }) => JsonValue::Null,
      Some(JsonToken::LeftBrace { .. }) => self.parse_object()?,
      Some(JsonToken::LeftBracket { .. }) => self.parse_array()?,
      Some(JsonToken::Colon { pos }) => return Err(Error::json("Unexpected ':'".to_string(), pos)),
      Some(JsonToken::Comma { pos }) => return Err(Error::json("Unexpected ','".to_string(), pos)),
      Some(JsonToken::RightBrace { pos }) => {
        return Err(Error::json("Unexpected '}'".to_string(), pos))
      },
      Some(JsonToken::RightBracket { pos }) => {
        return Err(Error::json("Unexpected ']'".to_string(), pos))
      },
      Some(JsonToken::Eof { pos }) => {
        return Err(Error::json("Unexpected end of input".to_string(), pos))
      },
      None => unreachable!(),
    };

    Ok(val)
  }

  pub fn parse(&mut self) -> Result<JsonValue, Error> {
    let mut lexer = JsonLexer::new(self.json.clone());
    self.tokens = lexer.lex()?;
    self.parse_value()
  }
}

fn generate_json(val: JsonValue, pretty: i32, level: i32) -> String {
  match val {
    JsonValue::Null => "null".to_string(),
    JsonValue::String(s) => format!(
      "\"{}\"",
      s.replace('\\', "\\\\")
        .replace('/', "\\/")
        .replace('\"', "\\\"")
        .replace('\x08', "\\b")
        .replace('\x0C', "\\f")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
    ),
    JsonValue::Number(n) => n.to_string(),
    JsonValue::Boolean(b) => b.to_string(),
    JsonValue::Array(arr) => {
      if arr.is_empty() {
        return "[]".to_string();
      }
      let mut result = "[".to_string();
      for (i, v) in arr.iter().enumerate() {
        if pretty == 1 {
          result.push(' ');
        } else if pretty == 2 {
          result.push_str(("\n".to_string() + "  ".repeat((level + 1) as usize).as_str()).as_str());
        }
        result.push_str(&generate_json(v.clone(), pretty, level + 1));
        if i < arr.len() - 1 {
          result.push(',');
        }
      }
      if pretty == 1 {
        result.push(' ');
      } else if pretty == 2 {
        result.push_str(("\n".to_string() + "  ".repeat(level as usize).as_str()).as_str());
      }
      result.push(']');
      result
    },
    JsonValue::Object(obj) => {
      if obj.is_empty() {
        return "{}".to_string();
      }
      let mut result = "{".to_string();
      for (i, (k, v)) in obj.iter().enumerate() {
        if pretty == 1 {
          result.push(' ');
        } else if pretty == 2 {
          result.push_str(("\n".to_string() + "  ".repeat((level + 1) as usize).as_str()).as_str());
        }
        result.push_str(&format!(
          "\"{}\":{}{}",
          k,
          if [1, 2].contains(&pretty) { " " } else { "" },
          generate_json(v.clone(), pretty, level + 1)
        ));
        if i < obj.len() - 1 {
          result.push(',');
        }
      }
      if pretty == 1 {
        result.push(' ');
      } else if pretty == 2 {
        result.push_str(("\n".to_string() + "  ".repeat(level as usize).as_str()).as_str());
      }
      result.push('}');
      result
    },
  }
}

/// Struct with methods for parsing and stringifying JSON similar to the
/// JavaScript JSON object.
pub struct JSON {}

impl JSON {
  /// Parses a JSON string and returns a JsonValue struct.
  ///
  /// # Arguments
  ///
  /// - `json` - The JSON string to parse.
  ///
  /// # Errors
  ///
  /// Returns an Error if the JSON string is invalid.
  pub fn parse(json: String) -> Result<JsonValue, Error> {
    let mut parser = JsonParser::new(json);
    parser.parse()
  }

  /// Stringifies a JsonValue struct and returns a JSON string.
  ///
  /// # Arguments
  ///
  /// - `value` - The JsonValue struct to stringify.
  /// - `pretty` - The level of pretty formatting to use. 0 for no whitespace, 1
  ///   for spaces, 2 for newlines.
  ///
  /// # Errors
  ///
  /// Never returns an Error.
  pub fn stringify(value: JsonValue, pretty: i32) -> String { generate_json(value, pretty, 0) }
}
