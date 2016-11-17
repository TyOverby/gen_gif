extern crate lux;
extern crate nd_iter;
extern crate rand;
extern crate rustc_serialize;

use rand::XorShiftRng;
use weighted_sample::WeightedSample;
use rustc_serialize::json;
use lux::prelude::*;
use lux::interactive::Event;
use lux::graphics::ColorVertex;
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

    fn new_interesting(rng: &mut XorShiftRng) -> Program {
        loop {
            let p = Program::new_random(rng);
            if p.is_interesting() {
                return p;
            }
        }
    }

    fn eval(&self, (x, y, t): (u8,u8,u8)) -> u8 {
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

fn colormap() -> Vec<[f32; 4]> {
    let mut out = vec![];
    for i in 0 .. 256 {
        let h = (i as f32 / 255.0) * 240.0;
        out.push(hsv(h, 1.0, 1.0));
    }
    out
}

fn display_program(p: &Program, lux: &mut Window, wait_space: bool) {
    let mut iteration = 0;
    let mut first = true;
    let colormap = colormap();
    while lux.is_open() && (first || iteration != 0) {
        if !wait_space || lux.is_key_pressed(' ') {
            let mut frame = lux.cleared_frame(rgb(0, 0, 0));
            {
                let mut out = Vec::with_capacity(256 * 256 * 4);
                for (x, y) in nd_iter::iter_2d(0 .. 256, 0 .. 256) {
                    let c = p.eval((x as u8, y as u8, iteration));
                    out.push(ColorVertex {
                        pos: [x as f32, y as f32],
                        color: colormap[c as usize],
                    });
                }
                frame.draw(Pixels{  pixels: &out, .. Default::default()}).unwrap();
            }

            first = false;
            iteration += 1;
        }
    }
}

fn run_generations() {
    let mut lux = Window::new_with_defaults().unwrap();
    let mut rand = XorShiftRng::new_unseeded();
    let mut iter: u8 = 0;
    let rand_id: u64 = rand::random();

    while lux.is_open() {
        iter += 1;
        let prog = Program::new_interesting(&mut rand);
        display_program(&prog, &mut lux, false);

        let mut should_save = false;
        for event in lux.events() {
            if let Event::KeyPressed(_, Some(' '), _) = event {
                should_save = true;
            }
        }

        if should_save {
            let string_repr = json::encode(&prog).unwrap();
            let filename = format!("examples/{}-{}.json", rand_id, iter);
            File::create(&filename)
                 .and_then(|mut f| f.write_all(string_repr.as_bytes()))
                 .unwrap();
            println!("Saved to {}", filename);
        }
    }
}

fn run_single(name: &str) {
    let mut lux = Window::new_with_defaults().unwrap();
    let mut string_buffer = String::new();
    File::open(name)
         .and_then(|mut f| f.read_to_string(&mut string_buffer))
         .ok().expect("expected reading to work.");
    let program = json::decode(&string_buffer[..])
                       .ok().expect("expected decoding to work.");

    display_program(&program, &mut lux, true);
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
