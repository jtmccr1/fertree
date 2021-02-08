use serde::{ Deserialize};
use std::fmt;




pub mod mutable_tree;
pub mod fixed_tree;

#[derive(Debug,Clone,Deserialize)]
#[serde(untagged)]

pub enum AnnotationValue{
    Discrete(String),
    Continuous(f64),
    Set(Vec<AnnotationValue>)
}

impl fmt::Display for AnnotationValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AnnotationValue::Discrete(string)=>write!(f,"{}",string),
            AnnotationValue:: Continuous(f64)=>write!(f,"{}",f64.to_string()),
            AnnotationValue::Set(s)=>{
                let s = s.iter().map(|a| a.to_string())
                    .collect::<Vec<String>>()
                    .join(",");
                write!(f,"{{ {} }}",s)
            }
        }
    }
}