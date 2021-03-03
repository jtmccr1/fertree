use pest_consume::{match_nodes, Error, Parser};
use crate::tree::AnnotationValue;
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
        let name = input.as_str();
        Ok(name.to_string())
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
            [discrete(n)]=>n
        ))
    }
    fn continuous(input: Node) -> PestResult<AnnotationValue> {
        let x = input
            .as_str()
            .parse::<f64>()
            // `input.error` links the error to the location in the input file where it occurred.
            .map_err(|e| input.error(e));

        Ok(AnnotationValue::Continuous(x.unwrap()))
    }
    fn discrete(input: Node) -> PestResult<AnnotationValue> {
        let name = input.as_str().to_string();
        Ok(AnnotationValue::Discrete(name))
    }
    fn set(input: Node) -> PestResult<AnnotationValue> {
        let set = match_nodes!(input.into_children();
            [one_entry(n)..]=>n.collect(),
        );
        Ok(AnnotationValue::Set(set))
    }
}
impl AnnotationParser{
    pub(crate) fn parse_annotation(s:&str) ->PestResult<HashMap<String,AnnotationValue>>{
        let inputs = AnnotationParser::parse(Rule::node_annotation, s)?;
        // There should be a single root node in the parsed tree
        let input = inputs.single()?;
        AnnotationParser::node_annotation(input)
    }
}