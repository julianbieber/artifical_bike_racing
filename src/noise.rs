use ::noise::{NoiseFn, Simplex};

pub struct WorldNoise {
    samplers: Vec<NoiseSampler>,
}
impl WorldNoise {
    pub fn new() -> Self {
        Self {
            samplers: vec![
                NoiseSampler {
                    simplex: Simplex::new(0),
                    divisor: 50.0,
                    height_multiplier: 2.5,
                },
                NoiseSampler {
                    simplex: Simplex::new(5),
                    divisor: 10.0,
                    height_multiplier: 1.5,
                },
                NoiseSampler {
                    simplex: Simplex::new(4),
                    divisor: 5.0,
                    height_multiplier: 0.5,
                },
                NoiseSampler {
                    simplex: Simplex::new(3),
                    divisor: 75.0,
                    height_multiplier: 5.5,
                },
                NoiseSampler {
                    simplex: Simplex::new(2),
                    divisor: 100.0,
                    height_multiplier: 20.5,
                },
            ],
        }
    }

    pub fn get_height(&self, x: usize, z: usize) -> f32 {
        self.samplers.iter().map(|s| s.sample(x, z)).sum()
    }
}

struct NoiseSampler {
    simplex: Simplex,
    divisor: f64,
    height_multiplier: f64,
}

impl NoiseSampler {
    fn sample(&self, x: usize, z: usize) -> f32 {
        (self
            .simplex
            .get([x as f64 / self.divisor, z as f64 / self.divisor])
            * self.height_multiplier) as f32
    }
}
