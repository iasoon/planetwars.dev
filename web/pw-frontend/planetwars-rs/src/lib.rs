extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate octoon_math;
extern crate serde_json;
extern crate voronoi;

use octoon_math::Mat3;
use voronoi::{make_polygons, voronoi, Point};

mod types;
mod utils;

use std::collections::HashMap;
use wasm_bindgen::prelude::*;

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Debug, Clone)]
pub struct Circle {
    r: f32,
    x: f32,
    y: f32,
    a0: f32,
    ad: f32,
    distance: usize,
}

use std::f32::consts::PI;
fn spr(from: f32) -> f32 {
    let pi2 = PI * 2.;
    ((from % pi2) + pi2) % pi2
}

impl Circle {
    pub fn new(p1: &types::Planet, p2: &types::Planet) -> Self {
        let x1 = p1.x;
        let y1 = p1.y;
        let x2 = p2.x;
        let y2 = p2.y;

        // Distance between planets
        let q = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
        // Center of between planets
        let x3 = (x1 + x2) / 2.0;
        let y3 = (y1 + y2) / 2.0;

        // Radius of circle
        let r = q * 1.0;

        // Center of circle
        let x = x3 + (r.powi(2) - (q / 2.0).powi(2)).sqrt() * (y1 - y2) / q;
        let y = y3 + (r.powi(2) - (q / 2.0).powi(2)).sqrt() * (x2 - x1) / q;
        // console_log!("{},{} -> {},{} ({},{} r={})", x1, y1, x2, y2, x, y, r);

        let a0 = spr((y - y1).atan2(x - x1));
        let a2 = spr((y - y2).atan2(x - x2));

        let mut ad = spr(a0 - a2);
        if ad > PI {
            ad = spr(a2 - a0);
        }
        // console_log!("a1 {} a2 {} ad {}", a0/PI * 180.0, a2/PI * 180.0, ad/PI*180.0);

        let distance = q.ceil() as usize + 1;
        Self {
            r,
            x,
            y,
            a0,
            ad,
            distance,
        }
    }

    pub fn get_for_remaining(&self, remaining: usize) -> ((Mat3<f32>, f32), (Mat3<f32>, f32)) {
        (
            self.get_remaining(remaining),
            self.get_remaining((remaining + 1).min(self.distance - 1)),
        )
    }

    fn get_remaining(&self, remaining: usize) -> (Mat3<f32>, f32) {
        let alpha = self.a0 + (1.0 - (remaining as f32 / self.distance as f32)) * self.ad;

        let cos = alpha.cos();
        let sin = alpha.sin();
        (
            Mat3::new(
                0.3,
                0.0,
                0.0,
                0.0,
                0.3,
                0.0,
                -self.x + cos * self.r,
                -self.y + sin * self.r,
                0.3,
            ),
            alpha,
        )
    }
}

fn create_voronoi(planets: &Vec<types::Planet>, bbox: f32) -> (Vec<f32>, Vec<usize>) {
    let mut verts: Vec<[f32; 2]> = planets.iter().map(|p| [p.x, p.y]).collect();
    let mut ids = Vec::new();

    let vor_points = planets
        .iter()
        .map(|p| Point::new(p.x as f64, p.y as f64))
        .collect();

    let vor = voronoi(vor_points, bbox as f64);
    let vor = make_polygons(&vor);

    for poly in vor.iter() {
        // Get planet index for planet that is inside this poligon
        let idx = 0;

        let mut prev = ids.len() + poly.len() - 1;
        for p in poly.iter() {
            let now = verts.len();
            verts.push([p.x.0 as f32, p.y.0 as f32]);

            ids.push(idx);
            ids.push(now);
            ids.push(prev);
            prev = now;
        }
    }

    (verts.concat(), ids)
}

#[wasm_bindgen]
pub struct Game {
    states: Vec<types::State>,
    turn: usize,

    planet_map: HashMap<(String, String), Circle>,

    /* put extra shit here */
    view_box: Vec<f32>,

    planets: Vec<f32>,
    planet_ships: Vec<usize>,

    ship_locations: Vec<f32>,
    ship_label_locations: Vec<f32>,
    ship_colours: Vec<f32>,
    ship_counts: Vec<usize>,

    current_planet_colours: Vec<f32>,

    voronoi_vertices: Vec<f32>,
    voronoi_colors: Vec<f32>,
    voronoi_indices: Vec<usize>,
}

#[wasm_bindgen]
impl Game {
    pub fn new(file: &str) -> Self {
        utils::set_panic_hook();

        // First line is fucked but we just filter out things that cannot parse
        let states: Vec<types::State> = file
            .split("\n")
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();

        let mut planet_map = HashMap::new();

        // Iterator?
        for p1 in states[0].planets.iter() {
            for p2 in states[0].planets.iter() {
                planet_map.insert((p1.name.clone(), p2.name.clone()), Circle::new(&p1, &p2));
            }
        }
        let view_box = utils::caclulate_viewbox(&states[0].planets);

        let (voronoi_vertices, voronoi_indices) =
            create_voronoi(&states[0].planets, view_box[2].max(view_box[3]));

        let voronoi_colors: Vec<f32> = voronoi_indices
            .iter()
            .map(|_| [0.0, 0.0, 0.0])
            .collect::<Vec<[f32; 3]>>()
            .concat(); // Init these colours on black

        Self {
            planets: utils::get_planets(&states[0].planets, 2.0),
            planet_ships: Vec::new(),
            view_box,

            planet_map,
            turn: 0,
            states,
            ship_locations: Vec::new(),
            ship_label_locations: Vec::new(),
            ship_colours: Vec::new(),
            ship_counts: Vec::new(),
            current_planet_colours: Vec::new(),

            voronoi_vertices,
            voronoi_indices,
            voronoi_colors,
        }
    }

    pub fn push_state(&mut self, state_str: &str) {
        if let Ok(state) = serde_json::from_str(state_str) {
            self.states.push(state);
        }
    }

    pub fn get_viewbox(&self) -> Vec<f32> {
        self.view_box.clone()
    }

    pub fn get_planets(&self) -> Vec<f32> {
        self.planets.clone()
    }

    pub fn get_planet_ships(&self) -> Vec<usize> {
        self.planet_ships.clone()
    }

    pub fn get_planet_colors(&self) -> Vec<f32> {
        self.current_planet_colours.clone()
    }

    pub fn turn_count(&self) -> usize {
        self.states.len()
    }

    pub fn update_turn(&mut self, turn: usize) -> usize {
        self.turn = turn.min(self.states.len() - 1);

        self.update_planet_ships();
        self.update_planet_colours();
        self.update_voronoi_colors();
        self.update_ship_locations();
        self.update_ship_counts();

        self.turn
    }

    fn update_planet_ships(&mut self) {
        self.planet_ships = self.states[self.turn]
            .planets
            .iter()
            .map(|p| p.ship_count as usize)
            .collect();
    }

    fn update_voronoi_colors(&mut self) {
        for (i, p) in self.states[self.turn].planets.iter().enumerate() {
            let color = utils::COLORS[p.owner.unwrap_or(0) as usize % utils::COLORS.len()];
            self.voronoi_colors[i * 3 + 0] = color[0];
            self.voronoi_colors[i * 3 + 1] = color[1];
            self.voronoi_colors[i * 3 + 2] = color[2];
        }
    }

    fn update_planet_colours(&mut self) {
        let mut new_vec: Vec<[f32; 3]> = Vec::new();
        let planets_now = self.states[self.turn].planets.iter();
        let planets_later = self.states[(self.turn + 1).min(self.states.len() - 1)]
            .planets
            .iter();

        for (p1, p2) in planets_now.zip(planets_later) {
            new_vec
                .push(utils::COLORS[p1.owner.unwrap_or(0) as usize % utils::COLORS.len()].into());
            new_vec
                .push(utils::COLORS[p2.owner.unwrap_or(0) as usize % utils::COLORS.len()].into());
        }

        self.current_planet_colours = new_vec.concat::<f32>();
    }

    fn update_ship_locations(&mut self) {
        let mut new_sl = Vec::new();
        let mut new_sll = Vec::new();

        let t = Mat3::new(0.2, 0., 0., 0., 0.2, 0.0, 0., -0.5, 0.2);

        for ship in self.states[self.turn].expeditions.iter() {
            let ((o1, a1), (o2, a2)) = self
                .planet_map
                .get(&(ship.origin.clone(), ship.destination.clone()))
                .unwrap()
                .get_for_remaining(ship.turns_remaining as usize);
            new_sl.push((o1 * Mat3::rotate_z(a1)).to_array());
            new_sl.push((o2 * Mat3::rotate_z(a2)).to_array());

            new_sll.push((o1 + t).to_array());
            new_sll.push((o2 + t).to_array());
        }

        self.ship_locations = new_sl.concat();
        self.ship_label_locations = new_sll.concat();

        self.ship_colours = self.states[self.turn]
            .expeditions
            .iter()
            .map(|s| utils::COLORS[s.owner as usize % utils::COLORS.len()])
            .collect::<Vec<[f32; 3]>>()
            .concat();
    }

    fn update_ship_counts(&mut self) {
        self.ship_counts = self.states[self.turn]
            .expeditions
            .iter()
            .map(|s| s.ship_count as usize)
            .collect();
    }

    pub fn get_max_ships(&self) -> usize {
        self.states
            .iter()
            .map(|s| s.expeditions.len())
            .max()
            .unwrap()
    }

    pub fn get_ship_locations(&self) -> Vec<f32> {
        self.ship_locations.clone()
    }

    pub fn get_ship_label_locations(&self) -> Vec<f32> {
        self.ship_label_locations.clone()
    }

    pub fn get_ship_colours(&self) -> Vec<f32> {
        self.ship_colours.clone()
    }

    pub fn get_ship_counts(&self) -> Vec<usize> {
        self.ship_counts.clone()
    }

    pub fn get_voronoi_verts(&self) -> Vec<f32> {
        self.voronoi_vertices.clone()
    }

    pub fn get_voronoi_colours(&self) -> Vec<f32> {
        self.voronoi_colors.clone()
    }

    pub fn get_voronoi_inds(&self) -> Vec<usize> {
        self.voronoi_indices.clone()
    }
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}
