// mod introductions {
//     use rebl::tree::fixed_tree::FixedNode;
//
//     struct TaxaIntroductionLabel {
//         taxa: String,
//         introduction: usize,
//         tmrca: f64,
//     }
//
//     fn label_introductions(tree: FixedNode) {}
// }



pub mod clades {
    use rebl::tree::mutable_tree::{MutableTree, TreeIndex};
    use rebl::tree::AnnotationValue;
    use std::collections::HashSet;
    use structopt::StructOpt;

    use rand::seq::SliceRandom;
    use std::error::Error;
    use std::io::Write;
    use rebl::io::parser::tree_importer::TreeImporter;

    #[derive(Debug, StructOpt)]
    pub struct SharedOptions {
        #[structopt(short, long, help = "annotation key we are collapsing by. must be discrete")]
        annotation: String,
        #[structopt(short, long, help = "annotation value we are collapsing by")]
        value: String,
    }

    #[derive(Debug, StructOpt)]
    pub enum SubCommands {
        /// annotate tips with unique clade key based on annotation
        Label {
            #[structopt(short, long, help = "prefix for output annotation, if not provided defaults to 'annotation_value.' - Not implemented")]
            prefix: Option<String>,
            #[structopt(short, long, help = "annotation key we are collapsing by. must be discrete")]
            annotation: String,
            #[structopt(short, long, help = "annotation value we are collapsing by")]
            value: String,
        },
        /// Collapse monophyletic clades
        Collapse {
            #[structopt(short, long, help = "annotation key we are collapsing by. must be discrete")]
            annotation: String,
            #[structopt(short, long, help = "annotation value we are collapsing by")]
            value: String,
            #[structopt(short, long, help = "the minimum clade size", default_value = "1")]
            min_size: usize,
        },
    }


    //TODO set random seed.
    pub fn run<R: std::io::Read, T: TreeImporter<R>>(mut trees: T,
                                                     cmd: SubCommands) -> Result<(), Box<dyn Error>> {
        let stdout = std::io::stdout(); // get the global stdout entity
        let mut handle = stdout.lock(); // acquire a lock on it

        while trees.has_tree() {
            let mut tree = trees.read_next_tree()?;
            match cmd {
                SubCommands::Collapse { ref annotation, ref value, min_size } => {
                    let new_tree = collapse_uniform_clades(&mut tree, &annotation, &value, min_size);
                    writeln!(handle, "{}", new_tree)?;
                }
                SubCommands::Label { ref annotation, ref value, ref prefix } => {
                    annotate_uniform_clades(&mut tree, &annotation, &value, &prefix);
                    writeln!(handle, "{}", tree)?;
                }
            }
        }
        Ok(())
    }

    pub fn collapse_uniform_clades(tree: &mut MutableTree, key: &str, value: &str, min_size: usize) -> MutableTree {
        tree.calc_node_heights();

        let mut taxa: HashSet<String> = tree
            .external_nodes
            .iter()
            .map(|node| tree.get_taxon(*node))
            .map(|n| String::from(n.unwrap()))
            .collect();

        let monophyletic_groups =
            get_monophyletic_groups(tree, tree.get_root().unwrap(), key, value);
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

    fn annotate_uniform_clades(tree: &mut MutableTree, key: &str, value: &str, prefix: &Option<String>) {
        let monophyletic_groups =
            get_monophyletic_groups(tree, tree.get_root().unwrap(), key, value);
        if monophyletic_groups.0 {
            warn!("The whole tree is a monophyletic clade!")
        }
        let pre = if let Some(s) = prefix {
            s.clone()
        } else { "".to_string() };
        let mut counter = 0;
        for group in monophyletic_groups.1.iter() {
            if group.len() > 1 {
                for node in group {
                    tree.annotate_node(*node, String::from("Clade"), AnnotationValue::Discrete(format!("{}_{}.{}", pre, value, counter)));
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
            }
            // not ignoring empty nodes they are counted
            panic!("Annotation not found on a tip. all tips must be annotated")
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
            let combined_child_tips = child_output
                .into_iter()
                .map(|t| t.1)
                .flatten()
                .flatten()
                .collect::<Vec<TreeIndex>>();
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
}

pub mod annotate {
    use super::command_io;

    use rebl::tree::mutable_tree::MutableTree;
    use rebl::tree::AnnotationValue;
    use std::collections::HashMap;
    use std::error::Error;
    use std::io::Write;
    use std::path;

    use csv::Reader;
    use std::fs::File;
    use rebl::io::parser::tree_importer::TreeImporter;

    pub fn run<R: std::io::Read, T: TreeImporter<R>>(mut trees: T,
                                                     traits: path::PathBuf,
    ) -> Result<(), Box<dyn Error>> {
        let stdout = std::io::stdout(); // get the global stdout entity
        let mut handle = stdout.lock(); // acquire a lock on it

        while trees.has_tree() {
            let mut tree = trees.read_next_tree()?;
            //TODO avoid parsing at each loop
            let mut reader = command_io::parse_tsv(&traits)?;
            annotate_tips(&mut tree, &mut reader)?;
            writeln!(handle, "{}", tree)?;
        }
        Ok(())
    }

    pub fn annotate_tips(
        tree: &mut MutableTree,
        reader: &mut Reader<File>,
    ) -> Result<(), Box<dyn Error>> {
        //todo fix to handle taxa differently
        type Record = HashMap<String, Option<AnnotationValue>>;

        let header = reader.headers()?;
        let taxon_key = header.get(0).unwrap().to_string();

        for result in reader.deserialize() {
            let record: Record = result?;
            if let Some(AnnotationValue::Discrete(taxon)) = record.get(&*taxon_key).unwrap() {
                if let Some(node_ref) = tree.get_taxon_node(&taxon) {
                    for (key, value) in record {
                        if key != taxon_key {
                            if let Some(annotation_value) = value {
                                tree.annotate_node(node_ref, key, annotation_value)
                            }
                        }
                    }
                } else {
                    warn!("Taxon {} not found in tree", taxon)
                }
            }
        }
        Ok(())
    }
}

pub mod extract {
    use std::error::Error;
    use structopt::StructOpt;

    use rebl::tree::AnnotationValue;
    use std::io::Write;
    use rebl::io::parser::tree_importer::TreeImporter;

    #[derive(Debug, StructOpt)]
    pub enum SubCommands {
        /// Extract a list of the taxa names
        Taxa,
        /// Extract a tsv of the tip anotations
        Annotations,
        /// Extract a tree from a nexus file
        Tree{
            #[structopt(long,required_if("index", "None"), help = "the id of the tree to extract")]
            id: Option<String>,
            #[structopt(long,required_if("id", "None"),help = "The 0 based index of the tree to extract.")]
            index: Option<usize>,
        }

    }

    pub fn run<R: std::io::Read, T: TreeImporter<R>>(trees: T,
                                                     cmd: SubCommands,
    ) -> Result<(), Box<dyn Error>> {
        match cmd {
            SubCommands::Taxa => taxa(trees),
            SubCommands::Annotations => annotations(trees),
            SubCommands::Tree { id, index } => tree(trees, id, index),

        }
    }

    fn taxa<R: std::io::Read, T: TreeImporter<R>>(mut trees: T) -> Result<(), Box<dyn Error>> {
        let stdout = std::io::stdout(); // get the global stdout entity
        let mut handle = stdout.lock(); // acquire a lock on it
        while trees.has_tree() {
            let tree = trees.read_next_tree()?;
            let mut i = 0;
            while i < tree.get_external_node_count() {
                if let Some(tip) = tree.get_external_node(i) {
                    if let Some(taxa) = &tip.taxon {
                        writeln!(handle, "{}", taxa)?;
                    }
                }
                i += 1;
            }
        }
        Ok(())
    }

    fn annotations<R: std::io::Read, T: TreeImporter<R>>(mut trees: T) -> Result<(), Box<dyn Error>> {
        let stdout = std::io::stdout(); // get the global stdout entity
        let mut handle = stdout.lock(); // acquire a lock on it
        while trees.has_tree() {
            let tree = trees.read_next_tree()?;
            let header = tree
                .annotation_type
                .keys()
                .cloned()
                .collect::<Vec<String>>()
                .join("\t");
            writeln!(handle, "taxa\t{}", header)?;
            for node_ref in tree.external_nodes.iter() {
                let annotation_string = tree
                    .annotation_type
                    .keys()
                    .map(|k| annotation_value_string(tree.get_annotation(*node_ref, k)))
                    .collect::<Vec<String>>()
                    .join("\t");
                if let Some(taxa) = tree.get_taxon(*node_ref) {
                    writeln!(handle, "{}\t{}", taxa, annotation_string)?;
                } else {
                    writeln!(handle, "\t{}", annotation_string)?;
                }
            }
        }
        Ok(())
    }

    fn annotation_value_string(value: Option<&AnnotationValue>) -> String {
        if let Some(annotation) = value {
            annotation.to_string()
        } else {
            "".to_string()
        }
    }

    fn tree<R: std::io::Read, T:TreeImporter<R>>(mut trees: T, id: Option<String>, index: Option<usize>)->Result<(), Box<dyn Error>> {
        let stdout = std::io::stdout(); // get the global stdout entity
        let mut handle = stdout.lock(); // acquire a lock on it
       //TODO could skip trees that don't match instead of parsing them all.
        let mut found =false;

        if let Some(i)=index{
            let mut k = 0;
            while trees.has_tree()& !found {
                let tree = trees.read_next_tree()?;
                if k==i{
                    writeln!(handle,"{}", tree)?;
                    found=true;
                }
                k+=1;
            }
        }else if let Some(tree_id) =id {
            while trees.has_tree()& !found {
                let tree = trees.read_next_tree()?;
                if Some(tree_id.as_str())==tree.get_id(){
                    writeln!(handle,"{}", tree)?;
                    found=true;
                }
            }
        };
        if !found{
            warn!("Tree not found");
        }
        Ok(())
    }
}

pub mod split {
    use rebl::tree::mutable_tree::{MutableTree, TreeIndex};
    use std::collections::HashSet;
    use std::error::Error;
    use std::io::Write;
    use rebl::io::parser::tree_importer::TreeImporter;

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

    pub fn run<R: std::io::Read, T: TreeImporter<R>>(mut trees: T,
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
                searcher.collate_subtrees(
                    min_clade_size.expect("min-clade should be set to an integer"),
                );
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
                    debug!("{:?}", subtree);
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
}

pub mod stats {
    use std::error::Error;
    use std::io::Write;
    use structopt::StructOpt;
    use rebl::io::parser::tree_importer::TreeImporter;

    #[derive(Debug, StructOpt)]
    pub enum SubCommands {
        Heights,
    }

    fn general_stats<R: std::io::Read, T: TreeImporter<R>>(mut trees: T) -> Result<(), Box<dyn Error>> {
        let stdout = std::io::stdout(); // get the global stdout entity
        let mut handle = stdout.lock(); // acquire a lock on it
        writeln!(handle, "nodes\ttips\trootHeight\tsumbl\tmeanbl")?;

        while trees.has_tree() {
            let mut tree = trees.read_next_tree()?;
            let root = tree.get_root().unwrap();
            let nodes = tree.get_node_count();
            // let internal = tree.get_internal_node_count();
            let tips = tree.get_external_node_count();
            let mut bl = Vec::with_capacity(tree.get_node_count());
            bl.resize(tree.get_node_count(), 0.0);
            for node_ref in tree.preorder_iter() {
                if node_ref != tree.get_root().expect("stats assume rooted nodes") {
                    if let Some(node) = tree.get_node(node_ref) {
                        if let Some(length) = node.length {
                            bl[node_ref] = length;
                        }
                    }
                }
            }
            let sum_bl = bl.iter().fold(0.0, |acc, x| acc + x);
            let mean_bl = sum_bl / ((tree.get_node_count() as f64) - 1.0); //no branch on root
            tree.calc_node_heights();
            let root_height = tree.get_height(root).unwrap();
            writeln!(
                handle,
                "{}\t{}\t{:.2e}\t{:.2e}\t{:.2e}",
                nodes, tips, root_height, sum_bl, mean_bl
            )?;
        }
        Ok(())
    }

    fn node_heights<R:std::io::Read,T:TreeImporter<R>>(mut trees:T) -> Result<(), Box<dyn Error>> {
        let stdout = std::io::stdout(); // get the global stdout entity
        let mut handle = stdout.lock(); // acquire a lock on it
        writeln!(handle, "tree\theight\t\ttaxa")?;
        let mut t=0; //TODO use id if in tree maybe every tree gets an id in parser
        while trees.has_tree(){
            let mut tree = trees.read_next_tree()?;
            tree.calc_node_heights();
            for i in 0..tree.get_node_count(){
                let taxa = match tree.get_taxon(i){
                  Some(t) =>t,
                    None=>""
                };
                let height = tree.get_height(i).expect("Heights should be calculated");
                writeln!(handle,"{}\t{}\t{}", t, height, taxa);
            }
            t+=1;
        }

        Ok(())

    }

    pub fn run<R: std::io::Read, T: TreeImporter<R>>(
        trees: T,
        cmd: Option<SubCommands>,
    ) -> Result<(), Box<dyn Error>> {
        //TODO move tree reading and output buffer handling out here and pass to commands

        match cmd {
            None => general_stats(trees),
            Some(SubCommands::Heights) => node_heights(trees),
            _ => {
                warn!("nothing done");
                Ok(())
            }
        }
    }
}

pub mod transmission_lineage {
    use rebl::io::parser::tree_importer::TreeImporter;
    use std::error::Error;
    use rebl::tree::AnnotationValue;
    use rebl::tree::mutable_tree::MutableTree;
    use std::f32::NEG_INFINITY;
    use std::io::Write;

    #[derive(Debug)]
    struct TransmissionLineage {
        taxa: Vec<String>,
        tmrca: f64,
        parent_tmrca: f64,
        id: usize,
    }

    impl TransmissionLineage {
        fn add_taxa(&mut self, taxa: String) {
            self.taxa.push(taxa)
        }
    }

    struct LineageFinder {
        lineages: Vec<TransmissionLineage>,
        key: String,
        value: AnnotationValue,
        //TODO ignore tips without annotations?
    }


    impl LineageFinder {
        fn new(key:String,value :AnnotationValue)->Self{
            LineageFinder{lineages:vec![],key,value}
        }
        fn clear(&mut self){
            self.lineages = vec![];
        }
        fn find_lineages(&mut self, tree: &MutableTree, node: usize, lineage_index: Option<usize>) {
            if let Some(parent) = tree.get_parent(node) {
                let child_annotation = tree.get_annotation(node, &self.key).unwrap();

                if child_annotation == &self.value {
                    if let Some(li) = lineage_index { // parent was in this lineage
                        if tree.is_external(node) {
                            let l = & mut self.lineages[li];
                            l.add_taxa(tree.get_taxon(node).unwrap().to_string())
                        } else {
                            for child in tree.get_children(node) {
                                self.find_lineages(tree, child, lineage_index);
                            }
                        }
                    } else { // new lineage
                        let id = self.lineages.len();
                        let new_lineage = TransmissionLineage {
                            taxa: vec![],
                            tmrca: tree.get_height(node).unwrap(),
                            parent_tmrca: tree.get_height(parent).unwrap(),
                            id,
                        };
                        self.lineages.push(new_lineage);
                        if tree.is_external(node) {
                            let l = &mut self.lineages[id];
                            l.add_taxa(tree.get_taxon(node).unwrap().to_string())
                        } else {
                            for child in tree.get_children(node) {
                                self.find_lineages(tree, child, Some(id));
                            }
                        }
                    }
                } else {
                    for child in tree.get_children(node) {
                        self.find_lineages(tree, child, None);
                    }
                }
            }else{
                //At the root
                let child_annotation = tree.get_annotation(node, &self.key).unwrap();
                if child_annotation==&self.value{
                    let id = self.lineages.len();
                    let new_lineage = TransmissionLineage {
                        taxa: vec![],
                        tmrca: tree.get_height(node).unwrap(),
                        parent_tmrca: NEG_INFINITY as f64,
                        id,
                    };
                    self.lineages.push(new_lineage);
                    for child in tree.get_children(node) {
                        self.find_lineages(tree, child, Some(id));
                    }
                } else{
                    for child in tree.get_children(node) {
                        self.find_lineages(tree, child, None);
                    }
                }
            }
        }
    }

    pub fn run<R: std::io::Read, T: TreeImporter<R>>(mut trees: T, key: String, value: String) -> Result<(), Box<dyn Error>> {
        let stdout = std::io::stdout(); // get the global stdout entity
        let mut handle = stdout.lock(); // acquire a lock on it
        writeln!(handle,"tree\tlineage\ttaxa\ttmrca\tptmrca")?;

        let mut count = 0;
        let mut lineage_finder = LineageFinder::new(key, AnnotationValue::Discrete(value));
        while trees.has_tree() {
            let mut tree = trees.read_next_tree()?;
            tree.calc_node_heights();
            lineage_finder.find_lineages(&tree, tree.get_root().unwrap(), None);
            for l in &lineage_finder.lineages{
                for taxa in &l.taxa{
                    writeln!(handle, "{}\t{}\t{}\t{}\t{}", count,l.id,taxa,l.tmrca,l.parent_tmrca)?;
                }
            }
            count+=1;
            lineage_finder.clear()
        }
        Ok(())
    }
    #[cfg(test)]
    mod tests {
        use rebl::io::parser::newick_importer::NewickImporter;
        use std::io::BufReader;
        use crate::commands::transmission_lineage::LineageFinder;
        use rebl::tree::AnnotationValue;

        #[test]
        fn find_lineages(){
            let s = "((A[&location=UK]:0.1,B[&location=USA]:0.1)[&location=UK]:0.1,'C d'[&location=US]:0.1)[&location=US];";
            let mut tree = NewickImporter::read_tree(BufReader::new(s.as_bytes())).expect("error in parsing");
            //TODO work this into Lineage Finder
            tree.calc_node_heights();
            let mut lf = LineageFinder::new("location".to_string(),AnnotationValue::Discrete("UK".to_string()));

            lf.find_lineages(&tree, tree.get_root().unwrap(), None);
            assert_eq!(1, lf.lineages.len());
            assert_eq!("A",lf.lineages[0].taxa[0]);
        }
        #[test]
        fn find_2lineages(){
            let s = "((A[&location=UK]:0.1,B[&location=US]:0.1)[&location=UK]:0.1,'C d'[&location=US]:0.1)[&location=US];";
            let mut tree = NewickImporter::read_tree(BufReader::new(s.as_bytes())).expect("error in parsing");
            //TODO work this into Lineage Finder
            tree.calc_node_heights();
            let mut lf = LineageFinder::new("location".to_string(),AnnotationValue::Discrete("US".to_string()));

            lf.find_lineages(&tree, tree.get_root().unwrap(), None);
            assert_eq!(2, lf.lineages.len());
        }
    }
}

pub mod resolve {
    use rand::{thread_rng, Rng};
    use rebl::tree::mutable_tree::{MutableTree, TreeIndex};
    use std::error::Error;
    use std::io::Write;
    use structopt::StructOpt;
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
}


pub mod command_io {
    use csv::Reader;
    use std::error::Error;
    use std::fs::File;
    use std::path;

    //HashMap<String,HashMap<String,AnnotationValue>>
    pub fn parse_tsv(trait_file: &path::PathBuf) -> Result<Reader<File>, Box<dyn Error>> {
        let file = File::open(trait_file)?;
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .flexible(true)
            .comment(Some(b'#'))
            .from_reader(file);

        // We nest this call in its own scope because of lifetimes.
        debug!("read with headers:{:?}", rdr.headers().unwrap());

        Ok(rdr)
    }
}
