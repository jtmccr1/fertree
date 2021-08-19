use rebl::tree::mutable_tree::{MutableTree, TreeIndex};
use rebl::tree::AnnotationValue;
use std::collections::HashSet;
use structopt::StructOpt;

use rand::seq::SliceRandom;
use rebl::io::parser::tree_importer::TreeImporter;
use std::error::Error;
use std::io::Write;

#[derive(Debug, StructOpt)]
pub struct SharedOptions {
    #[structopt(
        short,
        long,
        help = "annotation key we are collapsing by. must be discrete"
    )]
    annotation: String,
    #[structopt(short, long, help = "annotation value we are collapsing by")]
    value: String,
}

#[derive(Debug, StructOpt)]
pub enum SubCommands {
    /// annotate tips with unique clade key based on annotation
    Label {
        #[structopt(
            short,
            long,
            help = "prefix for output annotation, if not provided defaults to 'annotation_value.' - Not implemented"
        )]
        prefix: Option<String>,
        #[structopt(
            short,
            long,
            help = "annotation key we are collapsing by. must be discrete"
        )]
        annotation: String,
        #[structopt(short, long, help = "annotation value we are interested in")]
        value: String,
        #[structopt(short, long, help = "annotate internal nodes as well")]
        internal: bool,
    },
    /// Collapse monophyletic clades
    Collapse {
        #[structopt(
            short,
            long,
            help = "annotation key we are collapsing by. must be discrete"
        )]
        annotation: String,
        #[structopt(short, long, help = "annotation value we are collapsing by")]
        value: String,
        #[structopt(short, long, help = "the minimum clade size", default_value = "1")]
        min_size: usize,
    },
}

//TODO set random seed.
pub fn run<R: std::io::Read, T: TreeImporter<R>>(
    mut trees: T,
    cmd: SubCommands,
) -> Result<(), Box<dyn Error>> {
    let stdout = std::io::stdout(); // get the global stdout entity
    let mut handle = stdout.lock(); // acquire a lock on it

    while trees.has_tree() {
        let mut tree = trees.read_next_tree()?;
        match cmd {
            SubCommands::Collapse {
                ref annotation,
                ref value,
                min_size,
            } => {
                let new_tree = collapse_uniform_clades(&mut tree, &annotation, &value, min_size);
                writeln!(handle, "{}", new_tree)?;
            }
            SubCommands::Label {
                ref annotation,
                ref value,
                ref prefix,
                ref internal,
            } => {
                annotate_uniform_clades(&mut tree, &annotation, &value, &prefix, &internal);
                writeln!(handle, "{}", tree)?;
            }
        }
    }
    Ok(())
}

pub fn collapse_uniform_clades(
    tree: &mut MutableTree,
    key: &str,
    value: &str,
    min_size: usize,
) -> MutableTree {
    tree.calc_node_heights();

    let mut taxa: HashSet<String> = tree
        .external_nodes
        .iter()
        .map(|node| tree.get_taxon(*node))
        .map(|n| String::from(n.unwrap()))
        .collect();

    let monophyletic_groups = get_monophyletic_groups(tree, tree.get_root().unwrap(), key, value);
    if monophyletic_groups.0 {
        warn!("The whole tree is a monophyletic clade!")
    }
    let mut removed = 0;
    for group in monophyletic_groups.1.iter() {
        //TODO only make this once
        let mut rng = &mut rand::thread_rng();

        for node in group.choose_multiple(&mut rng, group.len() - min_size) {
            let taxon = tree.get_taxon(*node).expect("This is not external node!");
            taxa.remove(taxon);
            removed += 1;
        }
    }
    info!("Removed : {} taxa", removed);
    MutableTree::from_tree(tree, &taxa)
}

fn annotate_uniform_clades(
    tree: &mut MutableTree,
    key: &str,
    value: &str,
    prefix: &Option<String>,
    internal: &bool,
) {
    let monophyletic_groups = get_monophyletic_groups(tree, tree.get_root().unwrap(), key, value);
    if monophyletic_groups.0 {
        warn!("The whole tree is a monophyletic clade!")
    }
    let pre = if let Some(s) = prefix {
        s.clone()
    } else {
        "".to_string()
    };
    let mut counter = 0;
    for group in monophyletic_groups.1.iter() {
        if group.len() > 1 {
            for node in group {
                if *internal || tree.is_external(*node) {
                    tree.annotate_node(
                        *node,
                        String::from("Clade"),
                        AnnotationValue::Discrete(format!("{}_{}.{}", pre, value, counter)),
                    );
                }
                if *internal || !tree.is_external(*node) {
                    tree.annotate_node(
                        *node,
                        String::from(key),
                        AnnotationValue::Discrete(String::from(value)),
                    );
                }
            }
            counter += 1;
        }
    }
}

fn get_monophyletic_groups(
    tree: &MutableTree,
    node_ref: TreeIndex,
    key: &str,
    target_annotation: &str,
) -> (bool, Vec<Vec<TreeIndex>>) {
    if tree.is_external(node_ref) {
        if let Some(annotation) = tree.get_annotation(node_ref, key) {
            match annotation {
                AnnotationValue::Discrete(s) => {
                    return if s == target_annotation {
                        (true, vec![vec![node_ref]])
                    } else {
                        (false, vec![vec![]])
                    };
                }
                _ => {
                    panic!("not a discrete trait")
                }
            }
        } else {
            return (false, vec![vec![]]);
        }
        // ignoring empty nodes they are counted
        // panic!("Annotation not found on a tip: {}. all tips must be annotated", tree.get_taxon(node_ref).unwrap_or("no label"));
    }

    let mut child_output = vec![];
    for child in tree.get_children(node_ref).iter() {
        child_output.push(get_monophyletic_groups(
            tree,
            *child,
            key,
            &target_annotation,
        ))
    }
    let am_i_a_root = child_output
        .iter()
        .map(|t| t.0)
        .fold(true, |acc, b| acc & b);
    if am_i_a_root {
        let mut combined_child_tips = child_output
            .into_iter()
            .map(|t| t.1)
            .flatten()
            .flatten()
            .collect::<Vec<TreeIndex>>();
        combined_child_tips.push(node_ref);
        (true, vec![combined_child_tips])
    } else {
        let child_tips = child_output
            .into_iter()
            .map(|t| t.1)
            .fold(vec![], |mut acc, next| {
                acc.extend(next);
                acc
            });
        (false, child_tips)
    }
}
