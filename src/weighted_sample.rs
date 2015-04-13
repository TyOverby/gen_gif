use rand::Rng;

pub struct WeightedSample<T, R> {
    options: Vec<((u32, u32), Box<Fn(&mut R) -> T>)>,
    total: u32
}

impl <T, R> WeightedSample<T, R> where R: Rng{
    pub fn new() -> WeightedSample<T, R> {
        WeightedSample {
            options: Vec::new(),
            total: 0
        }
    }

    pub fn option<F: 'static + Fn(&mut R)-> T>(&mut self, weight: u32, f: F) {
        self.options.push(((self.total, self.total + weight), Box::new(f)));
        self.total += weight;
    }

    pub fn sample(&mut self, r: &mut R) -> T {
        let i = r.gen_range(0, self.total);
        let f = self.options
                    .iter_mut()
                    .filter(|&&mut ((s, e), _)| i >= s && i <= e)
                    .nth(0)
                    .unwrap();
        let &mut (_, ref mut f) = f;
        (**f)(r)
    }
}
