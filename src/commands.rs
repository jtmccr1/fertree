
mod introductions{
    use rebl::tree::fixed_tree::FixedNode;
    struct TaxaIntroductionLabel{
        taxa:String,
        introduction:usize,
        tmrca:f64
    }
    fn label_introductions(tree:FixedNode){

    }
}

mod thin{
}

mod collapse{
    use rebl::tree::mutable_tree::MutableTree;

    pub fn collapse_uniform_clades(tree:&MutableTree,key:&String,ignore_missing:bool) {
        // Get clades that are all the same
        for node_ref in tree.preorder_iter().rev(){
            let node = tree.get_node(node_ref);

        }
        // sample 1 tip from those clades
        // remove all other taxa
        // output final tree


    }

}

mod annotate{
    use rebl::tree::mutable_tree::MutableTree;
    use std::collections::HashMap;
    use rebl::parsers::newick_parser::AnnotationValue;

    pub fn annotate_tips(mut tree:MutableTree, annotation_map:HashMap<String,HashMap<String,AnnotationValue>>){
        for taxon in annotation_map.keys(){
            let node_ref = tree.get_taxon_node(taxon).expect(&*("Taxon ".to_owned() + taxon + " not found in tree"));
            if let Some(annotations)=annotation_map.get(taxon){
                for (key,value) in annotations{
                    tree.annotate_node(&node_ref, key.clone(), value.clone())
                }
            }
        }
    }
}

mod split{
    
}