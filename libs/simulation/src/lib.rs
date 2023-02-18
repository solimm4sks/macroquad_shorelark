pub use self::{animal::*, food::*, world::*, eye::*, brain::*};

mod animal;
mod animal_individual;
mod food;
mod world;
mod eye;
mod brain;

use self::animal_individual::*;
use lib_neural_network as nn;
pub use lib_genetic_algorithm as ga;
pub use nalgebra as na;
pub use rand;
use rand::Rng;
use rand::RngCore;

use std::f32::consts::PI;
const SPEED_MIN: f32 = 0.001;
const SPEED_MAX: f32 = 0.005;
const SPEED_ACCEL: f32 = 0.2;
const ROTATION_ACCEL: f32 = PI / 32.0;
const GENERATION_LENGTH: usize = 2500;
const EAT_RANGE:f32 = 0.02;

pub struct Config {
    speed_min: f32,
    speed_max: f32,
    speed_accel: f32,
    rotation_accel: f32,
    generation_length: usize,
    nanimals: i32, 
    nfood: i32
}

impl Config {
    pub fn new(speed_min:f32, speed_max: f32, speed_accel:f32, rotation_accel:f32, generation_length:usize, nanimals: i32, nfood: i32) -> Self{
        Self {speed_min, speed_max, speed_accel, rotation_accel, generation_length, nanimals, nfood}
    }
    pub fn low_new(nanimals: i32, nfood: i32) -> Self {
        Self::new(SPEED_MIN, SPEED_MAX, SPEED_ACCEL, ROTATION_ACCEL, GENERATION_LENGTH, nanimals, nfood)
    }
    pub fn default() -> Self {
        Self::new(SPEED_MIN, SPEED_MAX, SPEED_ACCEL, ROTATION_ACCEL, GENERATION_LENGTH, 20, 40)
    }
}

pub struct Simulation {
    world: World,
    ga: ga::GeneticAlgorithm<ga::RouletteWheelSelection>,
    age: usize,
    generation: usize,
    config: Config
}
impl Simulation {
    pub fn random(rng: &mut dyn rand::RngCore, conf: Config) -> Self {
        let world = World::random(rng, conf.nanimals, conf.nfood);
        let ga = ga::GeneticAlgorithm::new(
            ga::RouletteWheelSelection::default(),
            ga::UniformCrossover::default(),
            ga::GaussianMutation::new(0.01, 0.3)
        );

        Self {
            world, ga, age: 0, generation: 0, config:conf
        }
    }

    pub fn generation(&self) -> usize {
        self.generation
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    fn move_animals(&mut self){
        for animal in &mut self.world.animals {
            animal.position -= animal.rotation * na::Vector2::new(0.0, animal.speed);
            
            let offset:na::Vector2<f32> = animal.rotation * na::Vector2::new(0.0, 0.03); //unforch hardcoded

            animal.position.x = na::wrap(animal.position.x + offset.x, 0.0, 1.0) - offset.x;
            animal.position.y = na::wrap(animal.position.y + offset.y, 0.0, 1.0) - offset.y;
        }
    }

    //improve with hit testing (https://en.wikipedia.org/wiki/Hit-testing)
    fn proc_eating(&mut self, rng: &mut dyn RngCore){
        for animal in &mut self.world.animals {
            for food in &mut self.world.foods {
                let dist = na::distance(&animal.position, &food.position);
                if dist <= EAT_RANGE {
                    animal.satiation += 1;
                    food.position = rng.gen(); //move food when eaten
                }
            }
        }
    }

    fn proc_brains(&mut self){ //, rng: &mut dyn RngCore
        for animal in &mut self.world.animals {
            let vision = animal.eye.process_vision(&animal.position, &animal.rotation, &self.world.foods);
            let response = animal.brain.nn.propagate(vision);
            
            //relavite values
            let speed = response[0].clamp(-self.config.speed_accel, self.config.speed_accel);
            let rotation = response[1].clamp(-self.config.rotation_accel, self.config.rotation_accel);

            animal.speed = (animal.speed + speed).clamp(self.config.speed_min, self.config.speed_max);
            animal.rotation = na::Rotation2::new(animal.rotation.angle() + rotation);
        }
    }


    pub fn step(&mut self, rng: &mut dyn RngCore) -> Option<ga::Statistics> {
        self.proc_eating(rng);
        self.proc_brains();
        self.move_animals();

        self.age += 1;

        if self.age > self.config.generation_length {
            self.generation += 1;
            Some(self.evolve(rng))
        } else {
            None
        }
    }

    pub fn next_gen(&mut self, rng: &mut dyn RngCore) -> ga::Statistics {
        loop {
            if let Some(summary) = self.step(rng){
                return summary;
            }
        }
    }

    pub fn multiple_gen(&mut self, amount:usize, rng: &mut dyn RngCore) -> ga::Statistics {
        let mut stats:lib_genetic_algorithm::Statistics = self.next_gen(rng);
        for _ in 0..(amount - 1) {
            stats = self.next_gen(rng);
        }
        stats
    }

    fn evolve(&mut self, rng: &mut dyn RngCore) -> ga::Statistics{
        self.age = 0;

        let current_population: Vec<_> = self.world.animals.iter().map(AnimalIndividual::from_animal).collect();
        let (evolved_population, stats) = self.ga.evolve(rng, &current_population);

        self.world.animals = evolved_population.into_iter().map(|i| i.into_animal(rng)).collect();

        for food in &mut self.world.foods {
            food.position = rng.gen();
        }

        stats
    }
}