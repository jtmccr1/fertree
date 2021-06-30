use rand::{thread_rng, Rng};
use std::error::Error;
use std::io::Write;
use structopt::StructOpt;
use rebl::tree::mutable_tree::{TreeIndex, MutableTree};
use rebl::io::parser::tree_importer::TreeImporter;

#[derive(Debug, StructOpt)]
pub enum SubCommands {
    /// insert branches with length 0
    Zero,
    /// spread the nodes evenly between the halfway point between parent node and oldest child
    Evenly,
}

struct Polytomy {
    root: TreeIndex,
    tips: Vec<TreeIndex>,
}

pub fn run<R: std::io::Read, T: TreeImporter<R>>(mut trees: T,
                                                 cmd: SubCommands,
) -> Result<(), Box<dyn Error>> {
    let stdout = std::io::stdout(); // get the global stdout entity
    let mut handle = stdout.lock(); // acquire a lock on it
    while trees.has_tree() {
        let mut tree = trees.read_next_tree()?;
        resolve(&mut tree, &cmd);
        writeln!(handle, "{}", tree)?;
    }
    Ok(())
}

// collect all poltyomies and child vectors in a stuct
// set heights
fn resolve(tree: &mut MutableTree, cmd: &SubCommands) {
    if let SubCommands::Evenly = cmd {
        tree.calc_node_heights();
    }
    let mut polytomies = tree.preorder_iter()
        .filter(|node| !tree.is_external(*node))
        .map(|n| (n, tree.get_children(n)))
        .filter(|(_n, kids)| kids.len() > 2)
        .map(|(root, tips)| Polytomy { root, tips })
        .collect::<Vec<Polytomy>>();
    let node_count = tree.get_node_count();
    info!("{} polytomies found", polytomies.len());
    for polytomy in polytomies.iter() {
        insert_nodes(tree, polytomy.root)
    }

    info!("resolved with {} nodes", tree.get_node_count() - node_count);

    match cmd {
        SubCommands::Zero => {
            for polytomy in polytomies.iter() {
                for tip in polytomy.tips.iter() {
                    let mut node = *tip;
                    while let Some(parent) = tree.get_parent(node) {
                        if parent == polytomy.root || tree.get_length(parent).is_some() {
                            break;
                        }
                        tree.set_length(parent, 0.0);
                        node = parent;
                    }
                }
            }
            debug!(
                "done setting branch lengths \n heights known : {} - lengths known: {}",
                tree.heights_known, tree.branchlengths_known
            );
        }
        SubCommands::Evenly => {
            debug!(
                "about to set  setting node heights \n heights known : {} - lengths known: {}",
                tree.heights_known, tree.branchlengths_known
            );
            for polytomy in polytomies.iter_mut() {
                // scootch the root node up a little

                if let Some(bl) = tree.get_length(polytomy.root) {
                    tree.set_height(
                        polytomy.root,
                        tree.get_height(polytomy.root).unwrap() + bl * 0.5,
                    );
                }

                polytomy.tips.sort_unstable_by(|a, b| {
                    tree.get_height(*b)
                        .partial_cmp(&tree.get_height(*a))
                        .unwrap()
                });
                for tip in polytomy.tips.iter() {
                    // get path back to tip with set height
                    // space out evenly between this and some factor of the the tip.
                    let mut path_to_proot = vec![];
                    let mut node = *tip;

                    let mut upper_bound = tree.get_height(*tip).unwrap();

                    while let Some(parent) = tree.get_parent(node) {
                        if tree.get_height(parent).is_some() {
                            upper_bound = tree.get_height(parent).unwrap();
                            break;
                        }
                        path_to_proot.push(parent);
                        node = parent;
                    }
                    let lower_bound = tree
                        .get_height(*tip)
                        .expect("lowerbound node should have a height")
                        + tree.get_length(*tip).unwrap() * 0.5;

                    let diff = (upper_bound - lower_bound) / ((path_to_proot.len() + 1) as f64);
                    let mut height = lower_bound + diff;

                    for node in path_to_proot.iter() {
                        tree.set_height(*node, height);
                        height += diff;
                    }
                }
            }
            tree.calculate_branchlengths();
            debug!(
                "done setting node heights \n heights known : {} - lengths known: {}",
                tree.heights_known, tree.branchlengths_known
            )
        }
    }
}

/// function that takes a polytomy node and randomly resolves
///
fn insert_nodes(tree: &mut MutableTree, node_ref: TreeIndex) {
    //dumb way
    //remove all kids
    // split kids into two groups
    // if group is 1 add it as child
    //if group is add internal node as child and repeat
    let mut kids = vec![];
    for child in tree.get_children(node_ref) {
        let removed = tree.remove_child(node_ref, child);
        if let Some(c) = removed {
            kids.push(c);
        }
    }
    // TODO maybe not recursive.
    let mut rng = thread_rng();
    let n: usize = rng.gen_range(1..kids.len());

    let first_family = &kids[0..n];
    let second_family = &kids[n..kids.len()];

    if first_family.len() == 1 {
        tree.add_child(node_ref, first_family[0]);
        tree.set_parent(node_ref, first_family[0]);
    } else {
        let still_polytomy = first_family.len() > 2;
        let kido = tree.make_internal_node(first_family.to_owned());
        tree.add_child(node_ref, kido);
        tree.set_parent(node_ref, kido);
        insert_nodes(tree, kido);
        if still_polytomy {
            insert_nodes(tree, kido);
        }
    }

    if second_family.len() == 1 {
        tree.add_child(node_ref, second_family[0]);
        tree.set_parent(node_ref, second_family[0]);
    } else {
        let still_polytomy = second_family.len() > 2;
        let kido = tree.make_internal_node(second_family.to_owned());
        tree.add_child(node_ref, kido);
        tree.set_parent(node_ref, kido);
        if still_polytomy {
            insert_nodes(tree, kido);
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::commands::resolve::{resolve, SubCommands};
    use rebl::io::parser::newick_importer::NewickImporter;
    use std::io::BufReader;

    // these just run at the momement. Need a way to compare the clades to ensure they are working
    #[test]
    fn zero() {
        let tree_string = "((A:1,(B:1,C:1,D:1):1,E:1):1,F:1,G:1);";
        let mut tree = NewickImporter::read_tree(BufReader::new(tree_string.as_bytes())).unwrap();
        println!("{}", tree.branchlengths_known);
        resolve(&mut tree, &SubCommands::Zero);
        println!("{}", tree.branchlengths_known);
        println!("{}", tree.to_string());
        let mut bl = 0.0;
        for node in tree.nodes {
            if let Some(l) = node.length {
                bl += l;
            }
        }
        assert_eq!(9.0, bl);
    }


    #[test]
    fn evenly() {
        let tree_string = "((A:1,(B:1,C:1,D:1,a:1):1,E:1):1,F:1,G:1);";
        let mut tree = NewickImporter::read_tree(BufReader::new(tree_string.as_bytes())).unwrap();
        tree.calc_node_heights();
        let starting_height = tree.get_height(tree.root.unwrap());
        resolve(&mut tree, &SubCommands::Evenly);
        println!("{}", tree.to_string());

        assert_eq!(starting_height, tree.get_height(tree.root.unwrap()));
    }
}

