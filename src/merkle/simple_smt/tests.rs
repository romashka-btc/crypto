use super::{
    super::{int_to_node, InnerNodeInfo, MerkleError, MerkleTree, RpoDigest, SimpleSmt},
    NodeIndex, Rpo256, Vec, Word, EMPTY_WORD,
};

// TEST DATA
// ================================================================================================

const KEYS4: [u64; 4] = [0, 1, 2, 3];
const KEYS8: [u64; 8] = [0, 1, 2, 3, 4, 5, 6, 7];

const VALUES4: [Word; 4] = [int_to_node(1), int_to_node(2), int_to_node(3), int_to_node(4)];

const VALUES8: [Word; 8] = [
    int_to_node(1),
    int_to_node(2),
    int_to_node(3),
    int_to_node(4),
    int_to_node(5),
    int_to_node(6),
    int_to_node(7),
    int_to_node(8),
];

const ZERO_VALUES8: [Word; 8] = [int_to_node(0); 8];

// TESTS
// ================================================================================================

#[test]
fn build_empty_tree() {
    // tree of depth 3
    let smt = SimpleSmt::new(3).unwrap();
    let mt = MerkleTree::new(ZERO_VALUES8.to_vec()).unwrap();
    assert_eq!(mt.root(), smt.root());
}

#[test]
fn build_sparse_tree() {
    let mut smt = SimpleSmt::new(3).unwrap();
    let mut values = ZERO_VALUES8.to_vec();

    // insert single value
    let key = 6;
    let new_node = int_to_node(7);
    values[key as usize] = new_node;
    let old_value = smt.update_leaf(key, new_node).expect("Failed to update leaf");
    let mt2 = MerkleTree::new(values.clone()).unwrap();
    assert_eq!(mt2.root(), smt.root());
    assert_eq!(
        mt2.get_path(NodeIndex::make(3, 6)).unwrap(),
        smt.get_path(NodeIndex::make(3, 6)).unwrap()
    );
    assert_eq!(old_value, EMPTY_WORD);

    // insert second value at distinct leaf branch
    let key = 2;
    let new_node = int_to_node(3);
    values[key as usize] = new_node;
    let old_value = smt.update_leaf(key, new_node).expect("Failed to update leaf");
    let mt3 = MerkleTree::new(values).unwrap();
    assert_eq!(mt3.root(), smt.root());
    assert_eq!(
        mt3.get_path(NodeIndex::make(3, 2)).unwrap(),
        smt.get_path(NodeIndex::make(3, 2)).unwrap()
    );
    assert_eq!(old_value, EMPTY_WORD);
}

#[test]
fn test_depth2_tree() {
    let tree = SimpleSmt::with_leaves(2, KEYS4.into_iter().zip(VALUES4.into_iter())).unwrap();

    // check internal structure
    let (root, node2, node3) = compute_internal_nodes();
    assert_eq!(root, tree.root());
    assert_eq!(node2, tree.get_node(NodeIndex::make(1, 0)).unwrap());
    assert_eq!(node3, tree.get_node(NodeIndex::make(1, 1)).unwrap());

    // check get_node()
    assert_eq!(VALUES4[0], tree.get_node(NodeIndex::make(2, 0)).unwrap());
    assert_eq!(VALUES4[1], tree.get_node(NodeIndex::make(2, 1)).unwrap());
    assert_eq!(VALUES4[2], tree.get_node(NodeIndex::make(2, 2)).unwrap());
    assert_eq!(VALUES4[3], tree.get_node(NodeIndex::make(2, 3)).unwrap());

    // check get_path(): depth 2
    assert_eq!(vec![VALUES4[1], node3], *tree.get_path(NodeIndex::make(2, 0)).unwrap());
    assert_eq!(vec![VALUES4[0], node3], *tree.get_path(NodeIndex::make(2, 1)).unwrap());
    assert_eq!(vec![VALUES4[3], node2], *tree.get_path(NodeIndex::make(2, 2)).unwrap());
    assert_eq!(vec![VALUES4[2], node2], *tree.get_path(NodeIndex::make(2, 3)).unwrap());

    // check get_path(): depth 1
    assert_eq!(vec![node3], *tree.get_path(NodeIndex::make(1, 0)).unwrap());
    assert_eq!(vec![node2], *tree.get_path(NodeIndex::make(1, 1)).unwrap());
}

#[test]
fn test_inner_node_iterator() -> Result<(), MerkleError> {
    let tree = SimpleSmt::with_leaves(2, KEYS4.into_iter().zip(VALUES4.into_iter())).unwrap();

    // check depth 2
    assert_eq!(VALUES4[0], tree.get_node(NodeIndex::make(2, 0)).unwrap());
    assert_eq!(VALUES4[1], tree.get_node(NodeIndex::make(2, 1)).unwrap());
    assert_eq!(VALUES4[2], tree.get_node(NodeIndex::make(2, 2)).unwrap());
    assert_eq!(VALUES4[3], tree.get_node(NodeIndex::make(2, 3)).unwrap());

    // get parent nodes
    let root = tree.root();
    let l1n0 = tree.get_node(NodeIndex::make(1, 0))?;
    let l1n1 = tree.get_node(NodeIndex::make(1, 1))?;
    let l2n0 = tree.get_node(NodeIndex::make(2, 0))?;
    let l2n1 = tree.get_node(NodeIndex::make(2, 1))?;
    let l2n2 = tree.get_node(NodeIndex::make(2, 2))?;
    let l2n3 = tree.get_node(NodeIndex::make(2, 3))?;

    let nodes: Vec<InnerNodeInfo> = tree.inner_nodes().collect();
    let expected = vec![
        InnerNodeInfo {
            value: root.into(),
            left: l1n0.into(),
            right: l1n1.into(),
        },
        InnerNodeInfo {
            value: l1n0.into(),
            left: l2n0.into(),
            right: l2n1.into(),
        },
        InnerNodeInfo {
            value: l1n1.into(),
            left: l2n2.into(),
            right: l2n3.into(),
        },
    ];
    assert_eq!(nodes, expected);

    Ok(())
}

#[test]
fn update_leaf() {
    let mut tree = SimpleSmt::with_leaves(3, KEYS8.into_iter().zip(VALUES8.into_iter())).unwrap();

    // update one value
    let key = 3;
    let new_node = int_to_node(9);
    let mut expected_values = VALUES8.to_vec();
    expected_values[key] = new_node;
    let expected_tree = MerkleTree::new(expected_values.clone()).unwrap();

    let old_leaf = tree.update_leaf(key as u64, new_node).unwrap();
    assert_eq!(expected_tree.root(), tree.root);
    assert_eq!(old_leaf, VALUES8[key]);

    // update another value
    let key = 6;
    let new_node = int_to_node(10);
    expected_values[key] = new_node;
    let expected_tree = MerkleTree::new(expected_values.clone()).unwrap();

    let old_leaf = tree.update_leaf(key as u64, new_node).unwrap();
    assert_eq!(expected_tree.root(), tree.root);
    assert_eq!(old_leaf, VALUES8[key]);
}

#[test]
fn small_tree_opening_is_consistent() {
    //        ____k____
    //       /         \
    //     _i_         _j_
    //    /   \       /   \
    //   e     f     g     h
    //  / \   / \   / \   / \
    // a   b 0   0 c   0 0   d

    let z = Word::from(RpoDigest::default());

    let a = Word::from(Rpo256::merge(&[z.into(); 2]));
    let b = Word::from(Rpo256::merge(&[a.into(); 2]));
    let c = Word::from(Rpo256::merge(&[b.into(); 2]));
    let d = Word::from(Rpo256::merge(&[c.into(); 2]));

    let e = Word::from(Rpo256::merge(&[a.into(), b.into()]));
    let f = Word::from(Rpo256::merge(&[z.into(), z.into()]));
    let g = Word::from(Rpo256::merge(&[c.into(), z.into()]));
    let h = Word::from(Rpo256::merge(&[z.into(), d.into()]));

    let i = Word::from(Rpo256::merge(&[e.into(), f.into()]));
    let j = Word::from(Rpo256::merge(&[g.into(), h.into()]));

    let k = Word::from(Rpo256::merge(&[i.into(), j.into()]));

    let depth = 3;
    let entries = vec![(0, a), (1, b), (4, c), (7, d)];
    let tree = SimpleSmt::with_leaves(depth, entries).unwrap();

    assert_eq!(tree.root(), Word::from(k));

    let cases: Vec<(u8, u64, Vec<Word>)> = vec![
        (3, 0, vec![b, f, j]),
        (3, 1, vec![a, f, j]),
        (3, 4, vec![z, h, i]),
        (3, 7, vec![z, g, i]),
        (2, 0, vec![f, j]),
        (2, 1, vec![e, j]),
        (2, 2, vec![h, i]),
        (2, 3, vec![g, i]),
        (1, 0, vec![j]),
        (1, 1, vec![i]),
    ];

    for (depth, key, path) in cases {
        let opening = tree.get_path(NodeIndex::make(depth, key)).unwrap();

        assert_eq!(path, *opening);
    }
}

#[test]
fn fail_on_duplicates() {
    let entries = [(1_u64, int_to_node(1)), (5, int_to_node(2)), (1_u64, int_to_node(3))];
    let smt = SimpleSmt::with_leaves(64, entries);
    assert!(smt.is_err());

    let entries = [(1_u64, int_to_node(0)), (5, int_to_node(2)), (1_u64, int_to_node(0))];
    let smt = SimpleSmt::with_leaves(64, entries);
    assert!(smt.is_err());

    let entries = [(1_u64, int_to_node(0)), (5, int_to_node(2)), (1_u64, int_to_node(1))];
    let smt = SimpleSmt::with_leaves(64, entries);
    assert!(smt.is_err());

    let entries = [(1_u64, int_to_node(1)), (5, int_to_node(2)), (1_u64, int_to_node(0))];
    let smt = SimpleSmt::with_leaves(64, entries);
    assert!(smt.is_err());
}

#[test]
fn with_no_duplicates_empty_node() {
    let entries = [(1_u64, int_to_node(0)), (5, int_to_node(2))];
    let smt = SimpleSmt::with_leaves(64, entries);
    assert!(smt.is_ok());
}

// HELPER FUNCTIONS
// --------------------------------------------------------------------------------------------

fn compute_internal_nodes() -> (Word, Word, Word) {
    let node2 = Rpo256::hash_elements(&[VALUES4[0], VALUES4[1]].concat());
    let node3 = Rpo256::hash_elements(&[VALUES4[2], VALUES4[3]].concat());
    let root = Rpo256::merge(&[node2, node3]);

    (root.into(), node2.into(), node3.into())
}
