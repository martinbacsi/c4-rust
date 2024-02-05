use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use crate::{fish::*, game::Game};
// Assuming you already have the FishType and Game structs from the previous examples


#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Scan {
    pub fish_type: FishType,
    pub color: i32,
}

impl Scan {
    pub fn new_from_fish(fish: &Fish) -> Scan {
        Scan {
            fish_type: fish.fish_type.clone(),
            color: fish.color,
        }
    }

    pub fn new_from_type_color(fish_type: FishType, color: i32) -> Scan {
        Scan {
            fish_type,
            color,
        }
    }

    //pub fn to_input_string(&self) -> String {
    //    self.fish_id.to_string()
    //}
}
