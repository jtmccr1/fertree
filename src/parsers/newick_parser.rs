use pest_consume::{match_nodes, Error, Parser};
use super::super::tree::fixed_tree::FixedNode;

#[derive(Parser)]
#[grammar = "./parsers/newick.pest"]
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
        let mut node: Option<FixedNode> = None;

        Ok(match_nodes!(input.into_children();
            [subtree(mut n),length(l)]=>{n.length=Some(l);n},
            [subtree(n)]=>n
        ))
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
        let inputs = NewickParser::parse(Rule::tree, str).unwrap();
// There should be a single root node in the parsed tree
        let input = inputs.single().unwrap();
// Consume the `Node` recursively into the final value
        NewickParser::tree(input)
    }
}


#[cfg(test)]
mod tests {
    use crate::parsers::newick_parser::NewickParser;
    use crate::tree::fixed_tree::FixedNode;

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
    fn no_labels(){
        let root = NewickParser::parse_tree("(:1,:4):5;").unwrap();

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
    fn quoted() {
        NewickParser::parse_tree("('234] ','here a *');");
    }

    #[test]
    #[should_panic]
    fn should_error() {
        NewickParser::parse_tree("('234] ','here a *')");
    }

    #[test]
    #[should_panic]
    fn should_error_again() {
        NewickParser::parse_tree("(a,b));");
    }

}