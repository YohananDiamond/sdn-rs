use crate::pest;
use crate::pest::{
    iterators::{Pair, Pairs},
    Parser,
};
use std::collections::hash_map::{Entry, HashMap};
use std::fmt;

#[derive(Parser)]
#[grammar = "sdn.pest"]
struct SdnParser;

pub type PestError<T> = pest::error::Error<T>;
pub type KeywordMap = HashMap<String, Data>;

pub enum ParserResult {
    Success(Vec<Data>),
    PestError(PestError<Rule>),
    StringError(String),
}

pub fn parse(data: &str) -> ParserResult {
    match SdnParser::parse(Rule::root, data) {
        Ok(mut parsed) => {
            let mut parsed_iter = parsed.next().unwrap().into_inner();
            let mut data = Vec::<Data>::new();

            for pair in parsed_iter {
                match Data::parse_pair(pair) {
                    Ok(o) => data.push(o),
                    Err(e) => return ParserResult::StringError(e),
                }
            }

            data.pop(); // remove DataPre::Nil resulted from EOI
            ParserResult::Success(data)
        }
        Err(e) => ParserResult::PestError(e),
    }
}

#[derive(Clone)]
pub enum Data {
    List {
        args: Vec<Data>,
        kwargs: HashMap<String, Data>,
    },
    Int(i64),
    Float(f64),
    Str(String),
    Symbol(String),
    Keyword(String),
    Nil,
} // TODO: use &str instead of Strings

impl fmt::Debug for Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.repr())
    }
}

impl Data {
    /// Parses a pair and converts it into data.
    pub fn parse_pair(arg: Pair<'_, Rule>) -> Result<Data, String> {
        match arg.as_rule() {
            Rule::expr => {
                let mut inner = arg.into_inner();
                let inner_str = inner.as_str();
                match inner.clone().next().unwrap().as_rule() {
                    Rule::list => {
                        let parse: Vec<Data> = inner
                            .next()
                            .unwrap()
                            .into_inner()
                            .map(Data::parse_pair)
                            .collect::<Result<Vec<Data>, String>>()?;

                        match Data::parse_list(parse) {
                            Ok((vec, map)) => Ok(Data::List {
                                args: vec,
                                kwargs: map,
                            }),
                            Err(e) => return Err(e),
                        }
                    }
                    Rule::int => Ok(Data::Int(inner_str.parse().unwrap())),
                    Rule::float => Ok(Data::Float(inner_str.parse().unwrap())),
                    Rule::string => Ok(Data::Str(Data::parse_string(inner))),
                    Rule::symbol => Ok(Data::Symbol(inner_str.to_string())),
                    Rule::keyword => {
                        Ok(Data::Keyword(inner_str.get(1..).unwrap_or("").to_string()))
                    }
                    other => {
                        return Err(format!(
                            "supposedly unreachable rule inside expr: {:?}",
                            other
                        ))
                    }
                }
            }
            Rule::EOI => Ok(Data::Nil),
            other => return Err(format!("supposedly unreachable rule: {:?}", other)),
        }
    }

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

    /// Returns Ok(content) if successful, or Err(error_str) if failed
    fn parse_list(list: Vec<Data>) -> Result<(Vec<Data>, KeywordMap), String> {
        let mut current_kw: Option<&str> = None;
        let mut position: usize = 0;
        let mut vec = Vec::new();
        let mut map = HashMap::new();

        while let Some(element) = list.get(position) {
            if let Data::Keyword(keyword) = element {
                if let Some(keyword_prev) = current_kw {
                    return Err(format!("keyword :{} without value in list", keyword_prev));
                }
                current_kw = Some(keyword.as_str());
            } else {
                if let Some(keyword) = current_kw {
                    match map.entry(keyword.to_string()) {
                        Entry::Occupied(_) => {
                            return Err(format!(
                                "keyword :{} provided more than once in same list",
                                keyword
                            ))
                        }
                        Entry::Vacant(e) => {
                            e.insert(element.clone());
                            current_kw = None;
                        }
                    }
                } else {
                    vec.push(element.clone());
                }
            }

            position += 1;
        }

        match current_kw {
            Some(kw) => Err(format!("keyword :{} without value in list", kw)),
            None => Ok((vec, map)),
        }
    }

    pub fn repr(&self) -> String {
        match self {
            Data::Symbol(s) => s.clone(),
            Data::Str(s) => format!("{:?}", s),
            Data::Int(i) => format!("{}", i),
            Data::Float(f) => format!("{}", f),
            Data::List { args, kwargs } => format!(
                "({})",
                vec![
                    args.iter()
                        .map(Data::repr)
                        .collect::<Vec<String>>()
                        .join(" "),
                    kwargs
                        .keys()
                        .map(|key| format!(":{} {}", key, kwargs[key].repr()))
                        .collect::<Vec<String>>()
                        .join(" "),
                ].iter().filter(|s| s.len() != 0).map(|s| s.as_str()).collect::<Vec<&str>>().join(" "),
            ),
            Data::Keyword(k) => format!(":{}", k),
            Data::Nil => "nil".into(),
        }
    }
}
