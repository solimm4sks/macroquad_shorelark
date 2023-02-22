pub use self::{plant::*, world::*, eye::*, brain::*, hervor::*, chaser::*,
    individual::{chaser_individual::*, hervor_individual::*}
};

//mod animal;
mod individual;
mod plant;
mod world;
mod eye;
mod brain;
mod hervor;
mod chaser;

use serde_json;
use lib_neural_network as nn;
pub use lib_genetic_algorithm as ga;
pub use nalgebra as na;
//use nn::Network;
pub use rand;
use rand::Rng;
use rand::RngCore;

use std::{f32::consts::PI, fs};
const SPEED_MIN: f32 = 0.001;
const SPEED_MAX: f32 = 0.005;
const SPEED_ACCEL: f32 = 0.2;
const ROTATION_ACCEL: f32 = PI / 32.0;
const GENERATION_LENGTH: usize = 2500;
const EAT_RANGE:f32 = 0.02;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub speed_min: f32,
    pub speed_max: f32,
    pub speed_accel: f32,
    pub rotation_accel: f32,
    pub generation_length: usize,
    pub nhervors: usize,
    pub nchasers: usize,
    pub nplants: usize,
    pub nworlds: usize
}

impl Config {
    pub fn new(speed_min:f32, speed_max: f32, speed_accel:f32, rotation_accel:f32, generation_length:usize, nhervors: usize, nchasers: usize, nplants: usize, nworlds: usize) -> Self{
        Self {speed_min, speed_max, speed_accel, rotation_accel, generation_length, nhervors, nchasers, nplants, nworlds}
    }
    pub fn low_new(nhervors: usize, nchasers: usize, nplants: usize, nworlds: usize) -> Self {
        Self::new(SPEED_MIN, SPEED_MAX, SPEED_ACCEL, ROTATION_ACCEL, GENERATION_LENGTH, nhervors, nchasers, nplants, nworlds)
    }
    pub fn default() -> Self {
        Self::new(SPEED_MIN, SPEED_MAX, SPEED_ACCEL, ROTATION_ACCEL, GENERATION_LENGTH, 1, 2, 20, 1000)
    }
}

pub struct Simulation {
    worlds: Vec<World>,
    ga: ga::GeneticAlgorithm<ga::RouletteWheelSelection>,
    age: usize,
    generation: usize,
    config: Config
}
impl Simulation {
    pub fn random(rng: &mut dyn rand::RngCore, conf: Config) -> Self {
        let mut worlds = Vec::with_capacity(conf.nworlds);
        for _ in 0..(conf.nworlds){
            worlds.push(World::random(rng, conf.nhervors, conf.nchasers, conf.nplants));
        }
        
        let ga = ga::GeneticAlgorithm::new(
            ga::RouletteWheelSelection::default(),
            ga::UniformCrossover::default(),
            ga::GaussianMutation::new(0.01, 0.3)
        );

        Self {
            worlds, ga, age: 0, generation: 0, config:conf
        }
    }

    pub fn generation(&self) -> usize {
        self.generation
    }

    pub fn worlds(&self) -> &Vec<World> {
        &self.worlds
    }

    fn move_hervors(&mut self){
        for world in &mut self.worlds{
            for hervor in &mut world.hervors {
                if hervor.dead {
                    continue;
                }

                hervor.position -= hervor.rotation * na::Vector2::new(0.0, hervor.speed);
                
                let offset:na::Vector2<f32> = hervor.rotation * na::Vector2::new(0.0, 0.03); //unforch hardcoded visual offset for border

                hervor.position.x = na::wrap(hervor.position.x + offset.x, 0.0, 1.0) - offset.x;
                hervor.position.y = na::wrap(hervor.position.y + offset.y, 0.0, 1.0) - offset.y;
            }
        }
    }

    fn move_chasers(&mut self){
        for world in &mut self.worlds{
            for chaser in &mut world.chasers {
                chaser.position -= chaser.rotation * na::Vector2::new(0.0, chaser.speed);
                
                let offset:na::Vector2<f32> = chaser.rotation * na::Vector2::new(0.0, 0.03); //unforch hardcoded visual offset for border

                chaser.position.x = na::wrap(chaser.position.x + offset.x, 0.0, 1.0) - offset.x;
                chaser.position.y = na::wrap(chaser.position.y + offset.y, 0.0, 1.0) - offset.y;
            }
        }
    }
    

    //improve with hit testing (https://en.wikipedia.org/wiki/Hit-testing)
    fn proc_eating_plants(&mut self, rng: &mut dyn RngCore){
        for world in &mut self.worlds{
            for hervor in &mut world.hervors {
                for plant in &mut world.plants {
                    if plant.eaten {
                        continue;
                    }

                    let dist = na::distance(&hervor.position, &plant.position);
                    if dist <= EAT_RANGE {
                        hervor.satiation += 1;
                        //plant.position = rng.gen(); //move plant when eaten,
                        plant.eaten = true;
                    }
                }
            }
        }
    }
    
    fn proc_eating_hervors(&mut self){
        for world in &mut self.worlds{
            for chaser in &mut world.chasers {
                for hervor in &mut world.hervors {
                    let dist = na::distance(&hervor.position, &chaser.position);

                    if dist <= EAT_RANGE && !hervor.dead{ //this is shit detection
                        chaser.killed = true;
                        hervor.dead = true;
                    }
                }
            }
        }
    }

    fn proc_hervor_brains(&mut self){ //, rng: &mut dyn RngCore
        for world in &mut self.worlds{
            for hervor in &mut world.hervors {
                let mut vision:Vec<f32> = hervor.eye.process_vision_see_plants(&hervor.position, &hervor.rotation, &world.plants);
                vision.append(&mut hervor.eye.process_vision_see_chasers(&hervor.position, &hervor.rotation, &world.chasers));

                let response = hervor.brain.nn.propagate(vision);
                
                //relavite values
                let speed = response[0].clamp(-self.config.speed_accel, self.config.speed_accel);
                let rotation = response[1].clamp(-self.config.rotation_accel, self.config.rotation_accel);

                hervor.speed = (hervor.speed + speed).clamp(self.config.speed_min, self.config.speed_max);
                hervor.rotation = na::Rotation2::new(hervor.rotation.angle() + rotation);
            }
        }
    }

    fn proc_chaser_brains(&mut self){ //, rng: &mut dyn RngCore
        for world in &mut self.worlds{
            for chaser in &mut world.chasers {
                let vision = chaser.eye.process_vision_see_hervors(&chaser.position, &chaser.rotation, &world.hervors);
                let response = chaser.brain.nn.propagate(vision);
                
                //relavite values
                let speed = response[0].clamp(-self.config.speed_accel, self.config.speed_accel);
                let rotation = response[1].clamp(-self.config.rotation_accel, self.config.rotation_accel);

                chaser.speed = (chaser.speed + speed).clamp(self.config.speed_min, self.config.speed_max);
                chaser.rotation = na::Rotation2::new(chaser.rotation.angle() + rotation);
            }
        }
    }


    pub fn step(&mut self, rng: &mut dyn RngCore) -> Option<(ga::Statistics, ga::Statistics)> {
        self.proc_eating_plants(rng);
        self.proc_eating_hervors();
        self.proc_hervor_brains();
        self.proc_chaser_brains();
        self.move_hervors();
        self.move_chasers();

        self.age += 1;

        if self.age > self.config.generation_length {
            self.generation += 1;
            Some(self.evolve(rng))
        } else {
            None
        }
    }

    pub fn next_gen(&mut self, rng: &mut dyn RngCore) -> (ga::Statistics, ga::Statistics) {
        loop {
            if let Some(summary) = self.step(rng){
                return summary;
            }
        }
    }

    pub fn multiple_gen(&mut self, amount:usize, rng: &mut dyn RngCore) -> (ga::Statistics, ga::Statistics) {
        let mut stats = self.next_gen(rng);
       
        for i in 1..amount {
            //print!("Processing Generation: {i} / {amount}"); //printing doesnt work for some reason
            stats = self.next_gen(rng);
        }
        stats
    }

    fn evolve_hervors(&mut self, rng: &mut dyn RngCore) -> ga::Statistics{
        let mut current_population_hervors: Vec<HervorIndividual> = vec![];
        for world in &self.worlds {
            current_population_hervors.extend(world.hervors.iter().map(HervorIndividual::from_hervor).collect::<Vec<HervorIndividual>>());
        }
        let (evolved_population_hervors, stats_hervors) = self.ga.evolve(rng, &current_population_hervors);

        assert_eq!(evolved_population_hervors.len(), self.config.nworlds * self.config.nhervors);

        let mut i = 0; //hervor counter
        let mut j = 0; //world counter
        self.worlds[0].hervors.clear();
        for hervor_ind in evolved_population_hervors{
            self.worlds[j].hervors.push(hervor_ind.into_hervor(rng));
            i += 1;
            if i == self.config.nhervors && j < self.config.nworlds - 1 {
                i = 0; j += 1;
                self.worlds[j].hervors.clear();
            }
        }

        stats_hervors
    }

    fn evolve_chasers(&mut self, rng: &mut dyn RngCore) -> ga::Statistics{
        let mut current_population_chasers: Vec<ChaserIndividual> = vec![];
        for world in &self.worlds {
            current_population_chasers.extend(world.chasers.iter().map(ChaserIndividual::from_chaser).collect::<Vec<ChaserIndividual>>());
        }
        let (evolved_population_chasers, stats_chasers) = self.ga.evolve(rng, &current_population_chasers);

        assert_eq!(evolved_population_chasers.len(), self.config.nworlds * self.config.nchasers);

        let mut i = 0; //chaser counter
        let mut j = 0; //world counter
        self.worlds[0].chasers.clear();

        for chaser_ind in evolved_population_chasers {
            self.worlds[j].chasers.push(chaser_ind.into_chaser(rng));

            i += 1;
            if i == self.config.nchasers && j < self.config.nworlds - 1{
                i = 0; j += 1;
                self.worlds[j].chasers.clear();
            }
        }

        stats_chasers
    }

    fn evolve_plants(&mut self, rng: &mut dyn RngCore){
        for world in &mut self.worlds{
            for plant in &mut world.plants {
                plant.position = rng.gen();
                plant.eaten = false;
            }
        }
    }

    fn evolve(&mut self, rng: &mut dyn RngCore) -> (ga::Statistics, ga::Statistics){
        self.age = 0;

        let stats_hervor = self.evolve_hervors(rng);
        let stats_chasers = self.evolve_chasers(rng);
        self.evolve_plants(rng);

        (stats_hervor, stats_chasers)
    }

    pub fn save_simulation(&self){
        let mut text = String::new();
        text.push_str(&serde_json::to_string(&self.config).unwrap());
        std::fs::write("save_data/config.json", text).expect("Unable to write file");

        let mut text = String::new();
        for world in &self.worlds{
            for hervor in &world.hervors {
                text.push_str(&serde_json::to_string(&hervor.brain.nn.weights()).unwrap());
                text.push_str("\n");
            }
            for chaser in &world.chasers {
                text.push_str(&serde_json::to_string(&chaser.brain.nn.weights()).unwrap());
                text.push_str("\n");
            }
        }

        std::fs::write("save_data/weights", text).expect("Unable to open file");
    }

    pub fn load_simulation(&mut self, rng: &mut dyn RngCore){
        let conf: Config = serde_json::from_str(&fs::read_to_string("save_data/config.json").expect("Unable to open file")).unwrap();
        let binding = fs::read_to_string("save_data/weights").expect("Unable to open file");
        let text: Vec<&str> = binding.lines().collect();

        self.worlds = Vec::with_capacity(conf.nworlds);
        let mut line_cnt = 0;
        for i in 0..conf.nworlds {
            self.worlds.push(World::random(rng, conf.nhervors, conf.nchasers, conf.nplants));
            for j in 0..conf.nhervors {
                let wei:Vec<f32> = serde_json::from_str(&text[line_cnt]).unwrap();
                let topo = HervorBrain::topology(&self.worlds[i].hervors[j].eye);
                self.worlds[i].hervors[j].brain.nn.from_weights_inplace(&topo, wei);
                line_cnt += 1;
            }

            for j in 0..conf.nchasers {
                let wei:Vec<f32> = serde_json::from_str(&text[line_cnt]).unwrap();
                let topo = ChaserBrain::topology(&self.worlds[i].chasers[j].eye);
                self.worlds[i].chasers[j].brain.nn.from_weights_inplace(&topo, wei);
                line_cnt += 1;
            }
        }
    }
}