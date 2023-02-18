use rand::RngCore;
use rand::Rng;
use rand::seq::SliceRandom;
use std::ops::*;
use serde::{Serialize, Deserialize};

//ooga booga do this: https://setu677.medium.com/how-to-perform-roulette-wheel-and-rank-based-selection-in-a-genetic-algorithm-d0829a37a189

pub trait Individual {
    fn fitness(&self) -> f32;
    fn chromosome(&self) -> &Chromosome;
    fn create(chromosome: Chromosome) -> Self;
}

pub trait SelectionMethod {
    fn select<'a, I>(&self, rng: &mut dyn RngCore, population: &'a [I]) -> &'a I where I: Individual;
}

#[derive(Default)]
pub struct RouletteWheelSelection;

impl RouletteWheelSelection {
    pub fn new() -> Self {
        Self
    }
}

impl SelectionMethod for RouletteWheelSelection {
    fn select<'a, I>(&self, rng: &mut dyn RngCore, population: &'a [I]) -> &'a I 
    where I: Individual {
        population
        .choose_weighted(rng, |individual| individual.fitness())
        .expect("got an empty population")
    }
}

pub struct GeneticAlgorithm<S> {
    selection_method: S,
    crossover_method: Box<dyn CrossoverMethod>,
    mutation_method: Box<dyn MutationMethod>,
}

impl<S> GeneticAlgorithm<S> where S: SelectionMethod{
    pub fn new(selection_method: S, crossover_method: impl CrossoverMethod + 'static, mutation_method: impl MutationMethod + 'static) -> Self {
        Self { selection_method, crossover_method: Box::new(crossover_method), mutation_method: Box::new(mutation_method)}
    }

    pub fn evolve<I>(&self, rng: &mut dyn RngCore, population: &[I]) -> (Vec<I>, Statistics) where I: Individual{
        assert!(!population.is_empty());

        let stats = Statistics::new(population);
        let new_population = (0..population.len())
        .map(|_| {
            let parent1 = self.selection_method.select(rng, population).chromosome();
            let parent2 = self.selection_method.select(rng, population).chromosome();
            let mut child = self.crossover_method.crossover(rng, parent1, parent2);
            self.mutation_method.mutate(rng, &mut child);
            
            return I::create(child);
        })
        .collect();

        (new_population, stats)
    }
}

#[derive(Clone, Debug)]
pub struct Chromosome {
    genes: Vec<f32>,
}

impl Chromosome {
    pub fn new(genes: Vec<f32>) -> Self {
        Self{genes: genes}
    }

    pub fn len(&self) -> usize {
        self.genes.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &f32> {
        self.genes.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut f32> {
        self.genes.iter_mut()
    }
}

impl Index<usize> for Chromosome {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.genes[index]
    }
}

impl FromIterator<f32> for Chromosome {
    fn from_iter<T: IntoIterator<Item = f32>>(iter: T) -> Self {
        Self {
            genes: iter.into_iter().collect(),
        }
    }
}

impl IntoIterator for Chromosome {
    type Item = f32;
    type IntoIter = std::vec::IntoIter<f32>;

    fn into_iter(self) -> Self::IntoIter {
        self.genes.into_iter()
    }
}

pub trait CrossoverMethod {
    fn crossover(
        &self,
        rng: &mut dyn RngCore,
        parent_a: &Chromosome,
        parent_b: &Chromosome,
    ) -> Chromosome;
}

#[derive(Clone, Debug, Default)]
pub struct UniformCrossover;

impl UniformCrossover {
    pub fn new() -> Self {
        Self
    }
}

impl CrossoverMethod for UniformCrossover {
    fn crossover(&self, rng: &mut dyn RngCore, parent_a: &Chromosome, parent_b: &Chromosome) -> Chromosome {
        assert_eq!(parent_a.len(), parent_b.len());

        let parent_a = parent_a.iter();
        let parent_b = parent_b.iter();
    
        parent_a
            .zip(parent_b)
            .map(|(&a, &b)| if rng.gen_bool(0.5) { a } else { b })
            .collect()
    }
}

pub trait MutationMethod {
    fn mutate(&self, rng: &mut dyn RngCore, child: &mut Chromosome);
}

#[derive(Clone, Debug)]
pub struct GaussianMutation {
    /// Probability of changing a gene:
    /// - 0.0 = no genes will be touched
    /// - 1.0 = all genes will be touched
    chance: f32,

    /// Magnitude of that change:
    /// - 0.0 = touched genes will not be modified
    /// - 3.0 = touched genes will be += or -= by at most 3.0
    coeff: f32,
}

impl GaussianMutation {
    pub fn new(chance: f32, coeff: f32) -> Self {
        assert!(chance >= 0.0 && chance <= 1.0);

        Self { chance, coeff }
    }
}

impl MutationMethod for GaussianMutation {
    fn mutate(&self, rng: &mut dyn RngCore, child: &mut Chromosome) {
        for gene in child.iter_mut() {
            let sign = if rng.gen_bool(0.5) { -1.0 } else { 1.0 };

            if rng.gen_bool(self.chance as _) {
                *gene += sign * self.coeff * rng.gen::<f32>();
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Statistics {
    min_fitness: f32,
    max_fitness: f32,
    avg_fitness: f32,
}

impl Statistics {
    fn new<I>(population: &[I]) -> Self where I: Individual{
        assert!(!population.is_empty());
        let mut min_fitness = population[0].fitness();
        let mut max_fitness = min_fitness;
        let mut sum_fitness = 0.0;

        for individual in population {
            let fitness = individual.fitness();

            min_fitness = min_fitness.min(fitness);
            max_fitness = max_fitness.max(fitness);
            sum_fitness += fitness;
        }

        Self {
            min_fitness,
            max_fitness,
            avg_fitness: sum_fitness / (population.len() as f32),
        }
    }

    pub fn min_fitness(&self) -> f32 {
        self.min_fitness
    }

    pub fn max_fitness(&self) -> f32 {
        self.max_fitness
    }

    pub fn avg_fitness(&self) -> f32 {
        self.avg_fitness
    }
}