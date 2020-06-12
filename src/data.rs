use std::fmt;
use std::collections::hash_map::HashMap;

pub type KeywordMap<'a> = HashMap<String, Data<'a>>;

#[derive(Clone, PartialEq)]
pub enum Data<'a> {
    List {
        args: Vec<Data<'a>>,
        kwargs: KeywordMap<'a>,
    },
    Int(i64),
    Float(f64),
    Str(String),
    Symbol(&'a str),
    Keyword(&'a str),
    Nil,
}

impl Data<'_> {
    pub fn repr(&self) -> String {
        match self {
            Data::Symbol(s) => s.to_string(),
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
                ]
                .iter()
                .filter(|s| s.len() != 0)
                .map(|s| s.as_str())
                .collect::<Vec<&str>>()
                .join(" "),
            ),
            Data::Keyword(k) => format!(":{}", k),
            Data::Nil => "nil".into(),
        }
    }
}

impl<'a> fmt::Debug for Data<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.repr())
    }
}
