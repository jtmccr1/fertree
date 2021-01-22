mod tree;
// mod newickParser;

use structopt::StructOpt;
use std::{path, io};
use std::io::{BufReader, BufRead, Read};
use std::fs::File;
// use tree::NewickParser;
use crate::tree::{TreeNode, Tree, FixedNode};

extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest_consume::{match_nodes, Error, Parser};

#[derive(Parser)]
#[grammar = "newick.pest"]
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


#[derive(Debug, StructOpt)]
#[structopt(about = "command line tools for processing phylogenetic trees in rust")]
enum Fertree {
    Stats {
        #[structopt(flatten)]
        common: Common,
        #[structopt(subcommand)]
        cmd: Option<StatsSubCommands>,
    }
}

#[derive(Debug, StructOpt)]
enum StatsSubCommands {
    Tips,
}

#[derive(Debug, StructOpt)]
struct Common {
    #[structopt(short, long, parse(from_os_str), help = "input tree file")]
    infile: Option<path::PathBuf>,
    #[structopt(short, long, parse(from_os_str), help = "output tree file")]
    outfile: Option<path::PathBuf>,
    #[structopt(short, long)]
    debug: bool,
    #[structopt(short, long)]
    release: bool,
}

fn main() {
    // let args = Fertree::from_args();
    // println!("{:?}",args);
    // match Fertree::from_args() {
    //     Fertree::Stats { common, cmd } => {
    //         match cmd {
    //             Some(StatsSubCommands::Tips) =>{
    //                 println!("{:?}",common);
    //                 parse_input(common.infile).expect("error reading file");
    //                 println!("This is us getting the tips!")
    //             },
    //             None =>{
    //                 parse_input(common.infile).expect("error reading file");
    //                 println!("This would be the number of tips and such in the file/stdin");
    //             }
    //         }
    //     }
    // }

    let mut tree = Tree::new();
    let e = tree.add_node(TreeNode::new(Some("a".to_string()), None));
    let d = tree.add_node(TreeNode::new(Some("a".to_string()), Some(e)));
    let c = tree.add_node(TreeNode::new(Some("a".to_string()), Some(e)));
    let a = tree.add_node(TreeNode::new(Some("a".to_string()), Some(d)));
    let b = tree.add_node(TreeNode::new(Some("a".to_string()), Some(d)));

    tree.set_root(Some(e));
    let mut preorder = tree.iter();

    preorder = tree.iter();
    while let Some(i) = preorder.next(&tree) {
        let node = tree.node_at(i).expect("node to exist at given index");
        println!("{}", node.taxon.as_ref().unwrap())
    }

    let inputs = NewickParser::parse(Rule::tree, "((a:1,b:1),c:1);").unwrap();
    // There should be a single root node in the parsed tree
    println!("{}", inputs);
    let input = inputs.single().unwrap();
    // Consume the `Node` recursively into the final value
    let l = NewickParser::tree(input);
    println!("{:?}", l.unwrap());
}
