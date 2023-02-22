use crate::*;

pub trait Brain {
    fn random(rng: &mut dyn RngCore, eye: &Eye) -> Self where Self: Sized;
    fn as_chromosome(&self) -> ga::Chromosome;
    fn from_chromosome(chromosome: ga::Chromosome, eye: &Eye) -> Self where Self: Sized;
    fn topology(eye: &Eye) -> Vec<nn::LayerTopology> where Self: Sized;
}

#[derive(Debug)]
pub struct HervorBrain {
    pub(crate) nn: nn::Network
}

impl Brain for HervorBrain{
    fn random(rng: &mut dyn RngCore, eye: &Eye) -> Self {
        Self {
            nn: nn::Network::random(rng, &Self::topology(eye)),
        }
    }

    fn as_chromosome(&self) -> ga::Chromosome {
        ga::Chromosome::new(self.nn.weights())
    }

    fn from_chromosome(chromosome: ga::Chromosome, eye: &Eye) -> Self {
        Self {
            nn: nn::Network::from_weights(&Self::topology(eye), chromosome),
        }
    }

    //edit topology here
    fn topology(eye: &Eye) -> Vec<nn::LayerTopology> {
        vec![
            nn::LayerTopology {
                neurons: eye.cells() * 2,
            },
            nn::LayerTopology {
                neurons: 4 * eye.cells(),
            },
            nn::LayerTopology { neurons: 2 },
        ]
    }
}

#[derive(Debug)]
pub struct ChaserBrain{
    pub(crate) nn: nn::Network
}

impl Brain for ChaserBrain{
    fn random(rng: &mut dyn RngCore, eye: &Eye) -> Self {
        Self {
            nn: nn::Network::random(rng, &Self::topology(eye)),
        }
    }

    fn as_chromosome(&self) -> ga::Chromosome {
        ga::Chromosome::new(self.nn.weights())
    }

    fn from_chromosome(chromosome: ga::Chromosome, eye: &Eye) -> Self {
        Self {
            nn: nn::Network::from_weights(&Self::topology(eye), chromosome),
        }
    }

    //edit topology here
    fn topology(eye: &Eye) -> Vec<nn::LayerTopology> {
        vec![
            nn::LayerTopology {
                neurons: eye.cells(),
            },
            nn::LayerTopology {
                neurons: 2 * eye.cells(),
            },
            nn::LayerTopology { neurons: 2 },
        ]
    }
}

