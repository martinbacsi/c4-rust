use std::collections::HashSet;

use crate::{player::*, vector::*, scan::*, entity::*, game::{self, *}};

// Assuming you already have the Vector, Player, and Scan structs from the previous examples

#[derive(Debug, Clone)]
pub struct Drone {
    pub id: i32,
    pub owner: i32,
    pub pos: Vector,
    pub speed: Vector,
    pub light: i32,
    pub battery: i32,
    pub scans: HashSet<Scan>,
    pub fishes_scanned_this_turn: HashSet<i32>,
    pub light_switch: bool,
    pub light_on: bool,
    pub dying: bool,
    pub dead: bool,
    pub did_report: bool,
    pub die_at: f64,
    pub message: String,
    pub max_turns_spent_with_scan: i32,
    pub turns_spent_with_scan: i32,
    pub max_y: i32,
    pub move_command: Option<Vector>
}

impl Entity for Drone {
    fn get_pos(&self) -> Vector {
        self.pos
    }

    fn get_speed(&self) -> Vector {
        self.speed
    }
}

impl Entity for &Drone {
    fn get_pos(&self) -> Vector {
        self.pos
    }

    fn get_speed(&self) -> Vector {
        self.speed
    }
}

impl Drone {
    pub fn new(x: f64, y: f64, id: i32, owner: &Player) -> Drone {
        Drone {
            id,
            owner: owner.get_index(),
            pos: Vector::new(x, y),
            speed: Vector::ZERO,
            light: 0,
            battery: Game::DRONE_MAX_BATTERY, // Assuming initial battery value
            scans: HashSet::new(),
            fishes_scanned_this_turn: HashSet::new(),
            light_switch: false,
            light_on: false,
            dying: false,
            dead: false,
            did_report: false,
            die_at: 0.0,
            message: String::new(),
            max_turns_spent_with_scan: 0,
            turns_spent_with_scan: 0,
            max_y: 0,
            move_command: Option::None
        }
    }

    pub fn is_engine_on(&self) -> bool {
        self.move_command.is_some()
    }

    pub fn is_light_on(&self) -> bool {
        self.light_on
    }

    pub fn drain_battery(&mut self) {
        self.battery -= Game::LIGHT_BATTERY_COST; // Assuming LIGHT_BATTERY_COST is 1
    }

    pub fn recharge_battery(&mut self) {
        if self.battery < Game::DRONE_MAX_BATTERY {
            self.battery += Game::DRONE_BATTERY_REGEN; // Assuming DRONE_BATTERY_REGEN is 1
        }

        if self.battery >= Game::DRONE_MAX_BATTERY {
            self.battery = Game::DRONE_MAX_BATTERY;
        }
    }

    pub fn is_dead_or_dying(&self) -> bool {
        self.dying || self.dead
    }

    pub fn get_x(&self) -> f64 {
        self.pos.x
    }

    pub fn get_y(&self) -> f64 {
        self.pos.y
    }

    pub fn get_speed(&self) -> Vector {
        self.speed
    }
}