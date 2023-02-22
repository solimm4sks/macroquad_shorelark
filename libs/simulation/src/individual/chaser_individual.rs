use crate::*;

pub struct ChaserIndividual{
    fitness: f32,
    chromosome: ga::Chromosome,
}

impl ga::Individual for ChaserIndividual {
    fn create(chromosome: ga::Chromosome) -> Self {
        Self {
            fitness: 0.0,
            chromosome,
        }
    }

    fn chromosome(&self) -> &ga::Chromosome {
        &self.chromosome
    }

    fn fitness(&self) -> f32 {
        self.fitness
    }
}

impl ChaserIndividual{
    //FITNESS CALCULATION
    pub(crate) fn from_chaser(chaser: &Chaser) -> Self {
        let fit = if chaser.killed {10.0} else {0.0};

        Self{
            fitness: fit,
            chromosome: chaser.as_chromosome(),
        }
    }

    pub(crate) fn into_chaser(self, rng: &mut dyn RngCore) -> Chaser {
        Chaser::from_chromosome(self.chromosome, rng)
    }
}