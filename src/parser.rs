use crate::pest::{
    error,
    iterators::{Pair, Pairs},
    Parser,
};
use std::fmt;

#[derive(Parser)]
#[grammar = "sdn.pest"]
struct SdnParser;

pub type PestError<T> = error::Error<T>;

pub fn parse(data: &str) -> Result<Vec<Data>, PestError<Rule>> {
    match SdnParser::parse(Rule::root, data) {
        Ok(mut parsed) => {
            let mut data: Vec<Data> = parsed
                .next()
                .unwrap()
                .into_inner()
                .map(Data::from)
                .collect();
            data.pop(); // remove DataPre::Nil resulted from EOI
            Ok(data)
        }
        Err(e) => Err(e),
    }
}

#[derive(Clone)]
pub enum Data {
    List(Vec<Data>),
    Int(i64),
    Float(f64),
    Str(String),
    Symbol(String),
    Keyword(String),
    Nil,
} // TODO: use &str instead of Strings

impl From<Pair<'_, Rule>> for Data {
    fn from(arg: Pair<Rule>) -> Data {
        match arg.as_rule() {
            Rule::expr => {
                let mut inner = arg.into_inner();
                let inner_str = inner.as_str();
                match inner.clone().next().unwrap().as_rule() {
                    Rule::list => {
                        Data::List(inner.next().unwrap().into_inner().map(Data::from).collect())
                    }
                    Rule::int => Data::Int(inner_str.parse().unwrap()),
                    Rule::float => Data::Float(inner_str.parse().unwrap()),
                    Rule::string => Data::Str(Data::parse_string(inner)),
                    Rule::symbol => Data::Symbol(inner_str.to_string()),
                    Rule::keyword => Data::Keyword(inner_str.get(1..).unwrap_or("").to_string()),
                    other => unreachable!("inside expr: {:?}", other), // TODO: maybe make a better check for this
                }
            }
            Rule::EOI => Data::Nil,
            other => unreachable!("{:?}", other), // TODO: maybe make a better check for this
        }
    }
}

impl fmt::Debug for Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.repr())
    }
}

impl Data {
    fn parse_string(string_data: Pairs<Rule>) -> String {
        let chars = string_data
            .clone() // string
            .next()
            .unwrap()
            .into_inner() // string_inner
            .next()
            .unwrap()
            .into_inner(); // char*
        let mut final_string = String::new();

        for ch in chars {
            let ch_data = ch.into_inner().next().unwrap();
            let ch_rule = ch_data.clone().as_rule();
            let ch_str = ch_data.clone().as_str();

            match ch_rule {
                Rule::char_normal => final_string.push_str(ch_str),
                Rule::char_escape_code => final_string.push(match ch_str {
                    "\\n" => '\n',
                    "\\t" => '\t',
                    "\\\"" => '\"',
                    "\\\\" => '\\',
                    _ => unreachable!("this escape code should not be here: '{}'", ch_str),
                }),
                _ => unreachable!("{:?}", ch_rule),
            }
        }

        final_string
    }

    pub fn repr(&self) -> String {
        match self {
            Data::Symbol(s) => s.clone(),
            Data::Str(s) => format!("{:?}", s),
            Data::Int(i) => format!("{}", i),
            Data::Float(f) => format!("{}", f),
            Data::List(v) => format!(
                "({})",
                v.iter().map(Data::repr).collect::<Vec<String>>().join(" ")
            ),
            Data::Keyword(k) => format!(":{}", k),
            Data::Nil => "nil".into(),
        }
    }
}
