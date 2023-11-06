use lazy_static::lazy_static;

use crate::{
    deftemplate,
    parser::parse_template,
    tree::{Node, Tree, TreeError},
};

pub struct Template {
    ping: Vec<Node>,
    pong: Vec<Node>,
}

impl Template {
    pub fn from(ping: Vec<Node>, pong: Vec<Node>) -> Result<Template, TreeError> {
        Ok(Template {
            ping: Tree::validate(ping)?,
            pong: Tree::validate(pong)?,
        })
    }

    pub fn mirror_templates(mut templates: Vec<Template>) -> Vec<Template> {
        let num = templates.len();
        for i in 0..num {
            let t = &templates[i];
            templates.push(Template {
                ping: t.pong.clone(),
                pong: t.ping.clone(),
            });
        }
        return templates;
    }
}

lazy_static! {
    static ref TEMPLATES: Vec<Template> = Template::mirror_templates(vec![

        // Factoring a multiplication out of addition.
        deftemplate!(
            (_ping (+ (* k a) (* k b))
             _pong (* k (+ a b)))
        ).unwrap(),
        // Min of two square-roots.
        deftemplate!(
            (_ping (min (sqrt a) (sqrt b))
             _pong (sqrt (min a b)))
        ).unwrap(),
        // Interchangeable fractions.
        deftemplate!(
            (_ping (* (/ a b) (/ x y))
             _pong (* (/ a y) (/ x b)))
        ).unwrap(),
        // Cancelling division.
        deftemplate!(
            (_ping (/ a a)
             _pong 1.0)
        ).unwrap(),
        // Distributing pow over division.
        deftemplate!(
            (_ping (pow (/ a b) 2.)
             _pong (/ (pow a 2.) (pow b 2.)))
        ).unwrap(),
        // Distribute pow over multiplication.
        deftemplate!(
            (_ping (pow (* a b) 2.)
             _pong (* (pow a 2.) (pow b 2.)))
        ).unwrap(),
        // Square of square-root.
        deftemplate!(
            (_ping (pow (sqrt a) 2.)
             _pong a)
        ).unwrap(),
        // Square root of square.
        deftemplate!(
            (_ping (sqrt (pow a 2.))
             _pong a)
        ).unwrap(),
        // Combine exponents.
        deftemplate!(
            (_ping (pow (pow a x) y)
             _pong (pow a (* x y)))
        ).unwrap(),
        // Adding fractions.
        deftemplate!(
            (_ping (+ (/ a d) (/ b d))
             _pong (/ (+ a b) d))
        ).unwrap(),

        // ====== Identity operations ======

        // Add zero.
        deftemplate!(
            (_ping (+ x 0.)
             _pong x)
        ).unwrap(),
        // Subtract zero.
        deftemplate!(
          (_ping (- x 0) _pong x)
        ).unwrap(),
        // Multiply by 1.
        deftemplate!(
          (_ping (* x 1.) _pong x)
        ).unwrap(),
        // Raised to the power of 1.
        deftemplate!(
            (_ping (pow x 1.) _pong x)
        ).unwrap(),

        // ====== Other templates =======

        // Multiply by zero.
        deftemplate!(
            (_ping (* x 0.) _pong 0.)
        ).unwrap(),
        // Raised to the power of zero.
        deftemplate!(
            (_ping (pow x 0.) _pong 1.)
        ).unwrap(),
        // Min and max simplifications from:
        // https://math.stackexchange.com/questions/1195917/simplifying-a-function-that-has-max-and-min-expressions
        deftemplate!( // Min
            (_ping (min a b)
             _pong (/ (+ (+ a b) (abs (- b a))) 2.))
        ).unwrap(),
        deftemplate!( // Max
            (_ping (min a b)
             _pong (/ (- (+ a b) (abs (- b a))) 2.))
        ).unwrap(),
    ]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_templates() {
        // Just to make sure all the templates are valid and load
        // correctly.
        assert!(!TEMPLATES.is_empty());
        assert!(TEMPLATES.len() >= 36);
    }
}
