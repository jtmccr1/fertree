use structopt::StructOpt;
use rebl::io::parser::tree_importer::TreeImporter;
use std::error::Error;
use std::io::Write;
use rebl::tree::AnnotationValue;
use regex::Regex;
use rebl::tree::mutable_tree::MutableTree;


#[derive(Debug, StructOpt)]
pub enum SubCommands {
    Scale{
        #[structopt(short, long, help = "scalar to apply to all branches", default_value = "1")]
        scalar: f64,
    },
    MinLength{
        #[structopt(short, long, help = "minimum allowed branch length", default_value = "1")]
        min_length: f64,
    },
    Round,
    /// Convert the mutations annotations from treetime into an integer count of mutations
    TreeTime{
        #[structopt(subcommand)]
        sub_cmd: TreeTimeSubCommands
    },
}
#[derive(Debug, StructOpt)]
pub enum TreeTimeSubCommands{
    /// Includes all reported mutations "-" ambiguities etc.
    All,
    Transitions,
    Transversions,
    /// mutation must be in format "[ATCG]\d+[ATCG]" this would count A->A as a mutation
    Nucleotide,
    /// mutation must be in format "[A-Z]\d+[A-Z]" this would count A->A as a mutation
    NoIndel,
}


pub fn run<R: std::io::Read, T: TreeImporter<R>>(mut trees: T,
                                                 cmd: SubCommands) -> Result<(), Box<dyn Error>> {
    let stdout = std::io::stdout(); // get the global stdout entity
    let mut handle = stdout.lock(); // acquire a lock on it

    while trees.has_tree() {
        let mut tree = trees.read_next_tree()?;
        match cmd {
            SubCommands::Scale { scalar} => {
                scale(&mut tree, scalar);
            }
            SubCommands::MinLength { min_length} => {
                self::min_length(&mut tree, min_length);
            }
            SubCommands::Round=>{
                round(&mut tree);
            }
            SubCommands::TreeTime {ref sub_cmd}=>{
                tree_time(&mut tree, sub_cmd);
            }
        }
        writeln!(handle, "{}", tree)?;
    }
    Ok(())
}
//functions so we can test them
fn scale(tree:&mut MutableTree,scalar:f64){
    for i in 0..tree.get_node_count() {
        if let Some(l) = tree.get_length(i){
            tree.set_length(i, l * scalar);
        }
    }
}
fn min_length( tree:&mut MutableTree,min_length:f64){
    for i in 0..tree.get_node_count() {
        if let Some(l) = tree.get_length(i){
            if l< min_length{
                tree.set_length(i, min_length);
            }
        }
    }
}
fn round( tree:&mut MutableTree){
    for i in 0..tree.get_node_count() {
        if let Some(l) = tree.get_length(i){
            tree.set_length(i, l.round());
        }
    }
}

fn tree_time( tree:&mut MutableTree, cmd:&TreeTimeSubCommands){

    let re =    match cmd{
        TreeTimeSubCommands::All => {
            Regex::new(r"[ATCG-]\d+[ATCG-]").unwrap()
        },
        //TODO remove duplication
        TreeTimeSubCommands::Nucleotide => {
            Regex::new(r"[ATCG]\d+[ATCG]").unwrap()

        }
        TreeTimeSubCommands::NoIndel => {
            Regex::new(r"[A-Z]\d+[A-Z]").unwrap()

        }
        TreeTimeSubCommands::Transitions => {
             Regex::new(r"[AG]\d+[AG]|[CT]\d+[CT]").unwrap()

        }
        TreeTimeSubCommands::Transversions => {
           Regex::new(r"[AG]\d+[CT]|[CT]\d+[AG]").unwrap()
        }
    };

    for i in 0..tree.get_node_count() {
            if i!=tree.get_root().unwrap(){
                if let Some(mutations)=tree.get_annotation(i,"mutations"){
                    if let AnnotationValue::Discrete(mut_string) = mutations{
                        let muts = mut_string.split(',');
                        let mut counter =0;
                        for m in muts{
                            if re.is_match(m){
                                counter+=1;
                            }else{
                                if m.len()>1 {
                                    let n = if tree.is_external(i){tree.get_taxon(i).unwrap()}else{ "internal node"};
                                    trace!("mut {} on {} didn't match criteria", m,n)
                                }
                            }
                        }
                        tree.set_length(i, counter as f64);
                    }
                }else{
                    tree.set_length(i,0.0)
                }
            }
        }
}

#[cfg(test)]
mod tests {
    use rebl::io::parser::newick_importer::NewickImporter;
    use std::io::BufReader;
    use crate::commands::branchlengths::{scale, min_length, round, tree_time, TreeTimeSubCommands};

    #[test]
    fn test_scale(){
        let s = "((A[&location=UK]:0.1,B[&location=USA]:0.1)[&location=UK]:0.1,'C d'[&location=US]:0.1)[&location=US];";
        let mut tree = NewickImporter::read_tree(BufReader::new(s.as_bytes())).expect("error in parsing");
        scale(&mut tree, 10.0);
        for i in 0..tree.get_node_count(){
            if i!=tree.get_root().unwrap(){
                assert_eq!(Some(1.0), tree.get_length(i));
            }
        }
    }

    #[test]
    fn test_min(){
        let s = "((A[&location=UK]:0.1,B[&location=USA]:0.1)[&location=UK]:0.1,'C d'[&location=US]:0.1)[&location=US];";
        let mut tree = NewickImporter::read_tree(BufReader::new(s.as_bytes())).expect("error in parsing");
        min_length(&mut tree, 10.0);
        for i in 0..tree.get_node_count(){
            if i!=tree.get_root().unwrap(){
                assert_eq!(Some(10.0), tree.get_length(i));
            }
        }
    }
    #[test]
    fn test_round(){
        let s = "((A[&location=UK]:1.1,B[&location=USA]:1.1)[&location=UK]:1.1,'C d'[&location=US]:1.1)[&location=US];";
        let mut tree = NewickImporter::read_tree(BufReader::new(s.as_bytes())).expect("error in parsing");
        round(&mut tree);
        for i in 0..tree.get_node_count(){
            if i!=tree.get_root().unwrap(){
                assert_eq!(Some(1.0), tree.get_length(i));
            }
        }
    }

    #[test]
    fn test_nt(){
        let s = "((A[&mutations=\"\"]:1.1,B[&mutations=\"A1G,C7G\"]:1.1):1.1,('C'[&mutations=\"T898G\"]:1.1,D[&mutations=\"T898G,A892-\"]:0):0);";
        let mut tree = NewickImporter::read_tree(BufReader::new(s.as_bytes())).expect("error in parsing");
        tree_time(&mut tree, &TreeTimeSubCommands::Nucleotide);
        assert_eq!(Some(0.0), tree.get_length(tree.get_taxon_node("A").unwrap()));
        assert_eq!(Some(2.0), tree.get_length(tree.get_taxon_node("B").unwrap()));
        assert_eq!(Some(1.0), tree.get_length(tree.get_taxon_node("C").unwrap()));
        assert_eq!(Some(1.0), tree.get_length(tree.get_taxon_node("D").unwrap()));

    }
    #[test]
    fn test_all(){
        let s = "((A[&mutations=\"\"]:1.1,B[&mutations=\"A1G,C7G\"]:1.1):1.1,('C'[&mutations=\"T898G\"]:1.1,D[&mutations=\"T898G,A892-\"]:0):0);";
        let mut tree = NewickImporter::read_tree(BufReader::new(s.as_bytes())).expect("error in parsing");
        tree_time(&mut tree, &TreeTimeSubCommands::All);
        assert_eq!(Some(0.0), tree.get_length(tree.get_taxon_node("A").unwrap()));
        assert_eq!(Some(2.0), tree.get_length(tree.get_taxon_node("B").unwrap()));
        assert_eq!(Some(1.0), tree.get_length(tree.get_taxon_node("C").unwrap()));
        assert_eq!(Some(2.0), tree.get_length(tree.get_taxon_node("D").unwrap()));
    }
    #[test]
    fn test_tv(){
        let s = "((A[&mutations=\"\"]:1.1,B[&mutations=\"A1G,C7G\"]:1.1):1.1,('C'[&mutations=\"T898G\"]:1.1,D[&mutations=\"T898G,A892-\"]:0):0);";
        let mut tree = NewickImporter::read_tree(BufReader::new(s.as_bytes())).expect("error in parsing");
        tree_time(&mut tree, &TreeTimeSubCommands::Transversions);
        assert_eq!(Some(0.0), tree.get_length(tree.get_taxon_node("A").unwrap()));
        assert_eq!(Some(1.0), tree.get_length(tree.get_taxon_node("B").unwrap()));
        assert_eq!(Some(1.0), tree.get_length(tree.get_taxon_node("C").unwrap()));
        assert_eq!(Some(1.0), tree.get_length(tree.get_taxon_node("D").unwrap()));
    }
    #[test]
    fn test_ts(){
        let s = "((A[&mutations=\"\"]:1.1,B[&mutations=\"A1G,C7G\"]:1.1):1.1,('C'[&mutations=\"T898G\"]:1.1,D[&mutations=\"T898G,A892-\"]:0):0);";
        let mut tree = NewickImporter::read_tree(BufReader::new(s.as_bytes())).expect("error in parsing");
        tree_time(&mut tree, &TreeTimeSubCommands::Transitions);
        assert_eq!(Some(0.0), tree.get_length(tree.get_taxon_node("A").unwrap()));
        assert_eq!(Some(1.0), tree.get_length(tree.get_taxon_node("B").unwrap()));
        assert_eq!(Some(0.0), tree.get_length(tree.get_taxon_node("C").unwrap()));
        assert_eq!(Some(0.0), tree.get_length(tree.get_taxon_node("D").unwrap()));
    }

}