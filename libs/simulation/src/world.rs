use crate::{*, hervor::Hervor, chaser::Chaser};

#[derive(Debug)]
pub struct World {
    pub(crate) hervors: Vec<Hervor>,
    pub(crate) chasers: Vec<Chaser>,
    pub(crate) plants: Vec<Plant>,
}
impl World {
    pub fn random(rng: &mut dyn RngCore, nhervor: usize, nchaser: usize, nplant: usize) -> Self {
        let hervors = (0..nhervor)
            .map(|_| Hervor::random(rng))
            .collect();

        let chasers = (0..nchaser)
        .map(|_| Chaser::random(rng))
        .collect();

        let plants = (0..nplant)
            .map(|_| Plant::random(rng))
            .collect();
        
        //animals and plants can overlap :(, use e.g. Poisson disk sampling ( https://en.wikipedia.org/wiki/Supersampling)
        Self { hervors, chasers, plants }
    }

    pub fn hervors(&self) -> &[Hervor] {
        &self.hervors
    }

    pub fn chasers(&self) -> &[Chaser] {
        &self.chasers
    }

    pub fn plants(&self) -> &[Plant] {
        &self.plants
    }
}