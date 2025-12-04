use std::collections::HashMap;
use std::iter::Peekable;
use std::str::CharIndices;

pub struct TokenIterator<'a> {
    input: &'a str,
    state: State,
    iter: Peekable<CharIndices<'a>>,
}

impl<'a> TokenIterator<'a> {
    fn skip_literal(iter: &mut Peekable<CharIndices>) -> bool {
        if iter.peek().is_some_and(|x| x.1 == '"') {
            iter.next();
            while iter.next().is_some_and(|x| x.1 != '"') {}
            return true;
        }
        false
    }

    fn skip_with_condition(iter: &mut Peekable<CharIndices>, condition: fn(char) -> bool) -> bool {
        let index = iter.peek().map(|x| x.0);
        while iter.peek().is_some_and(|x| condition(x.1)) {
            iter.next();
        }
        index != iter.peek().map(|x| x.0)
    }

    fn skip_alphanumeric(iter: &mut Peekable<CharIndices>) -> bool {
        Self::skip_with_condition(iter, |c| c.is_alphanumeric() || c == '_')
    }

    fn skip_whitespace(iter: &mut Peekable<CharIndices>) -> bool {
        Self::skip_with_condition(iter, char::is_whitespace)
    }

    fn skip_with_state(iter: &mut Peekable<CharIndices>, mut state: &State) -> bool {
        while let Some(next_state) = iter.peek().and_then(|x| state.trans.get(&x.1)) {
            iter.next();
            state = next_state;
        }
        state.is_end_state
    }

    fn skip_token(iter: &mut Peekable<CharIndices>, state: &State) -> bool {
        Self::skip_alphanumeric(iter)
            || Self::skip_whitespace(iter)
            || Self::skip_literal(iter)
            || Self::skip_with_state(iter, state)
    }

    fn take_token(
        iter: &mut Peekable<CharIndices>,
        state: &State,
        input: &'a str,
    ) -> Option<Result<&'a str, usize>> {
        let index_start = iter.peek().map(|x| x.0)?;
        let is_success = Self::skip_token(iter, state);
        let index_end = iter.peek().map(|x| x.0).unwrap_or(input.len());
        match is_success {
            true => Some(Ok(&input[index_start..index_end])),
            false => Some(Err(index_end)),
        }
    }
}

impl<'a> Iterator for TokenIterator<'a> {
    type Item = Result<&'a str, usize>;

    fn next(&mut self) -> Option<Self::Item> {
        let token = Self::take_token(&mut self.iter, &self.state, self.input)?;
        if token.is_err() {
            self.iter = "".char_indices().peekable();
        } else if token.is_ok_and(|t| t.chars().next().is_some_and(char::is_whitespace)) {
            return self.next();
        }
        Some(token)
    }
}

#[derive(Debug, PartialEq)]
struct State {
    is_end_state: bool,
    trans: HashMap<char, State>,
}

impl State {
    fn new() -> Self {
        State {
            is_end_state: false,
            trans: HashMap::new(),
        }
    }

    fn add_path(&mut self, mut chars: impl Iterator<Item = char>) {
        match chars.next() {
            Some(c) => self.trans.entry(c).or_insert(State::new()).add_path(chars),
            None => self.is_end_state = true,
        }
    }
}

pub trait Tokenizer {
    fn tokenize<'a>(
        &'a self,
        keywords: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> TokenIterator<'a>;
}

impl Tokenizer for str {
    fn tokenize<'a>(
        &'a self,
        keywords: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> TokenIterator<'a> {
        let mut state = State::new();
        for keyword in keywords {
            state.add_path(keyword.as_ref().chars());
        }
        TokenIterator {
            input: self,
            state: state,
            iter: self.char_indices().peekable(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_add_1() {
        let mut state = State::new();
        for k in ["->", "<-", "{", "}"] {
            state.add_path(k.chars());
        }
        assert_eq!(
            state,
            State {
                is_end_state: false,
                trans: HashMap::from([
                    (
                        '{',
                        State {
                            is_end_state: true,
                            trans: HashMap::new()
                        }
                    ),
                    (
                        '}',
                        State {
                            is_end_state: true,
                            trans: HashMap::new()
                        }
                    ),
                    (
                        '-',
                        State {
                            is_end_state: false,
                            trans: HashMap::from([(
                                '>',
                                State {
                                    is_end_state: true,
                                    trans: HashMap::new()
                                }
                            )])
                        }
                    ),
                    (
                        '<',
                        State {
                            is_end_state: false,
                            trans: HashMap::from([(
                                '-',
                                State {
                                    is_end_state: true,
                                    trans: HashMap::new()
                                }
                            )])
                        }
                    ),
                ]),
            }
        );
    }

    #[test]
    fn test_state_add_2() {
        let mut state = State::new();
        for k in ["->", "-", "*"] {
            state.add_path(k.chars());
        }
        assert_eq!(
            state,
            State {
                is_end_state: false,
                trans: HashMap::from([
                    (
                        '*',
                        State {
                            is_end_state: true,
                            trans: HashMap::new()
                        }
                    ),
                    (
                        '-',
                        State {
                            is_end_state: true,
                            trans: HashMap::from([(
                                '>',
                                State {
                                    is_end_state: true,
                                    trans: HashMap::new()
                                }
                            )])
                        }
                    ),
                ]),
            }
        );
    }

    #[test]
    fn test_tokenizer_1() {
        assert_eq!(
            "{aaa ->bbb }"
                .tokenize(vec![
                    String::from("->"),
                    String::from("<-"),
                    String::from("{"),
                    String::from("}"),
                ])
                .collect::<Vec<Result<&str, usize>>>(),
            vec![Ok("{"), Ok("aaa"), Ok("->"), Ok("bbb"), Ok("}")]
        );
    }

    #[test]
    fn test_tokenizer_2() {
        assert_eq!(
            "{inst_1 -> inst_2 -> {inst_4 <- inst_3} -> inst_5}"
                .tokenize([
                    String::from("->"),
                    String::from("<-"),
                    String::from("{"),
                    String::from("}"),
                ])
                .map(Result::unwrap)
                .collect::<Vec<&str>>(),
            vec![
                "{", "inst_1", "->", "inst_2", "->", "{", "inst_4", "<-", "inst_3", "}", "->",
                "inst_5", "}"
            ]
        );
    }

    #[test]
    fn test_tokenizer_3() {
        assert_eq!(
            "".tokenize(vec!["->", "<-", "{", "}"])
                .map(Result::unwrap)
                .collect::<Vec<&str>>(),
            Vec::<&str>::new()
        );
    }

    #[test]
    fn test_tokenizer_4() {
        assert_eq!(
            "{inst1 -> inst2 -> {inst4 <- inst3} -"
                .tokenize(["->", "<-", "{", "}"])
                .collect::<Vec<Result<&str, usize>>>(),
            vec![
                Ok("{"),
                Ok("inst1"),
                Ok("->"),
                Ok("inst2"),
                Ok("->"),
                Ok("{"),
                Ok("inst4"),
                Ok("<-"),
                Ok("inst3"),
                Ok("}"),
                Err("{inst1 -> inst2 -> {inst4 <- inst3} -".len())
            ]
        );
    }

    #[test]
    fn test_tokenizer_5() {
        assert_eq!(
            "{inst1 -> inst2 -> {inst4 < inst3}"
                .tokenize(["->", "<-", "{", "}"])
                .collect::<Vec<Result<&str, usize>>>(),
            vec![
                Ok("{"),
                Ok("inst1"),
                Ok("->"),
                Ok("inst2"),
                Ok("->"),
                Ok("{"),
                Ok("inst4"),
                Err("{inst1 -> inst2 -> {inst4 <".len())
            ]
        );
    }

    #[test]
    fn test_tokenizer_6() {
        assert_eq!(
            "ab(cd(ef),gh)"
                .tokenize([":-", "[", "]", "(", ")", ",", "."])
                .map(Result::unwrap)
                .collect::<Vec<&str>>(),
            vec!["ab", "(", "cd", "(", "ef", ")", ",", "gh", ")"]
        );
    }

    #[test]
    fn test_tokenizer_7() {
        assert_eq!(
            "a_b*a_c("
                .tokenize([":-", "[", "]", "(", ")", ",", "."])
                .collect::<Vec<Result<&str, usize>>>(),
            vec![Ok("a_b"), Err("a_b".len())]
        );
    }

    #[test]
    fn test_tokenizer_8() {
        assert_eq!(
            "ab(c_d(e_f),g_h)))("
                .tokenize([":-", "[", "]", "(", ")", ",", "."])
                .collect::<Vec<Result<&str, usize>>>(),
            vec![
                Ok("ab"),
                Ok("("),
                Ok("c_d"),
                Ok("("),
                Ok("e_f"),
                Ok(")"),
                Ok(","),
                Ok("g_h"),
                Ok(")"),
                Ok(")"),
                Ok(")"),
                Ok("(")
            ]
        );
    }

    #[test]
    fn test_tokenizer_9() {
        assert_eq!(
            "[2]a:-b,c.\n"
                .tokenize([":-", "[", "]", "(", ")", ",", "."])
                .collect::<Vec<Result<&str, usize>>>(),
            vec![
                Ok("["),
                Ok("2"),
                Ok("]"),
                Ok("a"),
                Ok(":-"),
                Ok("b"),
                Ok(","),
                Ok("c"),
                Ok(".")
            ]
        );
    }

    #[test]
    fn test_tokenizer_10() {
        assert_eq!(
            "f(a ,b ,X)"
                .tokenize([":-", "[", "]", "(", ")", ",", "."])
                .map(Result::unwrap)
                .collect::<Vec<&str>>(),
            vec!["f", "(", "a", ",", "b", ",", "X", ")"]
        );
    }

    #[test]
    fn test_tokenizer_11() {
        assert_eq!(
            r#"(*"3" -> int -> *"i".write)"#
                .tokenize([
                    "->", "<-", "(", ")", "{", "=", ",", "}", "[", "|", "]", "*", ".",
                ])
                .map(Result::unwrap)
                .collect::<Vec<&str>>(),
            vec![
                "(", "*", r#""3""#, "->", "int", "->", "*", r#""i""#, ".", "write", ")"
            ]
        );
    }
}
