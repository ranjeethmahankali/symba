use crate::tree::{Node, Node::*, Tree};

pub fn hash_nodes(nodes: &[Node], hashbuf: &mut Vec<u64>) {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    // Using a boxed slice to avoid accidental resizing later.
    hashbuf.clear();
    hashbuf.resize(nodes.len(), 0);
    for index in 0..nodes.len() {
        let hash: u64 = match nodes[index] {
            Constant(value) => value.to_bits().into(),
            Symbol(label) => {
                let mut s: DefaultHasher = Default::default();
                label.hash(&mut s);
                s.finish()
            }
            Unary(op, input) => {
                let mut s: DefaultHasher = Default::default();
                op.hash(&mut s);
                hashbuf[input].hash(&mut s);
                s.finish()
            }
            Binary(op, lhs, rhs) => {
                let (hash1, hash2) = {
                    let mut hash1 = hashbuf[lhs];
                    let mut hash2 = hashbuf[rhs];
                    if op.is_commutative() && hash1 > hash2 {
                        (hash1, hash2) = (hash2, hash1); // For determinism.
                    }
                    (hash1, hash2)
                };
                let mut s: DefaultHasher = Default::default();
                op.hash(&mut s);
                hash1.hash(&mut s);
                hash2.hash(&mut s);
                s.finish()
            }
        };
        hashbuf[index] = hash;
    }
}

impl Tree {
    pub fn hash(&self, hashbuf: &mut Vec<u64>) -> u64 {
        hash_nodes(self.nodes(), hashbuf);
        return hashbuf[self.root_index()];
    }
}
