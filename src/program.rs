use rand::Rng;
use weighted_sample::WeightedSample;

#[derive(RustcEncodable, RustcDecodable)]
pub enum Program {
    X, Y, T,
    Xor(Box<Program>, Box<Program>),
    And(Box<Program>, Box<Program>),
    Or(Box<Program>, Box<Program>),

    Add(Box<Program>, Box<Program>),
    Sub(Box<Program>, Box<Program>),
    Mul(Box<Program>, Box<Program>),
    Div(Box<Program>, Box<Program>),
}

impl Program {
    pub fn new_random<R: Rng>(rng: &mut R) -> Program {
        fn gen<R: Rng>(rng: &mut R) -> Program {
            Program::new_random(rng)
        }

        let mut ws = WeightedSample::new();
        ws.option(3, |_| Program::X);
        ws.option(3, |_| Program::Y);
        ws.option(3, |_| Program::T);

        ws.option(2, |r| Program::Xor(Box::new(gen(r)), Box::new(gen(r))));
        ws.option(2, |r| Program::And(Box::new(gen(r)), Box::new(gen(r))));
        ws.option(2, |r| Program::Or(Box::new(gen(r)), Box::new(gen(r))));

        ws.option(1, |r| Program::Add(Box::new(gen(r)), Box::new(gen(r))));
        ws.option(1, |r| Program::Sub(Box::new(gen(r)), Box::new(gen(r))));
        ws.option(1, |r| Program::Mul(Box::new(gen(r)), Box::new(gen(r))));
        ws.option(1, |r| Program::Div(Box::new(gen(r)), Box::new(gen(r))));

        ws.sample(rng)
    }

    pub fn new_interesting<R: Rng>(rng: &mut R) -> Program {
        loop {
            let p = Program::new_random(rng);
            if p.is_interesting() {
                return p;
            }
        }
    }

    pub fn eval(&self, (x, y, t): (u8,u8,u8)) -> u8 {
        let v = (x, y, t);
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

    fn length(&self) -> u32 {
        match *self {
            Program::X | Program::Y  | Program::T => 1,
            Program::Xor(ref a, ref b) => a.length() + b.length(),
            Program::And(ref a, ref b) => a.length() + b.length(),
            Program::Or(ref a, ref b) => a.length() + b.length(),

            Program::Add(ref a, ref b) => a.length() + b.length(),
            Program::Sub(ref a, ref b) => a.length() + b.length(),
            Program::Mul(ref a, ref b) => a.length() + b.length(),
            Program::Div(ref a, ref b) => a.length() + b.length(),
        }
    }

    fn is_interesting(&self) -> bool {
        fn is_t(p: &Program) -> bool {
            match *p {
                Program::T => true,
                _ => false
            }
        }
        fn isnt_var(p: &Program) -> bool {
            match *p {
                Program::X | Program::Y | Program::T => false,
                _ => true
            }
        }

        let len = self.length();

        return
            self.contains(is_t) &&
            self.contains(isnt_var) &&
            len >= 4 &&
            len <= 15;
    }
}

