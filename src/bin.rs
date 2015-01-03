extern crate lux;

use std::rand::{XorShiftRng, SeedableRng};
use std::io::File;
use weighted_sample::WeightedSample;

mod weighted_sample;

enum Program {
    X, Y, T,

    Xor(Box<Program>, Box<Program>),
    And(Box<Program>, Box<Program>),
    Or(Box<Program>, Box<Program>),

    Add(Box<Program>, Box<Program>),
    Sub(Box<Program>, Box<Program>),
    Mul(Box<Program>, Box<Program>),
    Div(Box<Program>, Box<Program>),
}

type ProgramTripple = (Program, Program, Program);

impl Program {
    fn new_random(rng: &mut XorShiftRng) -> Program {
        fn gen(rng: &mut XorShiftRng) -> Program {
            Program::new_random(rng)
        }

        let mut ws = WeightedSample::new();
        ws.option(1, box |_| Program::X);
        ws.option(1, box |_| Program::Y);
        ws.option(2, box |_| Program::T);

        ws.option(3, box |r| Program::Xor(box gen(r), box gen(r)));
        ws.option(1, box |r| Program::And(box gen(r), box gen(r)));
        ws.option(1, box |r| Program::Or(box gen(r), box gen(r)));

        ws.option(0, box |r| Program::Add(box gen(r), box gen(r)));
        ws.option(0, box |r| Program::Sub(box gen(r), box gen(r)));
        ws.option(0, box |r| Program::Mul(box gen(r), box gen(r)));
        ws.option(0, box |r| Program::Div(box gen(r), box gen(r)));

        ws.sample(rng)
    }

    fn new_tripple(rng: &mut XorShiftRng) -> ProgramTripple {
        let mut out = vec![];
        while out.len() != 3 {
            let p = Program::new_random(rng);
            if p.is_interesting() {
                out.push(p);
            }
        }
        (out.pop().unwrap(), out.pop().unwrap(), out.pop().unwrap())
    }

    fn eval(&self, (x,y,t): (u8,u8,u8)) -> u8 {
        let v = (x,y,t);
        match *self {
            Program::Xor(ref a, ref b) => a.eval(v) ^ b.eval(v),
            Program::And(ref a, ref b) => a.eval(v) & b.eval(v),
            Program::Or(ref a, ref b)  => a.eval(v) | b.eval(v),

            Program::Add(ref a, ref b) => a.eval(v) + b.eval(v),
            Program::Sub(ref a, ref b) => a.eval(v) - b.eval(v),
            Program::Mul(ref a, ref b) => a.eval(v) * b.eval(v),
            Program::Div(ref a, ref b) => {
                let d = b.eval(v);
                if d == 0 { return 0; }
                a.eval(v) / d
            }

            Program::X => x,
            Program::Y => y,
            Program::T => t,
        }
    }

    fn contains(&self, f: fn(&Program) -> bool) -> bool {
        f(self) ||
        match *self {
            Program::X | Program::Y  | Program::T => false,
            Program::Xor(ref a, ref b) => a.contains(f) || b.contains(f),
            Program::And(ref a, ref b) => a.contains(f) || b.contains(f),
            Program::Or(ref a, ref b) => a.contains(f) || b.contains(f),

            Program::Add(ref a, ref b) => a.contains(f) || b.contains(f),
            Program::Sub(ref a, ref b) => a.contains(f) || b.contains(f),
            Program::Mul(ref a, ref b) => a.contains(f) || b.contains(f),
            Program::Div(ref a, ref b) => a.contains(f) || b.contains(f),
        }
    }

    fn is_interesting(&self) -> bool {
        fn is_t(p: &Program) -> bool {
            match *p {
                Program::T => true, _ => false
            }
        }
        fn isnt_xy(p: &Program) -> bool {
            match *p {
                Program::X | Program::Y => false, _ => true
            }
        }
        return self.contains(is_t) || self.contains(isnt_xy);
    }
}

fn eval_tripple(pt: &ProgramTripple, v: (u8, u8, u8)) ->
(u8, u8, u8) {
    let &(ref pr, ref pg, ref pb) = pt;
    (pr.eval(v), pg.eval(v), pb.eval(v))
}
fn main() {
}
