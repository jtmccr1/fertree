// Needed by pest
use pest_consume::{match_nodes, Error, Parser};
use std::collections::HashMap;
use crate::tree::fixed_tree::FixedNode;
use crate::tree::AnnotationValue;




#[derive(Parser)]
#[grammar = "./io/parser/newick.pest"]
pub struct NewickParser;

type Result<T> = std::result::Result<T, Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, ()>;


#[pest_consume::parser]
impl NewickParser {
    fn branchlength(input: Node) -> Result<f64> {
        input.as_str()
            .parse::<f64>()
            // `input.error` links the error to the location in the input file where it occurred.
            .map_err(|e| input.error(e))
    }
    fn length(input: Node) -> Result<f64> {
        Ok(match_nodes!(input.into_children();
            [branchlength(n)] =>n
        ))
    }
    fn name(input: Node) -> Result<String> {
        let name = input.as_str();
        Ok(name.to_string())
    }
    // fn label(input: Node) -> Result<String> {
    //     let name = input.as_str();
    //     Ok(name.to_string())
    // }
    fn leaf(input: Node) -> Result<FixedNode> {
        let mut tip = FixedNode::new();
        let name = input.as_str();
        tip.taxon = Some(name.to_string());
        Ok(tip)
    }
    fn branch(input: Node) -> Result<FixedNode> {
        Ok(match_nodes!(input.into_children();
            [subtree(mut n),node_annotation(a),length(l)]=>{
            n.length=Some(l);
            n.annotations=Some(a);
            n},
            [subtree(mut n),node_annotation(a)]=>{
            n.annotations=Some(a);
            n.length=Some(0.0);
            warn!("branchlength 0 inserted for branch without length");
            n
            },
            [subtree(mut n),length(l)]=>{n.length=Some(l);n},
            [subtree(mut n)]=>{
            n.length=Some(0.0);
            warn!("branchlength 0 inserted for branch without length");
            n
            }
        ))
    }
    fn annotation(input:Node)-> Result<(String,AnnotationValue)>{
        Ok(match_nodes!(input.into_children();
            [key(k),value(v)]=>(k,v),
        ))
    }
    fn annotation_set(input:Node) -> Result<Vec<(String,AnnotationValue)>>{
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

    fn node_annotation(input:Node) -> Result<HashMap<String, AnnotationValue>> {
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

    fn key(input:Node)-> Result<String>{
        let name = input.as_str();
        Ok(name.to_string())
    }

    fn value(input:Node)->Result<AnnotationValue>{
        Ok(match_nodes!(input.into_children();
            [continuous(n)]=>n,
            [discrete(n)]=>n,
            [set(n)]=>n
        ))
    }
    fn one_entry(input:Node)->Result<AnnotationValue>{
        Ok(match_nodes!(input.into_children();
            [continuous(n)]=>n,
            [discrete(n)]=>n
        ))
    }
    fn continuous(input: Node) -> Result<AnnotationValue> {
       let x = input.as_str()
        .parse::<f64>()
        // `input.error` links the error to the location in the input file where it occurred.
        .map_err(|e| input.error(e));

        Ok(AnnotationValue::Continuous(x.unwrap()))
    }
    fn discrete(input:Node) ->Result<AnnotationValue>{
        let name = input.as_str().to_string();
        Ok(AnnotationValue::Discrete(name))
    }
    fn set(input:Node)-> Result<AnnotationValue>{
        let set = match_nodes!(input.into_children();
            [one_entry(n)..]=>n.collect(),
        );
        Ok(AnnotationValue::Set(set))
    }

    fn branchset(input: Node) -> Result<Vec<FixedNode>> {
        let mut children: Vec<FixedNode> = vec![];
        Ok(match_nodes!(input.into_children();
            [branch(child)]=>{
            children.push(child);
            children
            },
            [branch(child),branchset(siblings)]=>{
                children.push(child);
                for sibling in siblings{
                    children.push(sibling);
                }
                children
            }
        ))
    }
    //returns a node with name and children
    fn internal(input: Node) -> Result<FixedNode> {
        let mut internal = FixedNode::new();
        Ok(match_nodes!(input.into_children();
        [branchset(children)]=>{
            for child in children{
                internal.children.push(Box::new(child))
            };
            internal
           },
          [branchset(children),name(n)]=>{
             for child in children{
                internal.children.push(Box::new(child))
            };
            internal.label=Some(n);
            internal
          }
        ))
    }

    //Just pass the leaf or internal node back to the parent
    fn subtree(input: Node) -> Result<FixedNode> {
        Ok(match_nodes!(input.into_children();
            [leaf(tip)]=>tip,
            [internal(node)]=>node
        ))
    }

    fn tree(input: Node) -> Result<FixedNode> {
        Ok(match_nodes!(input.into_children();
            [subtree(root)] =>{root},
            [branch(root)]=>{root}
            ))
    }
}

impl NewickParser {
    pub fn parse_tree(str: &str) -> Result<FixedNode> {
        let start = std::time::Instant::now();
        let inputs = NewickParser::parse(Rule::tree, str)?;
// There should be a single root node in the parsed tree
        let input = inputs.single()?;
        let root = NewickParser::tree(input);
        trace!("Tree parsed in {} milli seconds ",start.elapsed().as_millis());
        root
    }
}

#[cfg(test)]
mod tests {
    use crate::io::parser::newick_parser::NewickParser;

    #[test]
    fn it_works() {
        let root = NewickParser::parse_tree("(a:1,b:4)l:5;").unwrap();
        assert_eq!(root.label.unwrap(), "l");
        let mut names = vec![];
        for child in root.children.iter(){
            if let Some(t)=&child.taxon{
                names.push(t)
            }
        }
        assert_eq!(names,vec!["a","b"]);

        let mut bl = vec![];
        if let Some(l) = root.length{
            bl.push(l);
        }
        for child in root.children.iter(){
            if let Some(t)=child.length{
                bl.push(t)
            }
        }
        assert_eq!(bl,vec![5.0,1.0,4.0]);
    }



    #[test]
    fn scientific(){
        let root = NewickParser::parse_tree("(a:1E1,b:+2e-5)l:5e-1;").unwrap();
        let mut bl = vec![];
        if let Some(l) = root.length{
            bl.push(l);
        }
        for child in root.children.iter(){
            if let Some(t)=child.length{
                bl.push(t)
            }
        }
        assert_eq!(bl,vec![0.5,10.0,0.00002]);
    }


    #[test]
    fn quoted() {
        NewickParser::parse_tree("('234] ','here a *');");
    }

    #[test]
    fn annotation(){
        NewickParser::parse_tree("(a[&test=ok],b:1);");
    }

    #[test]
    fn whitespace(){
        NewickParser::parse_tree("  (a[&test=ok],b:1);\t");
    }

    #[test]
    fn should_error() {
        let out = NewickParser::parse_tree("('234] ','here a *')");
                assert_eq!(true, out.is_err())
    }

    #[test]
    fn should_error_again() {
        let out = NewickParser::parse_tree("(a,b));");
        assert_eq!(true, out.is_err())
    }

}