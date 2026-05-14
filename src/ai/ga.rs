use crate::ai::network::{B, DinoNet, NUM_PARAMS};
use rand::Rng;

pub const POPULATION: usize = 100;
pub const ELITE: usize = 2;
pub const MUTATION_RATE: f32 = 0.10;
pub const MUTATION_STD: f32 = 0.2;
pub const TOURNAMENT: usize = 3;

#[derive(Clone, Debug)]
pub struct Individual {
    pub genes: Vec<f32>,
    pub fitness: f32,
}

pub struct GeneticAlgorithm {
    pub population: Vec<Individual>,
    pub generation: usize,
    pub current: usize,
    pub best_fitness: f32,
    pub avg_fitness: f32,
}

impl GeneticAlgorithm {
    pub fn new() -> Self {
        let pop: Vec<Individual> = (0..POPULATION)
            .map(|_| {
                let mut rng = rand::thread_rng();
                Individual {
                    genes: (0..NUM_PARAMS).map(|_| rng.gen_range(-1.0..1.0)).collect(),
                    fitness: 0.0,
                }
            })
            .collect();
        Self {
            population: pop,
            generation: 1,
            current: 0,
            best_fitness: 0.0,
            avg_fitness: 0.0,
        }
    }

    pub fn best_network(&self) -> DinoNet<B> {
        let idx = self
            .population
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.fitness.partial_cmp(&b.1.fitness).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);
        DinoNet::from_weights(&self.population[idx].genes)
    }

    pub fn is_done(&self) -> bool {
        self.current >= POPULATION
    }

    pub fn record_fitness(&mut self, fitness: f32) {
        if self.current < POPULATION {
            self.population[self.current].fitness = fitness;
            self.current += 1;
        }

        if self.is_done() {
            self.evolve();
        }
    }

    pub fn evolve(&mut self) {
        self.best_fitness = self
            .population
            .iter()
            .map(|i| i.fitness)
            .fold(0.0f32, f32::max);
        self.avg_fitness =
            self.population.iter().map(|i| i.fitness).sum::<f32>() / self.population.len() as f32;

        self.population.sort_by(|a, b| {
            b.fitness
                .partial_cmp(&a.fitness)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut next_gen: Vec<Individual> = Vec::with_capacity(POPULATION);

        // Elitism
        for i in 0..ELITE {
            next_gen.push(self.population[i].clone());
        }

        let mut rng = rand::thread_rng();
        while next_gen.len() < POPULATION {
            let p1 = self.tournament(&mut rng);
            let p2 = self.tournament(&mut rng);

            let mut child_genes = if rng.gen_range(0.0..1.0) < 0.7 {
                self.crossover(
                    &self.population[p1].genes,
                    &self.population[p2].genes,
                    &mut rng,
                )
            } else {
                // If no crossover, pick one of the parents
                if rng.gen_bool(0.5) {
                    self.population[p1].genes.clone()
                } else {
                    self.population[p2].genes.clone()
                }
            };

            self.mutate(&mut child_genes, &mut rng);

            next_gen.push(Individual {
                genes: child_genes,
                fitness: 0.0,
            });
        }

        self.population = next_gen;
        self.generation += 1;
        self.current = 0;
    }

    fn tournament(&self, rng: &mut impl Rng) -> usize {
        let mut best = rng.gen_range(0..POPULATION);
        for _ in 1..TOURNAMENT {
            let idx = rng.gen_range(0..POPULATION);
            if self.population[idx].fitness > self.population[best].fitness {
                best = idx;
            }
        }
        best
    }

    fn crossover(&self, a: &[f32], b: &[f32], rng: &mut impl Rng) -> Vec<f32> {
        // Multi-point or Uniform crossover
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| if rng.gen_bool(0.5) { *x } else { *y })
            .collect()
    }

    fn mutate(&self, genes: &mut [f32], rng: &mut impl Rng) {
        for g in genes.iter_mut() {
            if rng.gen_range(0.0..1.0) < MUTATION_RATE {
                // Gaussian mutation
                *g += rng.gen_range(-1.0..1.0) * MUTATION_STD;
                *g = g.clamp(-5.0, 5.0);
            }
        }
    }
}
