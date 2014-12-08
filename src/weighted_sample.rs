use std::rand::Rng;

pub struct WeightedSample<T, R> {
    options: Vec<((u32, u32), Box<|&mut R|: 'static -> T>)>,
    total: u32
}

impl <T, R> WeightedSample<T, R> where R: Rng{
    pub fn new() -> WeightedSample<T, R> {
        WeightedSample {
            options: Vec::new(),
            total: 0
        }
    }

    pub fn option(&mut self, weight: u32, f: Box<|&mut R|: 'static -> T>) {
        self.options.push(((self.total, self.total + weight), f));
        self.total += weight;
    }

    pub fn sample(&mut self, r: &mut R) -> T {
        let i = r.gen_range(0, self.total);
        let f = self.options
                    .iter_mut()
                    .filter(|&&((s, e), _)| i >= s && i <= e)
                    .nth(0)
                    .unwrap();
        let &(_, ref mut f) = f;
        (**f)(r)
    }
}
