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
    fn name(input:Node)->Result<String>{
        let name = input.as_str();
        Ok(name.to_string())
    }
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
        let mut children: Vec<FixedNode>=vec![];
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

    fn tree(input:Node)->Result<FixedNode>{

        Ok(match_nodes!(input.into_children();
            [subtree(root)] =>{root},
            [branch(root)]=>{root}
            ))
    }

}

impl NewickParser{
   pub fn parse_tree(str:&str)->Result<FixedNode>{
        let inputs = NewickParser::parse(Rule::tree, "((a:1,b:1),c:1);").unwrap();
// There should be a single root node in the parsed tree
        println!("{}", inputs);
        let input = inputs.single().unwrap();
// Consume the `Node` recursively into the final value
        let l = NewickParser::tree(input);
        println!("{:?}", l);

        l
    }
}
