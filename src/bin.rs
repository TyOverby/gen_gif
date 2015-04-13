extern crate lux;
extern crate nd_iter;
extern crate rand;
extern crate rustc_serialize;

use rand::XorShiftRng;
use weighted_sample::WeightedSample;
use rustc_serialize::json;
use lux::prelude::*;
use std::fs::File;
use std::io::{Write, Read};

mod weighted_sample;

#[derive(RustcEncodable, RustcDecodable)]
enum ColorSpace {
    Rgb(Program),
    Hls(Program)
}

#[derive(RustcEncodable, RustcDecodable)]
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

fn eval_tripple(pt: &ProgramTripple, v: (u8, u8, u8)) ->
(u8, u8, u8) {
    let &(ref pr, ref pg, ref pb) = pt;
    (pr.eval(v), pg.eval(v), pb.eval(v))
}

fn display_program(p: &ProgramTripple, lux: &mut Window) {
    let mut iter = 0;
    let mut first = true;
    while lux.is_open() && (first || iter != 0) {
        let mut frame = lux.cleared_frame(rgb(0, 0, 0));
        {
            let pix = nd_iter::iter_2d(0 .. 256, 0 .. 256).map(|(x, y)|{
                let c = eval_tripple(&p, (x as u8, y as u8, iter));
                let hue = (c.0 as f32 / 255.0) * 240.0;
                ((x as f32, y as f32), hsv(hue, 1.0, 1.0)) //, c.1 as f32 / 255.0, c.2 as f32 / 255.0))
            });
            frame.draw_pixels(pix);
        }

        first = false;
        iter += 1;
    }
}

fn run_generations() {
    let mut lux = Window::new().unwrap();
    let mut rand = XorShiftRng::new_unseeded();
    let mut iter: u8 = 0;
    let rand_id: u64 = rand::random();

    while lux.is_open() {
        iter += 1;
        let progs = Program::new_tripple(&mut rand);
        display_program(&progs, &mut lux);

        let mut should_save = false;
        for event in lux.events() {
            if let Event::KeyPressed(_, Some(' '), _) = event {
                should_save = true;
            }
        }

        if should_save {
            let string_repr = json::encode(&progs).unwrap();
            let filename = format!("{}-{}.json", rand_id, iter);
            File::create(&filename)
                 .and_then(|mut f| f.write_all(string_repr.as_bytes()))
                 .unwrap();
            println!("Saved to {}", filename);
        }
    }
}

fn run_single(name: &str) {
    let mut lux = Window::new().unwrap();
    let mut string_buffer = String::new();
    File::open(name)
         .and_then(|mut f| f.read_to_string(&mut string_buffer))
         .ok().expect("expected reading to work.");
    let program = json::decode(&string_buffer[..])
                       .ok().expect("expected decoding to work.");

    display_program(&program, &mut lux);
}

fn main() {
    let mut args = std::env::args();
    let _ = args.next();
    match args.next() {
        Some(file) => {
            run_single(&file[..]);
            for file in args {
                run_single(&file[..]);
            }
        }
        None => run_generations()
    }
}
