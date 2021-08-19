use crate::tree::AnnotationValue;
use crate::tree::MarkovJump;
use pest_consume::{match_nodes, Error, Parser};
use std::collections::HashMap;

type PestResult<T> = std::result::Result<T, Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, ()>;

#[derive(Parser)]
#[grammar = "./io/parser/tree_annotation.pest"]
pub struct AnnotationParser;

#[pest_consume::parser]
impl AnnotationParser {
    fn annotation(input: Node) -> PestResult<(String, AnnotationValue)> {
        Ok(match_nodes!(input.into_children();
            [key(k),value(v)]=>(k,v),
            [key(k)]=>(k,AnnotationValue::Boolean(true))
        ))
    }
    fn annotation_set(input: Node) -> PestResult<Vec<(String, AnnotationValue)>> {
        let mut annotations = vec![];
        Ok(match_nodes!(input.into_children();
            [annotation(a)]=>{
                annotations.push(a);
                annotations
            },
            [annotation(a),annotation_set(others)]=>{
                annotations.push(a);
                for other in others{
                    annotations.push(other);
            }
            annotations
        }
        ))
    }

    fn node_annotation(input: Node) -> PestResult<HashMap<String, AnnotationValue>> {
        Ok(match_nodes!(input.into_children();
            [annotation_set(annotations)]=>{
                let mut annotation_map = HashMap::new();
                for (key,value) in annotations{
                    annotation_map.insert(key,value);
                }
                annotation_map
            }
        ))
    }

    fn key(input: Node) -> PestResult<String> {
        Ok(match_nodes!(input.into_children();
          [unquoted_key(n)]=>n,
          [quoted_name(n)]=>n,
        ))
    }
    fn unquoted_key(input: Node) -> PestResult<String> {
        let name = input.as_str();
        Ok(name.to_string())
    }

    fn unquoted_name(input: Node) -> PestResult<String> {
        let name = input.as_str();
        Ok(name.to_string())
    }
    fn quoted_name(input: Node) -> PestResult<String> {
        Ok(match_nodes!(input.into_children();
          [single_inner(n)]=>n,
          [double_inner(n)]=>n,
        ))
    }
    fn single_inner(input: Node) -> PestResult<String> {
        let name = input.as_str();
        Ok(name.to_string())
    }
    fn double_inner(input: Node) -> PestResult<String> {
        let name = input.as_str();
        Ok(name.to_string())
    }
    fn empty_string(input: Node) -> PestResult<String> {
        Ok(String::new())
    }
    fn value(input: Node) -> PestResult<AnnotationValue> {
        Ok(match_nodes!(input.into_children();
            [continuous(n)]=>n,
            [discrete(n)]=>n,
            [set(n)]=>n
        ))
    }
    fn one_entry(input: Node) -> PestResult<AnnotationValue> {
        Ok(match_nodes!(input.into_children();
            [continuous(n)]=>n,
            [discrete(n)]=>n,
            [markovjump(n)]=>n
        ))
    }
    fn continuous(input: Node) -> PestResult<AnnotationValue> {
        let x = input
            .as_str()
            .parse::<f64>()
            // `input.error` links the error to the location in the input file where it occurred.
            .map_err(|e| input.error(e));

        if let Ok(float) = x {
            Ok(AnnotationValue::Continuous(float))
        } else {
            warn!("found numbers in annotation but failed to parse {} falling back to discrete annotation",input.as_str());
            Ok(AnnotationValue::Discrete(input.as_str().parse().unwrap()))
        }
    }
    fn discrete(input: Node) -> PestResult<AnnotationValue> {
        Ok(match_nodes!(input.into_children();
          [unquoted_name(n)]=>AnnotationValue::Discrete(n),
          [quoted_name(n)]=>AnnotationValue::Discrete(n),
          [empty_string(n)]=>AnnotationValue::Discrete(n)
        ))
    }
    fn set(input: Node) -> PestResult<AnnotationValue> {
        let set = match_nodes!(input.into_children();
            [one_entry(n)..]=>n.collect(),
        );
        Ok(AnnotationValue::Set(set))
    }

    fn markovjump(input: Node) -> PestResult<AnnotationValue> {
        Ok(match_nodes!(input.into_children();
          [continuous(t),discrete(s),discrete(d)]=>{
                  let mut mj_time = None;
                  let mut mj_source = None;
                  let mut mj_dest = None;
                  if let  AnnotationValue::Continuous(time) =t {
                      mj_time = Some(time);
                  }
                  if let AnnotationValue::Discrete(source) = s  {
                      mj_source = Some(source.clone());
                  }
                   if let AnnotationValue::Discrete(dest) =d {
                      mj_dest = Some(dest.clone());
                  }
                  AnnotationValue::MarkovJump(MarkovJump{time:mj_time.unwrap(),source:mj_source.unwrap(),destination:mj_dest.unwrap()})
              }
        ))
    }
}

impl AnnotationParser {
    pub(crate) fn parse_annotation(s: &str) -> PestResult<HashMap<String, AnnotationValue>> {
        let inputs = AnnotationParser::parse(Rule::node_annotation, s)?;
        // There should be a single root node in the parsed tree
        let input = inputs.single()?;
        AnnotationParser::node_annotation(input)
    }
    pub fn parse_annotation_value(s: &str) -> PestResult<AnnotationValue> {
        let inputs = AnnotationParser::parse(Rule::value, s)?;
        // There should be a single root node in the parsed tree
        let input = inputs.single()?;
        AnnotationParser::value(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::MarkovJump;

    #[test]
    fn discrete() {
        let mut exp = HashMap::new();
        exp.insert(
            "location".to_owned(),
            AnnotationValue::Discrete("UK".to_owned()),
        );

        assert_eq!(
            AnnotationParser::parse_annotation("[&location=UK]").unwrap(),
            exp
        );
    }

    #[test]
    fn discrete_quotes() {
        let mut exp = HashMap::new();
        exp.insert(
            "location".to_owned(),
            AnnotationValue::Discrete("UK".to_owned()),
        );
        assert_eq!(
            AnnotationParser::parse_annotation("[&location=\"UK\"]").unwrap(),
            exp
        );
    }

    #[test]
    fn empty_quotes() {
        let mut exp = HashMap::new();
        exp.insert(
            "location".to_owned(),
            AnnotationValue::Discrete("".to_owned()),
        );
        assert_eq!(
            AnnotationParser::parse_annotation("[&location=\"\"]").unwrap(),
            exp
        );
    }

    #[test]
    fn quoted_key() {
        let mut exp = HashMap::new();
        exp.insert(
            "location".to_owned(),
            AnnotationValue::Discrete("UK".to_owned()),
        );
        assert_eq!(
            AnnotationParser::parse_annotation("[&'location'=UK]").unwrap(),
            exp
        );
    }

    #[test]
    fn just_rooted() {
        let mut exp = HashMap::new();
        exp.insert("R".to_owned(), AnnotationValue::Boolean(true));
        assert_eq!(AnnotationParser::parse_annotation("[&R]").unwrap(), exp);
    }

    #[test]
    fn multiple_commnet() {
        let mut exp = HashMap::new();
        exp.insert(
            "location[1]".to_owned(),
            AnnotationValue::Discrete("UK".to_owned()),
        );
        exp.insert("lat".to_owned(), AnnotationValue::Continuous(0.0));
        assert_eq!(
            AnnotationParser::parse_annotation("[&location[1]=UK,lat=0.0]").unwrap(),
            exp
        );
    }

    #[test]
    fn markov_jump() {
        let parsed = AnnotationParser::parse_annotation("[&location={{0.5,UK,US}}]").unwrap();
        let mut exp = HashMap::new();
        exp.insert(
            "location".to_owned(),
            AnnotationValue::Set(vec![AnnotationValue::MarkovJump(MarkovJump {
                time: 0.5,
                source: "UK".to_string(),
                destination: "US".to_string(),
            })]),
        );
        assert_eq!(parsed, exp);
    }
}
