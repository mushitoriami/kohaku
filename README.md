# kohaku
A simple tokenizer that handles alphanumeric characters, underscore (`_`), whitespace, and some specified keywords.

```rust
let mut tokenizer = kohaku::Tokenizer::new(["->", "<-", "{", "}"]);

assert_eq!(
    tokenizer
        .tokenize(r#"{abc -> "123 <- 456"}"#)
        .collect::<Vec<Result<&str, usize>>>(),
    [Ok("{"), Ok("abc"), Ok("->"), Ok(r#""123 <- 456""#), Ok("}")]
);

assert_eq!(
    tokenizer
        .tokenize("{abc -> 1-3}")
        .collect::<Vec<Result<&str, usize>>>(),
    [Ok("{"), Ok("abc"), Ok("->"), Ok("1"), Err(10)]
);
```
