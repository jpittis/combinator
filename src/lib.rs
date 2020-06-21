/// A quick and dirty parser combinator library.
///
/// Thanks to James Coglan for the inspiration:
/// https://blog.jcoglan.com/2017/07/06/introduction-to-parser-combinators/
///
use regex::Regex;

#[derive(Debug, PartialEq, Clone)]
pub struct State {
    s: String,
    offset: usize,
}

impl State {
    pub fn new(s: String, offset: usize) -> Self {
        Self { s, offset }
    }

    fn peek(&self, n: usize) -> String {
        if self.offset + n > self.s.len() {
            String::new()
        } else {
            self.s[self.offset..self.offset + n].to_string()
        }
    }

    fn read(&self, n: usize) -> Self {
        Self::new(self.s.to_string(), self.offset + n)
    }
}

pub trait Parse {
    fn parse(&self, state: &State) -> Option<(Vec<String>, State)>;
}

pub struct Lit {
    lit: String,
}

impl Lit {
    pub fn new(lit: String) -> Self {
        Self { lit }
    }
}

impl Parse for Lit {
    fn parse(&self, state: &State) -> Option<(Vec<String>, State)> {
        let peeked = state.peek(self.lit.len());
        if peeked == self.lit {
            Some((vec![peeked], state.read(self.lit.len())))
        } else {
            None
        }
    }
}

pub struct Char {
    re: Regex,
}

impl Char {
    pub fn new(re: &str) -> Result<Self, regex::Error> {
        Ok(Self {
            re: Regex::new(&format!("[{}]", re))?,
        })
    }
}

impl Parse for Char {
    fn parse(&self, state: &State) -> Option<(Vec<String>, State)> {
        let peeked = state.peek(1);
        if self.re.is_match(&peeked) {
            Some((vec![peeked], state.read(1)))
        } else {
            None
        }
    }
}

pub struct Seq {
    seq: Vec<Box<dyn Parse>>,
}

impl Seq {
    pub fn new(seq: Vec<Box<dyn Parse>>) -> Self {
        Self { seq }
    }
}

impl Parse for Seq {
    fn parse(&self, state: &State) -> Option<(Vec<String>, State)> {
        let mut current = state.clone();
        let mut results = Vec::new();
        for parse in self.seq.iter() {
            let (res, state_next) = parse.parse(&current)?;
            results.push(res);
            current = state_next;
        }
        Some((results.concat(), current))
    }
}

pub struct Rep {
    parse: Box<dyn Parse>,
    min: usize,
}

impl Rep {
    pub fn new(parse: Box<dyn Parse>, min: usize) -> Self {
        Self { parse, min }
    }
}

impl Parse for Rep {
    fn parse(&self, state: &State) -> Option<(Vec<String>, State)> {
        let mut current = state.clone();
        let mut results = Vec::new();
        loop {
            match self.parse.parse(&current) {
                Some((res, state_next)) => {
                    results.push(res);
                    current = state_next;
                }
                None => {
                    if results.len() >= self.min {
                        return Some((results.concat(), current));
                    } else {
                        return None;
                    }
                }
            }
        }
    }
}

pub struct Alt {
    choices: Vec<Box<dyn Parse>>,
}

impl Alt {
    pub fn new(choices: Vec<Box<dyn Parse>>) -> Self {
        Self { choices }
    }
}

impl Parse for Alt {
    fn parse(&self, state: &State) -> Option<(Vec<String>, State)> {
        for parse in self.choices.iter() {
            let parsed = parse.parse(state);
            if parsed.is_some() {
                return parsed;
            }
        }
        None
    }
}

// TODO: A node mapping closure parameter.
// TODO: Some nicer high level wrappers.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal() {
        let hello = Lit::new("hello".to_string());
        assert_eq!(
            hello.parse(&State::new("hellofoobar".to_string(), 0)),
            Some((
                vec!["hello".to_string()],
                State::new("hellofoobar".to_string(), 5)
            ))
        );
        assert_eq!(hello.parse(&State::new("hellfoobar".to_string(), 0)), None,);
    }

    #[test]
    fn test_char() {
        let digits = Char::new("0-9").unwrap();
        assert_eq!(
            digits.parse(&State::new("7a".to_string(), 0)),
            Some((vec!["7".to_string()], State::new("7a".to_string(), 1)))
        );
        assert_eq!(digits.parse(&State::new("a".to_string(), 0)), None);
    }

    #[test]
    fn test_seq() {
        let cookie = Seq::new(vec![
            Box::new(Char::new("0-9").unwrap()),
            Box::new(Char::new(" ").unwrap()),
            Box::new(Lit::new("cookie".to_string())),
        ]);
        assert_eq!(
            cookie.parse(&State::new("5 cookie".to_string(), 0)),
            Some((
                vec!["5".to_string(), " ".to_string(), "cookie".to_string()],
                State::new("5 cookie".to_string(), 8)
            ))
        );
        assert_eq!(cookie.parse(&State::new("5xcookie".to_string(), 0)), None);
    }

    #[test]
    fn test_rep() {
        let two_or_more = Rep::new(Box::new(Char::new("g").unwrap()), 2);
        assert_eq!(
            two_or_more.parse(&State::new("gg".to_string(), 0)),
            Some((
                vec!["g".to_string(), "g".to_string()],
                State::new("gg".to_string(), 2)
            )),
        );
        assert_eq!(
            two_or_more.parse(&State::new("ggg".to_string(), 0)),
            Some((
                vec!["g".to_string(), "g".to_string(), "g".to_string()],
                State::new("ggg".to_string(), 3)
            )),
        );
        assert_eq!(two_or_more.parse(&State::new("g".to_string(), 0)), None);
    }

    #[test]
    fn test_alt() {
        let either = Alt::new(vec![
            Box::new(Lit::new("foo".to_string())),
            Box::new(Lit::new("bar".to_string())),
        ]);
        assert_eq!(
            either.parse(&State::new("foo".to_string(), 0)),
            Some((vec!["foo".to_string()], State::new("foo".to_string(), 3))),
        );
        assert_eq!(
            either.parse(&State::new("bar".to_string(), 0)),
            Some((vec!["bar".to_string()], State::new("bar".to_string(), 3))),
        );
        assert_eq!(either.parse(&State::new("lol".to_string(), 0)), None);
    }
}
