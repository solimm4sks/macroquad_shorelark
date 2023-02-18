use crate::*;

#[derive(Debug)]
pub struct World {
    pub(crate) animals: Vec<Animal>,
    pub(crate) foods: Vec<Food>,
}
impl World {
    pub fn random(rng: &mut dyn RngCore, nanimals: i32, nfood: i32) -> Self {
        let animals = (0..nanimals)
            .map(|_| Animal::random(rng))
            .collect();

        let foods = (0..nfood)
            .map(|_| Food::random(rng))
            .collect();
        
        //animals and foods can overlap :(, use e.g. Poisson disk sampling ( https://en.wikipedia.org/wiki/Supersampling)
        Self { animals, foods }
    }

    pub fn animals(&self) -> &[Animal] {
        &self.animals
    }

    pub fn foods(&self) -> &[Food] {
        &self.foods
    }
}