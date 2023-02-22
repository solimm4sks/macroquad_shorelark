use macroquad::prelude::*;
use macroquad::ui;
use lib_simulation::{Simulation, rand, na:: Point2};
use serde_json;

use std::f32::consts::PI;

fn window_conf() -> Conf {
    Conf {
        window_title: "Rusty Simulation".into(),
        fullscreen:false,
        ..Default::default()
      }
}

fn draw_chasers(simulation: &Simulation) {
    let world = &simulation.worlds()[0];

    for chaser in world.chasers() {
        let mut vbase:Point2<f32> = chaser.position();
        vbase.x *= screen_width() as f32;
        vbase.y *= screen_height() as f32;
        let v1 = macroquad::math::Vec2::new(vbase.x, vbase.y); //top point
        let mut v2 = macroquad::math::Vec2::new(0.0, 0.0); //left
        let mut v3 = macroquad::math::Vec2::new(0.0, 0.0); //right
        //triangle sides are 26, 26, 20

        //rotate v2 and v3 around v1
        let rotation = chaser.rotation().angle();
        v2.x = v1.x - (rotation - (112.619864948 as f32).to_radians()).cos() * 26.0; 
        v2.y = v1.y - (rotation - (112.619864948 as f32).to_radians()).sin() * 26.0;

        v3.x = v1.x - (rotation - (67.38013505195957 as f32).to_radians()).cos() * 26.0;
        v3.y = v1.y - (rotation - (67.38013505195957 as f32).to_radians()).sin() * 26.0;
        draw_triangle(v1, v2, v3, Color::from_rgba(183, 65, 14, 255));
    }
}

fn draw_hervors(simulation: &Simulation) {
    let world = &simulation.worlds()[0];

    for hervor in world.hervors() {
        let mut vbase:Point2<f32> = hervor.position();
        vbase.x *= screen_width() as f32;
        vbase.y *= screen_height() as f32;
        let v1 = macroquad::math::Vec2::new(vbase.x, vbase.y); //top point
        let mut v2 = macroquad::math::Vec2::new(0.0, 0.0); //left
        let mut v3 = macroquad::math::Vec2::new(0.0, 0.0); //right
        //triangle sides are 26, 26, 20

        //rotate v2 and v3 around v1
        let rotation = hervor.rotation().angle();
        v2.x = v1.x - (rotation - (112.619864948 as f32).to_radians()).cos() * 26.0; 
        v2.y = v1.y - (rotation - (112.619864948 as f32).to_radians()).sin() * 26.0;

        v3.x = v1.x - (rotation - (67.38013505195957 as f32).to_radians()).cos() * 26.0;
        v3.y = v1.y - (rotation - (67.38013505195957 as f32).to_radians()).sin() * 26.0;

        let clr: Color = if hervor.dead() {BLACK} else {WHITE};
        draw_triangle(v1, v2, v3, clr);
    }
}

fn draw_plants(simulation: &Simulation) {
    let world = &simulation.worlds()[0];

    for plant in world.plants() {
        let pos = plant.position();
        let clr: Color = if plant.eaten() {Color::from_rgba(51, 80, 71, 255)} else {Color::from_rgba(0, 221, 125, 255)};
        draw_circle(pos.x * screen_width() as f32, pos.y * screen_height() as f32, 5.0, clr);
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    let mut rng = rand::thread_rng();
    //let conf:lib_simulation::Config = lib_simulation::Config::low_new(30, 20);
    let conf:lib_simulation::Config = lib_simulation::Config {
        speed_min: 0.001,
        speed_max: 0.005,
        speed_accel: 0.2,
        rotation_accel: PI / 32.0,
        generation_length: 2500,
        nhervors: 1,
        nchasers: 2,
        nplants: 20,
        nworlds: 1000
    }; 

    let mut simulation: Simulation = Simulation::random(&mut rng, conf);

    let mut cur_stats:Option<(lib_simulation::ga::Statistics, lib_simulation::ga::Statistics)> = None;
    loop {
        clear_background(Color::from_rgba(31, 39, 57, 255));
        draw_plants(&simulation);
        draw_hervors(&simulation);
        draw_chasers(&simulation);

        if ui::root_ui().button(Vec2::new(10.0, 30.0), "Next Generation") {
            cur_stats = Some(simulation.next_gen(&mut rng));
            println!("Hervor, Chasers: {cur_stats:?}");
        }

        if ui::root_ui().button(Vec2::new(10.0, 55.0), "100 Generations") {
            cur_stats = Some(simulation.multiple_gen(100, &mut rng));
            println!("Hervor, Chasers: {cur_stats:?}");
        }

        if ui::root_ui().button(Vec2::new(10.0, 80.0), "1000 Generations") {
            cur_stats = Some(simulation.multiple_gen(1000, &mut rng));
            println!("Hervor, Chasers: {cur_stats:?}");
        }

        if ui::root_ui().button(Vec2::new(10.0, 105.0), "Save Simulation") {
            simulation.save_simulation();
            println!("Saved simulation.");
        }

        if ui::root_ui().button(Vec2::new(10.0, 130.0), "Load Simulation") {
            simulation.load_simulation(&mut rng);
            println!("Loaded simulation.");
        }

        let mut info_label:String = String::from("Generation ");
        info_label.push_str(&simulation.generation().to_string());
        info_label.push_str(" -> Hervor, Chasers: ");
        info_label.push_str(&serde_json::to_string(&cur_stats).unwrap());
        draw_text(&info_label, 10.0, 20.0, 18.0, WHITE);

        simulation.step(&mut rng);
        next_frame().await
    }
}
