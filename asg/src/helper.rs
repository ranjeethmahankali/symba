use crate::tree::{BinaryOp, Node, Node::*, Tree, UnaryOp};

impl Into<Tree> for Node {
    fn into(self) -> Tree {
        Tree::new(self)
    }
}

impl From<f64> for Tree {
    fn from(value: f64) -> Self {
        return Constant(value).into();
    }
}

impl From<f64> for Node {
    fn from(value: f64) -> Self {
        return Constant(value);
    }
}

impl From<char> for Node {
    fn from(value: char) -> Self {
        return Symbol(value);
    }
}

impl From<char> for Tree {
    fn from(c: char) -> Self {
        return Symbol(c).into();
    }
}

impl std::fmt::Display for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const BRANCH: &str = " ├── ";
        const BYPASS: &str = " │   ";
        write!(f, "\n")?;
        let mut depths: Box<[usize]> = vec![0; self.len()].into_boxed_slice();
        let mut walker = DepthWalker::new();
        for (index, parent) in walker.walk_tree(self, false, NodeOrdering::Original) {
            if let Some(pi) = parent {
                depths[index] = depths[pi] + 1;
            }
            let depth = depths[index];
            for d in 0..depth {
                write!(f, "{}", {
                    if d < depth - 1 {
                        BYPASS
                    } else {
                        BRANCH
                    }
                })?;
            }
            writeln!(f, "[{}] {}", index, self.node(index))?;
        }
        write!(f, "\n")
    }
}

impl UnaryOp {
    pub fn index(&self) -> u8 {
        use UnaryOp::*;
        match self {
            Negate => 0,
            Sqrt => 1,
            Abs => 2,
            Sin => 3,
            Cos => 4,
            Tan => 5,
            Log => 6,
            Exp => 7,
        }
    }
}

impl BinaryOp {
    pub fn index(&self) -> u8 {
        use BinaryOp::*;
        match self {
            Add => 0,
            Subtract => 1,
            Multiply => 2,
            Divide => 3,
            Pow => 4,
            Min => 5,
            Max => 6,
        }
    }
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Constant(value) => write!(f, "Constant({})", value),
            Symbol(label) => write!(f, "Symbol({})", label),
            Unary(op, input) => write!(f, "{:?}({})", op, input),
            Binary(op, lhs, rhs) => write!(f, "{:?}({}, {})", op, lhs, rhs),
        }
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering::*;
        match (self, other) {
            // Constant
            (Constant(a), Constant(b)) => a.partial_cmp(b),
            (Constant(_), Symbol(_)) => Some(Less),
            (Constant(_), Unary(_, _)) => Some(Less),
            (Constant(_), Binary(_, _, _)) => Some(Less),
            // Symbol
            (Symbol(_), Constant(_)) => Some(Greater),
            (Symbol(a), Symbol(b)) => Some(a.cmp(b)),
            (Symbol(_), Unary(_, _)) => Some(Less),
            (Symbol(_), Binary(_, _, _)) => Some(Less),
            // Unary
            (Unary(_, _), Constant(_)) => Some(Greater),
            (Unary(_, _), Symbol(_)) => Some(Greater),
            (Unary(op1, _), Unary(op2, _)) => Some(op1.index().cmp(&op2.index())),
            (Unary(_, _), Binary(_, _, _)) => Some(Less),
            // Binary
            (Binary(_, _, _), Constant(_)) => Some(Greater),
            (Binary(_, _, _), Symbol(_)) => Some(Greater),
            (Binary(_, _, _), Unary(_, _)) => Some(Greater),
            (Binary(op1, _, _), Binary(op2, _, _)) => Some(op1.index().cmp(&op2.index())),
        }
    }
}

pub fn eq_recursive(
    nodes: &[Node],
    li: usize,
    ri: usize,
    walker1: &mut DepthWalker,
    walker2: &mut DepthWalker,
) -> bool {
    {
        use crate::helper::NodeOrdering::*;
        // Zip the depth first iterators and compare.
        let mut iter1 = walker1.walk_nodes(&nodes, li, false, Sorted);
        let mut iter2 = walker2.walk_nodes(&nodes, ri, false, Sorted);
        loop {
            let (left, right) = (iter1.next(), iter2.next());
            match (left, right) {
                (None, None) => {
                    // Both iterators ended.
                    return true;
                }
                (None, Some(_)) | (Some(_), None) => {
                    // One of the iterators ended prematurely.
                    return false;
                }
                (Some((i1, _p1)), Some((i2, _p2))) => {
                    if i1 == i2 {
                        iter1.skip_children();
                        iter2.skip_children();
                        continue;
                    }
                    if !match (nodes[i1], nodes[i2]) {
                        (Constant(v1), Constant(v2)) => v1 == v2,
                        (Symbol(c1), Symbol(c2)) => c1 == c2,
                        (Unary(op1, _input1), Unary(op2, _input2)) => op1 == op2,
                        (Binary(op1, _lhs1, _rhs1), Binary(op2, _lhs2, _rhs2)) => op1 == op2,
                        _ => false,
                    } {
                        return false;
                    }
                }
            }
        }
    }
}

pub struct DepthWalker {
    stack: Vec<(usize, Option<usize>)>,
    visited: Vec<bool>,
}

pub enum NodeOrdering {
    Original,
    Sorted,
}

impl DepthWalker {
    pub fn new() -> DepthWalker {
        DepthWalker {
            stack: vec![],
            visited: vec![],
        }
    }

    pub fn walk_tree<'a>(
        &'a mut self,
        tree: &'a Tree,
        unique: bool,
        ordering: NodeOrdering,
    ) -> DepthIterator<'a> {
        self.walk_nodes(&tree.nodes(), tree.root_index(), unique, ordering)
    }

    pub fn walk_nodes<'a>(
        &'a mut self,
        nodes: &'a [Node],
        root_index: usize,
        unique: bool,
        ordering: NodeOrdering,
    ) -> DepthIterator<'a> {
        // Prep the stack.
        self.stack.clear();
        self.stack.reserve(nodes.len());
        self.stack.push((root_index, None));
        // Reset the visited flags.
        self.visited.clear();
        self.visited.resize(nodes.len(), false);
        // Create the iterator.
        DepthIterator {
            unique,
            ordering,
            walker: self,
            nodes: &nodes,
            last_pushed: 0,
        }
    }
}

pub struct DepthIterator<'a> {
    unique: bool,
    ordering: NodeOrdering,
    last_pushed: usize,
    walker: &'a mut DepthWalker,
    nodes: &'a [Node],
}

impl<'a> DepthIterator<'a> {
    fn sort_children(&self, children: &mut [usize]) {
        use std::cmp::Ordering;
        use NodeOrdering::*;
        match &self.ordering {
            Original => {} // Do nothing.
            Sorted => children.sort_by(|a, b| match self.nodes[*a].partial_cmp(&self.nodes[*b]) {
                Some(ord) => ord,
                // Assuming the only time we return None is with two
                // constant nodes with Nan's in them. This seems like
                // a harmless edge case for now. Specially given we
                // don't allow the construction of trees with Nan
                // constant nodes.
                None => Ordering::Equal,
            }),
        };
    }

    pub fn skip_children(&mut self) {
        for _ in 0..self.last_pushed {
            self.walker.stack.pop();
        }
    }
}

impl<'a> Iterator for DepthIterator<'a> {
    type Item = (usize, Option<usize>);

    fn next(&mut self) -> Option<Self::Item> {
        let (index, parent) = {
            // Pop the stack until we find a node we didn't already visit.
            let (mut i, mut p) = self.walker.stack.pop()?;
            while self.unique && self.walker.visited[i] {
                (i, p) = self.walker.stack.pop()?;
            }
            (i, p)
        };
        // Push the children on to the stack.
        match &self.nodes[index] {
            Constant(_) | Symbol(_) => {
                self.last_pushed = 0;
            }
            Unary(_op, input) => {
                self.walker.stack.push((*input, Some(index)));
                self.last_pushed = 1;
            }
            Binary(_op, lhs, rhs) => {
                // Pushing them rhs first because last in first out.
                let mut children = [*rhs, *lhs];
                // Sort according to the requested ordering.
                self.sort_children(&mut children);
                for child in children {
                    self.walker.stack.push((child, Some(index)));
                }
                self.last_pushed = children.len();
            }
        }
        self.walker.visited[index] = true;
        return Some((index, parent));
    }
}

pub struct Trimmer {
    indices: Vec<(bool, usize)>,
    trimmed: Vec<Node>,
}

impl Trimmer {
    pub fn new() -> Trimmer {
        Trimmer {
            indices: vec![],
            trimmed: vec![],
        }
    }

    pub fn trim(
        &mut self,
        mut nodes: Vec<Node>,
        root_index: usize,
        walker: &mut DepthWalker,
    ) -> Vec<Node> {
        self.indices.clear();
        self.indices.resize(nodes.len(), (false, 0));
        // Mark used nodes.
        walker
            .walk_nodes(&nodes, root_index, true, NodeOrdering::Original)
            .for_each(|(index, _parent)| {
                self.indices[index] = (true, 1usize);
            });
        // Do exclusive scan.
        let mut sum = 0usize;
        for pair in self.indices.iter_mut() {
            let (keep, i) = *pair;
            let copy = sum;
            sum += i;
            *pair = (keep, copy);
        }
        // Filter, update and copy nodes.
        self.trimmed.reserve(nodes.len());
        self.trimmed.extend(
            (0..self.indices.len())
                .zip(nodes.iter())
                .filter(|(i, _node)| {
                    let (keep, _index) = self.indices[*i];
                    return keep;
                })
                .map(|(_i, node)| {
                    match node {
                        // Update the indices of this node's inputs.
                        Constant(val) => Constant(*val),
                        Symbol(label) => Symbol(*label),
                        Unary(op, input) => Unary(*op, self.indices[*input].1),
                        Binary(op, lhs, rhs) => {
                            Binary(*op, self.indices[*lhs].1, self.indices[*rhs].1)
                        }
                    }
                }),
        );
        std::mem::swap(&mut self.trimmed, &mut nodes);
        return nodes;
    }
}
