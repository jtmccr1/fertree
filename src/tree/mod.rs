use serde::Deserialize;
use std::fmt;

//TODO unify the trees with traits so they can be used interchangeably where applicable.
pub mod fixed_tree;
pub mod mutable_tree;

#[serde(untagged)]
#[derive(Debug, Clone, Deserialize,PartialEq)]
pub enum AnnotationValue {
    Discrete(String),
    Continuous(f64),
    Boolean(bool),
    Set(Vec<AnnotationValue>),
}

impl fmt::Display for AnnotationValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AnnotationValue::Discrete(string) => write!(f, "{}", string),
            AnnotationValue::Continuous(f64) => write!(f, "{}", f64.to_string()),
            AnnotationValue::Boolean(b)=>write!(f, "{}", b.to_string()),
            AnnotationValue::Set(s) => {
                let s = s
                    .iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<String>>()
                    .join(",");
                write!(f, "{{ {} }}", s)
            }
        }
    }
}
