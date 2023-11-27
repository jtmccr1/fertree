use crate::commands::command_io;
use rebl::io::parser::tree_importer::TreeImporter;
use rebl::tree::mutable_tree::MutableTree;
use rebl::tree::AnnotationValue;
use std::collections::HashSet;
use std::error::Error;
use std::io::Write;
use std::path;

#[derive(Debug)]
struct TransmissionLineage {
    taxa: Vec<String>,
    tmrca: f64,
    parent_tmrca: f64,
    id: usize,
    source: String,
    last_seen: f64,
    first_seen: f64,
}

impl TransmissionLineage {
    fn add_taxa(&mut self, tree: &MutableTree, node: usize) {
        let taxa = tree.get_taxon(node).unwrap().to_string();
        self.taxa.push(taxa);
        let height = tree
            .get_height(node)
            .expect("You need to calculate heights before making lineages");
        let rt_height = tree.get_height(tree.get_root().unwrap()).unwrap();
        if (rt_height - height).abs() > (rt_height - self.last_seen).abs() {
            self.last_seen = height;
        }
        if (rt_height - height).abs() < (rt_height - self.first_seen).abs() {
            self.first_seen = height;
        }
    }
}

struct LineageFinder {
    lineages: Vec<TransmissionLineage>,
    key: String,
    value: AnnotationValue,
    ignore_taxa: HashSet<String>,
    cutoff: f64,
    lag: f64,
    //TODO ignore tips without annotations?
}

impl LineageFinder {
    fn new(
        key: String,
        value: AnnotationValue,
        ignore_taxa: HashSet<String>,
        cutoff: f64,
        lag: f64,
    ) -> Self {
        LineageFinder {
            lineages: vec![],
            key,
            value,
            ignore_taxa,
            cutoff,
            lag,
        }
    }
    fn clear(&mut self) {
        self.lineages = vec![];
    }
    fn find_lineages(&mut self, tree: &MutableTree, node: usize, lineage_index: Option<usize>) {
        if let Some(mut parent) = tree.get_parent(node) {
            let annotation = tree.get_annotation(node, &self.key);

            if annotation.is_some() && annotation.unwrap() == &self.value {
                if let Some(li) = lineage_index {
                    // parent was in this lineage
                    if tree.is_external(node) {
                        let taxa = tree.get_taxon(node).expect("tip should have a taxon");
                        if !self.ignore_taxa.contains(taxa) {
                            // if we are not ignoring this tip
                            let l = &mut self.lineages[li];
                            l.add_taxa(tree, node);
                        }
                    } else {
                        // if it respects the lag

                        if self.will_be_sampled_before_lag(tree, node, 0.0)
                            && !self.has_been_sampled_within_lag(tree, node, 0.0)
                        {
                            let id = self.lineages.len();
                            let default_location =
                                AnnotationValue::Discrete("unknown".parse().unwrap());
                            let mut parent_location = tree
                                .get_annotation(parent, &self.key)
                                .unwrap_or(&default_location);
                            // if parent is in the same location assert it was passed up because of the
                            //height cutoff and go back to root or first parent with location not here
                            let parent_height = tree.get_height(parent).unwrap();
                            while parent_location == &self.value {
                                parent = tree
                                    .get_parent(parent)
                                    .expect("Hit the root looking for ancestor location");
                                parent_location = tree
                                    .get_annotation(parent, &self.key)
                                    .unwrap_or(&default_location);
                            }

                            let new_lineage = TransmissionLineage {
                                taxa: vec![],
                                tmrca: tree.get_height(node).unwrap(),
                                parent_tmrca: parent_height,
                                id,
                                source: parent_location.to_string(),
                                first_seen: 0.0,
                                last_seen: tree.get_height(tree.get_root().unwrap()).unwrap(),
                            };
                            self.lineages.push(new_lineage);
                            trace!("adding new lineage due to gap in sampling");
                            for child in tree.get_children(node) {
                                self.find_lineages(tree, child, Some(id));
                            }
                        } else {
                            let current_index = if self.will_be_sampled_before_lag(tree, node, 0.0)
                            {
                                lineage_index
                            } else {
                                None
                            };
                            for child in tree.get_children(node) {
                                self.find_lineages(tree, child, current_index);
                            }
                        }
                    };
                } else if tree.get_height(node).expect("nodes should have heights") >= self.cutoff
                    && self.will_be_sampled_before_lag(tree, node, 0.0)
                {
                    // new lineage

                    let id = self.lineages.len();
                    let default_location = AnnotationValue::Discrete("unknown".parse().unwrap());
                    let mut parent_location = tree
                        .get_annotation(parent, &self.key)
                        .unwrap_or(&default_location);
                    // if parent is in the same location assert it was passed up because of the
                    //height cutoff and go back to root or first parent with location not here
                    let parent_height = tree.get_height(parent).unwrap();
                    while parent_location == &self.value {
                        parent = tree
                            .get_parent(parent)
                            .expect("Hit the root looking for ancestor location");
                        parent_location = tree
                            .get_annotation(parent, &self.key)
                            .unwrap_or(&default_location);
                    }

                    let new_lineage = TransmissionLineage {
                        taxa: vec![],
                        tmrca: tree.get_height(node).unwrap(),
                        parent_tmrca: parent_height,
                        id,
                        source: parent_location.to_string(),
                        first_seen: 0.0,
                        last_seen: tree.get_height(tree.get_root().unwrap()).unwrap(),
                    };
                    self.lineages.push(new_lineage);
                    trace!("lineage found");

                    if tree.is_external(node) {
                        let taxa = tree
                            .get_taxon(node)
                            .expect("tip should have an associated taxon");
                        if !self.ignore_taxa.contains(taxa) {
                            // if we are not ignoring this tip
                            let l = &mut self.lineages[id];
                            l.add_taxa(tree, node);
                        }
                    } else {
                        for child in tree.get_children(node) {
                            self.find_lineages(tree, child, Some(id));
                        }
                    }
                } else {
                    for child in tree.get_children(node) {
                        self.find_lineages(tree, child, None);
                    }
                }
            } else {
                for child in tree.get_children(node) {
                    self.find_lineages(tree, child, None);
                }
            }
        } else {
            //At the root
            let annotation = tree.get_annotation(node, &self.key);
            if annotation.is_some()
                && annotation.unwrap() == &self.value
                && self.will_be_sampled_before_lag(tree, node, 0.0)
            {
                let id = self.lineages.len();
                let new_lineage = TransmissionLineage {
                    taxa: vec![],
                    tmrca: tree.get_height(node).unwrap(),
                    parent_tmrca: f64::NEG_INFINITY,
                    id,
                    source: "NA (at-root)".to_string(),
                    first_seen: 0.0,
                    last_seen: tree.get_height(tree.get_root().unwrap()).unwrap(),
                };
                self.lineages.push(new_lineage);
                for child in tree.get_children(node) {
                    self.find_lineages(tree, child, Some(id));
                }
            } else {
                for child in tree.get_children(node) {
                    self.find_lineages(tree, child, None);
                }
            }
        }
    }

    fn will_be_sampled_before_lag(
        &self,
        tree: &MutableTree,
        node: usize,
        current_lag: f64,
    ) -> bool {
        let mut respects = false;
        let default_location = AnnotationValue::Discrete("unknown".parse().unwrap());
        let node_annotation = tree
            .get_annotation(node, &self.key)
            .unwrap_or(&default_location);
        if self.lag == f64::INFINITY {
            respects = true;
        } else if node_annotation != &self.value || current_lag > self.lag {
            respects = false;
        } else if tree.is_external(node) {
            respects = true;
        } else {
            for child in tree.get_children(node) {
                let l = tree.get_length(child).unwrap() + current_lag;
                respects = respects || self.will_be_sampled_before_lag(tree, child, l);
                if respects {
                    break;
                }
            }
        }
        respects
    }
    // helper function to be called when checking if a new tl needs to be inserted.
    // this makes assumptions
    fn has_been_sampled_within_lag(
        &mut self,
        tree: &MutableTree,
        mut node: usize,
        current_lag: f64,
    ) -> bool {
        let mut respects = false;
        if self.lag == f64::INFINITY {
            respects = true;
        }
        let mut parent = tree.get_parent(node);
        let mut new_lag = current_lag;
        while current_lag < self.lag && parent.is_some() && !respects {
            let default_location = AnnotationValue::Discrete("unknown".parse().unwrap());
            let parent_annotation = tree
                .get_annotation(parent.unwrap(), &self.key)
                .unwrap_or(&default_location);
            if parent_annotation != &self.value {
                break;
            }
            new_lag += tree.get_length(node).unwrap();
            for child in tree.get_children(node) {
                if child != node {
                    respects = respects || self.will_be_sampled_before_lag(tree, child, new_lag);
                }
            }
            node = parent.unwrap();
            parent = tree.get_parent(node);
        }

        respects
    }
}

pub fn run<R: std::io::Read, T: TreeImporter<R>>(
    mut trees: T,
    ignore_taxa: Option<path::PathBuf>,
    key: String,
    value: String,
    taxa_flag: bool,
    origin: Option<f64>,
    cutoff: Option<f64>,
    lag: Option<f64>,
) -> Result<(), Box<dyn Error>> {
    let ignore = command_io::parse_taxa(ignore_taxa)?;
    let stdout = std::io::stdout(); // get the global stdout entity
    let mut handle = stdout.lock(); // acquire a lock on it
    if taxa_flag {
        writeln!(
            handle,
            "tree\tlineage\tntaxa\ttmrca\tptmrca\tsource\tfirst_seen\tlast_seen\ttaxa"
        )?;
    } else {
        writeln!(
            handle,
            "tree\tlineage\tntaxa\ttmrca\tptmrca\tsource\tfirst_seen\tlast_seen"
        )?;
    }
    let mut count = 0;
    let most_recent_intro = cutoff.unwrap_or(f64::NEG_INFINITY);
    let max_lag = lag.unwrap_or(f64::INFINITY);

    let mut lineage_finder = LineageFinder::new(
        key,
        AnnotationValue::Discrete(value),
        ignore,
        most_recent_intro,
        max_lag,
    );
    while trees.has_tree() {
        let mut tree = trees.read_next_tree()?;

        if let Some(most_recent_sample) = origin {
            tree.calc_relative_node_heights(most_recent_sample);
        } else {
            tree.calc_node_heights();
        }
        //if clades then annotate internal nodes with labels

        lineage_finder.find_lineages(&tree, tree.get_root().unwrap(), None);
        for l in &lineage_finder.lineages {
            if taxa_flag {
                writeln!(
                    handle,
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    count,
                    l.id,
                    l.taxa.len(),
                    l.tmrca,
                    l.parent_tmrca,
                    l.source,
                    l.first_seen,
                    l.last_seen,
                    l.taxa.join("; ")
                )?;
            } else {
                writeln!(
                    handle,
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    count,
                    l.id,
                    l.taxa.len(),
                    l.tmrca,
                    l.parent_tmrca,
                    l.source,
                    l.first_seen,
                    l.last_seen,
                )?;
            }
        }
        count += 1;
        lineage_finder.clear()
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::commands::transmission_lineage::LineageFinder;
    use rebl::io::parser::newick_importer::NewickImporter;
    use rebl::tree::AnnotationValue;
    use std::collections::HashSet;
    use std::io::BufReader;

    #[test]
    fn find_lineages() {
        let s = "((A[&location=UK]:0.1,B[&location=USA]:0.1)[&location=UK]:0.1,'C d'[&location=US]:0.1)[&location=US];";
        let mut tree =
            NewickImporter::read_tree(BufReader::new(s.as_bytes())).expect("error in parsing");
        //TODO work this into Lineage Finder
        tree.calc_node_heights();
        let mut lf = LineageFinder::new(
            "location".to_string(),
            AnnotationValue::Discrete("UK".to_string()),
            HashSet::new(),
            f64::NEG_INFINITY,
            f64::INFINITY,
        );

        lf.find_lineages(&tree, tree.get_root().unwrap(), None);
        assert_eq!(1, lf.lineages.len());
        assert_eq!("A", lf.lineages[0].taxa[0]);
    }

    #[test]
    fn find_lineages_lag() {
        let s = "((A[&location=UK]:0.1,B[&location=UK]:0.1)[&location=UK]:0.1,'C d'[&location=US]:0.1)[&location=US];";
        let mut tree =
            NewickImporter::read_tree(BufReader::new(s.as_bytes())).expect("error in parsing");
        //TODO work this into Lineage Finder
        tree.calc_node_heights();
        let mut lf = LineageFinder::new(
            "location".to_string(),
            AnnotationValue::Discrete("UK".to_string()),
            HashSet::new(),
            f64::NEG_INFINITY,
            0.05,
        );

        lf.find_lineages(&tree, tree.get_root().unwrap(), None);
        assert_eq!(2, lf.lineages.len());
        assert_eq!("A", lf.lineages[0].taxa[0]);
    }

    #[test]
    fn find_lineages_internal_lag() {
        let s = "((A[&location=UK]:0.1,B[&location=UK]:0.1)[&location=UK]:6.1,'C'[&location=UK]:0.1)[&location=US];";
        let mut tree =
            NewickImporter::read_tree(BufReader::new(s.as_bytes())).expect("error in parsing");
        //TODO work this into Lineage Finder
        tree.calc_node_heights();
        let mut lf = LineageFinder::new(
            "location".to_string(),
            AnnotationValue::Discrete("UK".to_string()),
            HashSet::new(),
            f64::NEG_INFINITY,
            1.0,
        );

        lf.find_lineages(&tree, tree.get_root().unwrap(), None);
        assert_eq!(2, lf.lineages.len());
        assert_eq!("A", lf.lineages[0].taxa[0]);
    }

    #[test]
    fn find_2lineages() {
        let s = "((A[&location=UK]:0.1,B[&location=US]:0.1)[&location=UK]:0.1,'C d'[&location=US]:0.1)[&location=US];";
        let mut tree =
            NewickImporter::read_tree(BufReader::new(s.as_bytes())).expect("error in parsing");
        //TODO work this into Lineage Finder
        tree.calc_node_heights();
        let mut lf = LineageFinder::new(
            "location".to_string(),
            AnnotationValue::Discrete("US".to_string()),
            HashSet::new(),
            f64::NEG_INFINITY,
            1.0,
        );

        lf.find_lineages(&tree, tree.get_root().unwrap(), None);
        assert_eq!(2, lf.lineages.len());
    }
}
