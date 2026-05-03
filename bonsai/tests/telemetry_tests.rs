use bonsai_bt::Behavior::Sequence;

#[derive(Clone, Debug)]
enum Act {
    A,
    B,
    C,
    D,
}

/// Verify that `build_node_metas` assigns correct subtree sizes and that the
/// preorder IDs align with `TreeDefinition::build` for the same tree.
///
/// Tree: Sequence([Action(A), Sequence([Action(B), Action(C)]), Action(D)])
///
///   id 0: outer Sequence   subtree_size = 6
///   id 1: Action(A)        subtree_size = 1
///   id 2: inner Sequence   subtree_size = 3
///   id 3: Action(B)        subtree_size = 1
///   id 4: Action(C)        subtree_size = 1
///   id 5: Action(D)        subtree_size = 1
#[test]
fn node_metas_subtree_sizes() {
    use bonsai_bt::Action;
    use Act::*;

    let behavior = Sequence(vec![
        Action(A),
        Sequence(vec![Action(B), Action(C)]),
        Action(D),
    ]);

    let metas = bonsai_bt::telemetry::build_node_metas(&behavior);

    assert_eq!(metas.len(), 6, "tree has 6 nodes");
    assert_eq!(metas[0].subtree_size, 6, "outer Sequence spans all 6 nodes");
    assert_eq!(metas[1].subtree_size, 1, "Action(A) is a leaf");
    assert_eq!(metas[2].subtree_size, 3, "inner Sequence spans 3 nodes");
    assert_eq!(metas[3].subtree_size, 1, "Action(B) is a leaf");
    assert_eq!(metas[4].subtree_size, 1, "Action(C) is a leaf");
    assert_eq!(metas[5].subtree_size, 1, "Action(D) is a leaf");
}

/// The preorder IDs produced by `build_node_metas` (implicit via Vec index)
/// must align with the explicit `id` fields produced by `TreeDefinition::build`
/// for the same tree.
#[test]
fn node_metas_ids_match_tree_definition() {
    use bonsai_bt::telemetry::{build_node_metas, TreeDefinition};
    use bonsai_bt::Action;
    use Act::*;

    let behavior = Sequence(vec![
        Action(A),
        Sequence(vec![Action(B), Action(C)]),
        Action(D),
    ]);

    let metas = build_node_metas(&behavior);
    let def = TreeDefinition::build(&behavior);

    // Collect node IDs from the tree definition in DFS preorder.
    fn collect_ids(node: &bonsai_bt::telemetry::TreeNode, ids: &mut Vec<usize>) {
        ids.push(node.id);
        for child in &node.children {
            collect_ids(child, ids);
        }
    }
    let mut ids = Vec::new();
    collect_ids(&def.root, &mut ids);

    assert_eq!(ids.len(), metas.len(), "same number of nodes");
    for (idx, id) in ids.iter().enumerate() {
        assert_eq!(*id, idx, "preorder index {idx} must equal TreeDefinition id");
    }
}
