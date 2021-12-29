#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Expedition {
    pub id: u64,
    pub ship_count: u64,
    pub origin: String,
    pub destination: String,
    pub owner: u64,
    pub turns_remaining: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Planet {
    pub ship_count: u64,
    pub x: f32,
    pub y: f32,
    pub owner: Option<u32>,
    pub name: String,
}

use std::hash::{Hash, Hasher};
use std::mem;

impl Hash for Planet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe {
            let x: u32 = mem::transmute_copy(&self.x);
            let y: u32 = mem::transmute_copy(&self.y);
            state.write_u32(x);
            state.write_u32(y);
        }
    }
}

impl PartialEq for Planet {
    fn eq(&self, other: &Self) -> bool {
        (self.x - other.x).abs() < 0.0001 && (self.y - other.y).abs() < 0.0001
    }
}
impl Eq for Planet {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub planets: Vec<Planet>,
    pub expeditions: Vec<Expedition>,
}
