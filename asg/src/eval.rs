use crate::tree::{Node::*, Tree};

/// Errors that can occur when evaluating a tree.
#[derive(Debug)]
pub enum EvaluationError {
    /// A symbol was not assigned a value before evaluating.
    VariableNotFound(char),
    /// A register with uninitialized value was encountered during
    /// evaluation. This could mean the topology of the tree is
    /// broken.
    UninitializedValueRead,
}

/// This can be used to compute the value(s) of the tree.
pub struct Evaluator<'a> {
    tree: &'a Tree,
    regs: Box<[Option<f64>]>,
}

impl<'a> Evaluator<'a> {
    /// Create a new evaluator for `tree`.
    pub fn new(tree: &'a Tree) -> Evaluator {
        Evaluator {
            tree,
            regs: vec![None; tree.len()].into_boxed_slice(),
        }
    }

    /// Set all symbols in the evaluator matching `label` to
    /// `value`. This `value` will be used for all future evaluations,
    /// unless this function is called again with a different `value`.
    pub fn set_var(&mut self, label: char, value: f64) {
        for (node, reg) in self.tree.nodes().iter().zip(self.regs.iter_mut()) {
            match node {
                Symbol(l) if *l == label => {
                    *reg = Some(value);
                }
                _ => {}
            }
        }
    }

    /// Read the value from the `index`-th register. Returns an error
    /// if the register doesn't contain a value.
    fn read(&self, index: usize) -> Result<f64, EvaluationError> {
        match self.regs[index] {
            Some(val) => Ok(val),
            None => Err(EvaluationError::UninitializedValueRead),
        }
    }

    /// Write the `value` into the `index`-th register. The existing
    /// value is overwritten.
    fn write(&mut self, index: usize, value: f64) {
        self.regs[index] = Some(value);
    }

    /// Run the evaluator and return the result. The result may
    /// contain the output value, or an
    /// error. `Variablenotfound(label)` error means the variable
    /// matching `label` hasn't been assigned a value using `set_var`.
    pub fn run(&mut self) -> Result<f64, EvaluationError> {
        for idx in 0..self.tree.len() {
            self.write(
                idx,
                match &self.tree.node(idx) {
                    Constant(val) => *val,
                    Symbol(label) => match &self.regs[idx] {
                        None => return Err(EvaluationError::VariableNotFound(*label)),
                        Some(val) => *val,
                    },
                    Binary(op, lhs, rhs) => op.apply(self.read(*lhs)?, self.read(*rhs)?),
                    Unary(op, input) => op.apply(self.read(*input)?),
                },
            );
        }
        return self.read(self.tree.root_index());
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::deftree;
    use crate::test::util::{assert_float_eq, check_tree_eval, compare_trees};
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn t_variable_in_deftree() {
        let lisp = deftree!(+ 1. (+ (cos x) (pow (cos x) 2.)));
        let cx = deftree!(cos x);
        let with_vars = deftree!(+ 1. (+ {cx.clone()} (pow {cx} 2.)));
        assert_eq!(lisp, with_vars);
        compare_trees(&lisp, &with_vars, &[('x', -5., 5.)], 100, 0.);
        // More complex expressions.
        use crate::tree::pow;
        let tree: Tree = deftree!(
            (+
             {
                 let three: Tree = 3.0.into();
                 three * pow('x'.into(), 2.0.into())
             }
             (+ (* 2. x) 1.))
        );
        let expected = deftree!(+ (* 3. (pow x 2.)) (+ (* 2. x) 1.));
        assert_eq!(tree, expected);
        compare_trees(&expected, &tree, &[('x', -5., 5.)], 100, 0.);
    }

    #[test]
    fn t_constant() {
        let x = deftree!(const std::f64::consts::PI);
        assert_eq!(x.root(), &Constant(std::f64::consts::PI));
        let mut eval = Evaluator::new(&x);
        match eval.run() {
            Ok(val) => assert_eq!(val, std::f64::consts::PI),
            _ => assert!(false),
        }
    }

    #[test]
    fn t_pythagoras() {
        const TRIPLETS: [(f64, f64, f64); 6] = [
            (3., 4., 5.),
            (5., 12., 13.),
            (8., 15., 17.),
            (7., 24., 25.),
            (20., 21., 29.),
            (12., 35., 37.),
        ];
        let h = deftree!(sqrt (+ (pow x 2.) (pow y 2.)));
        let mut eval = Evaluator::new(&h);
        for (x, y, expected) in TRIPLETS {
            eval.set_var('x', x);
            eval.set_var('y', y);
            match eval.run() {
                Ok(val) => assert_eq!(val, expected),
                _ => assert!(false),
            }
        }
    }

    #[test]
    fn t_trig_identity() {
        use rand::Rng;
        const PI_2: f64 = 2.0 * std::f64::consts::TAU;
        let sum = deftree!(+ (pow (sin x) 2.) (pow (cos x) 2.));
        let mut eval = Evaluator::new(&sum);
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..100 {
            let x: f64 = PI_2 * rng.gen::<f64>();
            eval.set_var('x', x);
            match eval.run() {
                Ok(val) => assert_float_eq!(val, 1.),
                _ => assert!(false),
            }
        }
    }

    #[test]
    fn t_sum_test() {
        check_tree_eval(
            deftree!(+ x y),
            |vars: &[f64]| {
                if let [x, y] = vars[..] {
                    Some(x + y)
                } else {
                    None
                }
            },
            &[('x', -5., 5.), ('y', -5., 5.)],
            10,
            0.,
        );
    }

    #[test]
    fn t_evaluate_trees_1() {
        check_tree_eval(
            deftree!(/ (pow (log (+ (sin x) 2.)) 3.) (+ (cos x) 2.)),
            |vars: &[f64]| {
                if let [x] = vars[..] {
                    Some(
                        f64::powf(f64::log(f64::sin(x) + 2., std::f64::consts::E), 3.)
                            / (f64::cos(x) + 2.),
                    )
                } else {
                    None
                }
            },
            &[('x', -2.5, 2.5)],
            100,
            0.,
        );
    }

    #[test]
    fn t_evaluate_trees_2() {
        check_tree_eval(
            deftree!(
                (max (min
                      (- (sqrt (+ (+ (pow (- x 2.) 2.) (pow (- y 3.) 2.)) (pow (- z 4.) 2.))) 2.75)
                      (- (sqrt (+ (+ (pow (+ x 2.) 2.) (pow (- y 3.) 2.)) (pow (- z 4.) 2.))) 4.))
                 (- (sqrt (+ (+ (pow (+ x 2.) 2.) (pow (+ y 3.) 2.)) (pow (- z 4.) 2.))) 5.25))
            ),
            |vars: &[f64]| {
                if let [x, y, z] = vars[..] {
                    let s1 = f64::sqrt(
                        f64::powf(x - 2., 2.) + f64::powf(y - 3., 2.) + f64::powf(z - 4., 2.),
                    ) - 2.75;
                    let s2 = f64::sqrt(
                        f64::powf(x + 2., 2.) + f64::powf(y - 3., 2.) + f64::powf(z - 4., 2.),
                    ) - 4.;
                    let s3 = f64::sqrt(
                        f64::powf(x + 2., 2.) + f64::powf(y + 3., 2.) + f64::powf(z - 4., 2.),
                    ) - 5.25;
                    Some(f64::max(f64::min(s1, s2), s3))
                } else {
                    None
                }
            },
            &[('x', -10., 10.), ('y', -9., 10.), ('z', -11., 12.)],
            20,
            1e-14,
        );
    }
}
