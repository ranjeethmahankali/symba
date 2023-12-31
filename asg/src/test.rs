pub mod util {
    use crate::{eval::Evaluator, tree::Tree};
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    macro_rules! assert_float_eq {
        ($a:expr, $b:expr, $eps:expr) => {{
            // Make variables to avoid evaluating experssions multiple times.
            let a = $a;
            let b = $b;
            let eps = $eps;
            let error = f64::abs(a - b);
            assert!(
                error <= eps,
                "Assertion failed: |({}) - ({})| = {:e} < {:e}",
                a,
                b,
                error,
                eps
            );
        }};
        ($a:expr, $b:expr) => {
            assert_float_eq!($a, $b, f64::EPSILON)
        };
    }
    pub(crate) use assert_float_eq;

    /// Helper function to evaluate the tree with randomly sampled
    /// variable values and compare the result to the one returned by
    /// the `expectedfn` for the same inputs. The values must be
    /// within `eps` of each other.
    ///
    /// Each variable is sampled within the range indicated by the
    /// corresponding entry in `vardata`. Each entry in vardata
    /// consists of the label of the symbol / variable, lower bound
    /// and upper bound.
    pub fn check_tree_eval<F>(
        tree: Tree,
        mut expectedfn: F,
        vardata: &[(char, f64, f64)],
        samples_per_var: usize,
        eps: f64,
    ) where
        F: FnMut(&[f64]) -> Option<f64>,
    {
        use rand::Rng;
        let mut eval = Evaluator::new(&tree);
        let nvars = vardata.len();
        let mut indices = vec![0usize; nvars];
        let mut sample = Vec::<f64>::with_capacity(nvars);
        let mut rng = StdRng::seed_from_u64(42);
        while indices[0] <= samples_per_var {
            let vari = sample.len();
            let (label, lower, upper) = vardata[vari];
            let value = lower + rng.gen::<f64>() * (upper - lower);
            sample.push(value);
            eval.set_var(label, value);
            indices[vari] += 1;
            if vari < nvars - 1 {
                continue;
            }
            // We set all the variables. Run the test.
            assert_float_eq!(eval.run().unwrap(), expectedfn(&sample[..]).unwrap(), eps);
            // Clean up the index stack.
            sample.pop();
            let mut vari = vari;
            while indices[vari] == samples_per_var && vari > 0 {
                if let Some(_) = sample.pop() {
                    indices[vari] = 0;
                    vari -= 1;
                } else {
                    assert!(false); // To ensure the logic of this test is correct.
                }
            }
        }
    }

    pub fn compare_trees(
        tree1: &Tree,
        tree2: &Tree,
        vardata: &[(char, f64, f64)],
        samples_per_var: usize,
        eps: f64,
    ) {
        use rand::Rng;
        let mut eval1 = Evaluator::new(&tree1);
        let mut eval2 = Evaluator::new(&tree2);
        let nvars = vardata.len();
        let mut indices = vec![0usize; nvars];
        let mut sample = Vec::<f64>::with_capacity(nvars);
        let mut rng = StdRng::seed_from_u64(42);
        while indices[0] <= samples_per_var {
            let vari = sample.len();
            let (label, lower, upper) = vardata[vari];
            let value = lower + rng.gen::<f64>() * (upper - lower);
            sample.push(value);
            eval1.set_var(label, value);
            eval2.set_var(label, value);
            indices[vari] += 1;
            if vari < nvars - 1 {
                continue;
            }
            assert_float_eq!(eval1.run().unwrap(), eval2.run().unwrap(), eps);
            // Clean up the index stack.
            sample.pop();
            let mut vari = vari;
            while indices[vari] == samples_per_var && vari > 0 {
                if let Some(_) = sample.pop() {
                    indices[vari] = 0;
                    vari -= 1;
                } else {
                    assert!(false); // To ensure the logic of this test is correct.
                }
            }
        }
    }
}
