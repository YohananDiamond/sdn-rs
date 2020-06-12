use crate::data::Data;

pub fn data_vec_for_file(data: Vec<Data>) -> String {
    data.iter().map(Data::repr).collect::<Vec<String>>().join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use Data::*;

    #[test]
    fn unnamed() {
        assert_eq!(
            data_vec_for_file(vec![
                List {
                    args: vec![Int(10), Int(20)],
                    kwargs: HashMap::new()
                },
                Symbol("hello"),
                List {
                    args: vec![Str("hmm".into()), Str("this is great".into())],
                    kwargs: HashMap::new(),
                },
            ])
            .as_str(),
            vec!["(10 20)", "hello", r#"("hmm" "this is great")"#].join("\n"),
        );
    }
}
