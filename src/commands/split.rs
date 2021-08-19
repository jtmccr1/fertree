use rebl::io::parser::tree_importer::TreeImporter;
use rebl::tree::mutable_tree::{MutableTree, TreeIndex};
use std::collections::HashSet;
use std::error::Error;
use std::io::Write;

#[derive(Debug, PartialEq)]
struct Subtree {
    root: TreeIndex,
    tips: usize,
    level: usize,
}

struct SubtreeSearcher {
    tree: MutableTree,
    subtrees: Vec<Subtree>,
    strict: bool,
}

impl SubtreeSearcher {
    fn collate_subtrees(&mut self, min_size: usize) {
        let root = self.tree.get_root().unwrap();
        self.subtrees = vec![];
        self.get_subtrees(root, min_size, 0);
    }
    fn get_subtrees(&mut self, node: TreeIndex, min_size: usize, level: usize) -> usize {
        return if self.tree.is_external(node) {
            1
        } else {
            let mut tips = 0;
            for child in self.tree.get_children(node) {
                tips += self.get_subtrees(child, min_size, level + 1);
            }
            if tips >= min_size {
                let subtree = Subtree {
                    root: node,
                    tips,
                    level,
                };
                self.subtrees.push(subtree);
                return 0;
            } else if Some(node) == self.tree.get_root() {
                if self.strict && tips < min_size && !self.subtrees.is_empty() {
                    let earliest_subtree = self.subtrees.iter().fold(
                        &Subtree {
                            root: usize::MAX,
                            tips: usize::MIN,
                            level: usize::MAX,
                        },
                        |a, b| {
                            if a.level < b.level {
                                a
                            } else if b.level < a.level {
                                b
                            } else if a.tips < b.tips {
                                a
                            } else {
                                b
                            }
                        },
                    );

                    //if this is slow could make subtree mutable
                    let new_tip_count = tips + earliest_subtree.tips;
                    let root_subtree = Subtree {
                        root: node,
                        tips: tips + earliest_subtree.tips,
                        level,
                    };
                    //TODO error
                    let index = self
                        .subtrees
                        .iter()
                        .position(|x| *x == *earliest_subtree)
                        .expect("subtree not found");
                    self.subtrees.swap_remove(index);
                    self.subtrees.push(root_subtree);

                    return new_tip_count;
                } else {
                    let subtree = Subtree {
                        root: node,
                        tips,
                        level,
                    };
                    self.subtrees.push(subtree);
                    return 0;
                }
            }
            tips
        };
    }

    fn finalize_selection(&mut self) {
        for subtree in self.subtrees.iter() {
            if let Some(parent) = self.tree.get_parent(subtree.root) {
                self.tree.remove_child(parent, subtree.root);
            }
        }
    }
}

pub fn run<R: std::io::Read, T: TreeImporter<R>>(
    mut trees: T,
    min_clade_size: Option<usize>,
    explore: bool,
    strict: bool,
) -> Result<(), Box<dyn Error>> {
    let stdout = std::io::stdout(); // get the global stdout entity
    let mut handle = stdout.lock(); // acquire a lock on it

    if explore && min_clade_size.is_some() {
        warn!("Because explore is set. No trees will be written");
    }
    while trees.has_tree() {
        let mut starting_tree = trees.read_next_tree()?;
        starting_tree.calc_node_heights();
        trace!("starting to split");
        let mut searcher = SubtreeSearcher {
            tree: starting_tree,
            subtrees: vec![],
            strict,
        };

        if explore && min_clade_size.is_none() {
            writeln!(handle, "Exploring tree topology")?;
            let tip_count = searcher.tree.get_external_node_count();
            let mut min_size = 4;
            while min_size < tip_count {
                searcher.collate_subtrees(min_size);
                writeln!(
                    handle,
                    "cutoff of {} leads to {} trees",
                    min_size,
                    searcher.subtrees.len()
                )?;
                min_size *= 2;
            }
        } else {
            searcher
                .collate_subtrees(min_clade_size.expect("min-clade should be set to an integer"));
            let taxa = &searcher
                .tree
                .external_nodes
                .iter()
                .map(|n| searcher.tree.get_taxon(*n).unwrap().to_string())
                .collect::<HashSet<String>>();
            searcher.finalize_selection();
            info!("found {} trees", searcher.subtrees.len());

            if explore {
                writeln!(handle, "tree\ttips")?;
            }

            for (i, subtree) in searcher.subtrees.iter().enumerate() {
                if explore {
                    writeln!(handle, "{}\t{}", i, subtree.tips)?;
                } else {
                    info!("tree: {} - {} tips", i, subtree.tips);
                }
            }
            if !explore {
                for subtree in searcher.subtrees {
                    let mut st = MutableTree::copy_subtree(&searcher.tree, subtree.root, taxa);
                    st.calculate_branchlengths();

                    writeln!(handle, "{}", st)?;
                }
            }
        }
    }
    Ok(())
}
