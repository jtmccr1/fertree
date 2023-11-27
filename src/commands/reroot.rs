enum Subcommand {
    Midpoint,
    Outgroup,
    All,
}

fn midpoint(tree: &mut Tree) {
    let longest_branch = find_longest_branch(tree);
    reroot(tree, longest_branch);
}

fn find_longest_branch(tree: &Tree) -> &Branch {
    // implementation to find the longest branch
}

fn reroot(tree: &mut Tree, branch: &Branch) {
    // implementation to reroot the tree on the given branch
}

