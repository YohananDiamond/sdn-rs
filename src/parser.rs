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
type KeywordMap = HashMap<String, Data>;

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

impl From<Pair<'_, Rule>> for Data {
    fn from(arg: Pair<Rule>) -> Data {
        match arg.as_rule() {
            Rule::expr => {
                let mut inner = arg.into_inner();
                let inner_str = inner.as_str();
                match inner.clone().next().unwrap().as_rule() {
                    Rule::list => {
                        let (vec, map) = Data::parse_list(
                            inner.next().unwrap().into_inner().map(Data::from).collect(),
                        ).unwrap(); // TODO: remove this unwrap
                        Data::List {
                            args: vec,
                            kwargs: map,
                        }
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

    /**
     * Returns Ok(content) if successful, or Err(error_str) if failed
     */
    fn parse_list(list: Vec<Data>) -> Result<(Vec<Data>, KeywordMap), String> {
        let mut current_kw: Option<&str> = None;
        let mut position: usize = 0;
        let mut vec = Vec::new();
        let mut map = HashMap::new();

        while let Some(element) = list.get(position) {
            if let Data::Keyword(keyword) = element {
                if let Some(keyword_prev) = current_kw {
                    return Err(format!(
                        "keyword :{} without value in list",
                        keyword_prev
                    ));
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
                "({:?} {:?})",
                args, // v.iter().map(Data::repr).collect::<Vec<String>>().join(" "),
                kwargs,
            ),
            Data::Keyword(k) => format!(":{}", k),
            Data::Nil => "nil".into(),
        }
    }
}
