use crate::*;

pub struct HervorIndividual{
    fitness: f32,
    chromosome: ga::Chromosome,
}

impl ga::Individual for HervorIndividual {
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

impl HervorIndividual{
    //FITNESS CALCULATION
    pub(crate) fn from_hervor(hervor: &Hervor) -> Self {
        let fit: f32 = hervor.satiation as f32 + if hervor.dead {0.0} else {40.0};

        Self{
            fitness: fit,
            chromosome: hervor.as_chromosome(),
        }
    }

    pub(crate) fn into_hervor(self, rng: &mut dyn RngCore) -> Hervor {
        Hervor::from_chromosome(self.chromosome, rng)
    }
}