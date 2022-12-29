use parsers::json::*;

fn main() {
  println!(
    "{}",
    JSON::stringify(
      JSON::parse("[1,2,3.14159265,\"s\"]".to_string()).unwrap(),
      2
    )
  );
}
