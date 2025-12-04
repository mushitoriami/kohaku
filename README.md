# kohaku
A simple tokenizer

```rust
use kohaku::Tokenizer;

assert_eq!(
    r#"{abc -> "123 <- 456"}"#
        .tokenize(["->", "<-", "{", "}"])
        .collect::<Vec<Result<&str, usize>>>(),
    [Ok("{"), Ok("abc"), Ok("->"), Ok(r#""123 <- 456""#), Ok("}")]
);

assert_eq!(
    "{abc -> 1-3}"
        .tokenize(["->", "<-", "{", "}"])
        .collect::<Vec<Result<&str, usize>>>(),
    [Ok("{"), Ok("abc"), Ok("->"), Ok("1"), Err(10)]
);
```
