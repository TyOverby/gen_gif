extern crate nd_iter;
extern crate palette;
extern crate rand;
extern crate gif;
extern crate rustc_serialize;

mod weighted_sample;
mod program;

use rand::XorShiftRng;
use palette::{Hsv, RgbHue, IntoColor};

use program::*;

fn colormap() -> Vec<u8> {
    let mut out = vec![];
    for i in 0 .. 256 {
        let h = (i as f32 / 256.0) * 3.14 * 2.0;
        let hsv = Hsv::new(RgbHue::from_radians(h), 1.0, 1.0);
        let rgb = hsv.into_rgb();
        out.push(f32_to_u8(rgb.red));
        out.push(f32_to_u8(rgb.green));
        out.push(f32_to_u8(rgb.blue));
    }
    out
}
fn f32_to_u8(v: f32) -> u8 {
    (v * 255.0) as u8
}


fn display_program(p: &Program, name: &str) {
    use std::fs::File;
    use gif::{Frame, Encoder, Repeat, SetParameter};
    use std::borrow::Cow;

    let (width, height) = (255, 255);

    let colormap = colormap();
    let mut file = File::create(name).unwrap();
    let mut encoder = Encoder::new(&mut file, width, height, &colormap[..]).unwrap();
    encoder.set(Repeat::Infinite).unwrap();

    for iteration in 0 .. 256 {
        let mut out = Vec::with_capacity((width * height) as usize);
        for (x, y) in nd_iter::iter_2d(0 .. width, 0 .. height) {
            let c = p.eval((x as u8, y as u8, iteration as u8));
            out.push(c);
        }
        let mut frame = Frame::default();
        frame.width = width;
        frame.height = height;
        frame.buffer = Cow::Borrowed(&out[..]);
        encoder.write_frame(&frame).unwrap();
    }
}

fn main() {
    let mut _rand = XorShiftRng::new_unseeded();
    let mut rand = ::rand::thread_rng();
    let prog = Program::new_interesting(&mut rand);
    display_program(&prog, "foo.gif");
}
