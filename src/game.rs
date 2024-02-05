use std::{collections::{HashMap, HashSet}, f32::consts::LN_10, hash::Hash};
use rand::prelude::*;

use crate::{ugly::*, fish::*, player::*, scan::*, vector::*, collision::*, entity::*, drone::*, closest::*, xorshift::xorshift };

// Assuming you already have the necessary structs and enums from previous translations


pub const INPUT_PER_FISH: usize = 6;
pub const INPUT_PER_DRONE: usize = 4;
pub const GLOBAL_INPUTS: usize = 3;
pub const STATE_SIZE: usize =  INPUT_PER_FISH * 12 + Game::DRONES_PER_PLAYER as usize * INPUT_PER_DRONE * 2 + GLOBAL_INPUTS * 2;
pub const ACTION_SIZE: usize = 16;
//12 * fish x,y,vx,vy,available
    //2* drone x, y, vx, vy

pub struct Action {
    dir: Vector,
    light: bool
}

impl Action {
    pub fn new(a: usize, rot: bool) -> Action {
        let light = a % 2 == 0;
        let angle_index = a / 2;
        let angle = 2.0 * std::f64::consts::PI * angle_index  as f64 / ((ACTION_SIZE as f64 / 2.0));
        let mut  dir = Vector::new(angle.cos() * 10000.0, angle.sin() * 10000.0);
        //eprintln!("{}",  angle_index);
        if rot {
            dir.x = -dir.x;
        }
        Action {
            dir,
            light
        }
    }
}

fn update_ugly_target(ugly: &mut Ugly, players: &[Player]) -> bool {
    let mut targetable_drones = Vec::new();

    for p in players {
        for d in &p.drones {
            //assert!(!d.is_dead_or_dying());
            //assert!(!d.is_light_on());
            //eprintln!("{}", d.pos.distance(ugly.pos));
            if d.pos.in_range(&ugly.pos, if d.is_light_on() { Game::LIGHT_SCAN_RANGE } else { Game::DARK_SCAN_RANGE })
                && !d.is_dead_or_dying(){
                targetable_drones.push(d.clone());   
                //eprintln!("ugly {} chases", ugly.id);
            }
        }
    }

    if targetable_drones.len() > 0 {
        //eprintln!("ugly {} chases", ugly.id);
        let closest_targets = get_closest_to(ugly.pos, &mut targetable_drones.iter());
        ugly.target = closest_targets.get_mean_pos();
        //for drone in closest_targets.list.iter() {
        //     self.times_aggroed[drone.owner.get_index()] += 1;
        // }
        true
    } else {
        ugly.target = None;
        false
    }
}


fn snap_to_ugly_zone(ugly: &mut Ugly) {
    if ugly.pos.y > Game::HEIGHT as f64 - 1.0 {
        ugly.pos.y = Game::HEIGHT as f64 - 1.0;
    } else if ugly.pos.y < Game::UGLY_UPPER_Y_LIMIT {
        ugly.pos.y = Game::UGLY_UPPER_Y_LIMIT;
    }
}

fn snap_to_drone_zone(drone: &mut Drone) {
    if drone.pos.y > Game::HEIGHT as f64 - 1.0 {
        drone.pos = Vector::new(drone.pos.x, Game::HEIGHT as f64 - 1.0);
    } else if drone.pos.y < Game::DRONE_UPPER_Y_LIMIT as f64 {
        drone.pos = Vector::new(drone.pos.x, Game::DRONE_UPPER_Y_LIMIT as f64);
    }

    if drone.pos.x < 0.0 {
        drone.pos = Vector::new(0.0, drone.pos.y);
    } else if drone.pos.x >= Game::WIDTH as f64 {
        drone.pos = Vector::new( Game::WIDTH as f64 - 1.0, drone.pos.y);
    }
}

fn snap_to_fish_zone(fish: &mut Fish) {
    if fish.pos.y > (Game::HEIGHT as f64 - 1.0) as f64 {
        fish.pos = Vector::new(fish.pos.x, (Game::HEIGHT as f64 - 1.0) as f64);
    } else if fish.pos.y > fish.high_y as f64 {
        fish.pos = Vector::new(fish.pos.x, fish.high_y as f64);
    } else if fish.pos.y < fish.low_y as f64 {
        fish.pos = Vector::new(fish.pos.x, fish.low_y as f64);
    }
}


#[derive(Debug, Clone )]
pub struct Game {
    random: xorshift,
    pub players: Vec<Player>,
    pub fishes: Vec<Fish>,
    pub uglies: Vec<Ugly>,
    first_to_scan: HashMap<Scan, i32>,
    first_to_scan_temp: HashMap<Scan, i32>,
    first_to_scan_all_fish_of_color: HashMap<i32, i32>,
    first_to_scan_all_fish_of_color_temp: HashMap<i32, i32>,
    first_to_scan_all_fish_of_type: HashMap<FishType, i32>,
    first_to_scan_all_fish_of_type_temp: HashMap<FishType, i32>,
    entity_count: i32,
    pub game_turn: i32,

    pub  chased_fish_count: [i32; 2],
    pub  times_aggroed: [i32; 2],
    pub  max_turns_spent_with_scan: [i32; 2],
    pub  max_y: [i32; 2],
    pub  turn_saved_fish: [[i32; 12]; 2],
    pub  drones_eaten: i32,
    pub  fish_scanned: i32
}

impl Game {
    pub const COLORS: [&'static str; 4] = ["pink", "yellow", "green", "blue"];
    pub const WIDTH: i32 = 10000;
    pub const HEIGHT: i32 = 10000;
    pub const UGLY_UPPER_Y_LIMIT: f64 = 2500.0;
    pub const DRONE_UPPER_Y_LIMIT: f64 = 0.0;
    pub const DRONE_START_Y: f64 = 500.0;
    pub const COLORS_PER_FISH: i32 = 4;
    pub const DRONE_MAX_BATTERY: i32 = 30;
    pub const LIGHT_BATTERY_COST: i32 = 5;
    pub const DRONE_BATTERY_REGEN: i32 = 1;
    pub const DRONE_MAX_SCANS: i32 = i32::MAX;
    pub const DARK_SCAN_RANGE: f64 = 800.0;
    pub const LIGHT_SCAN_RANGE: f64 = 2000.0;
    pub const UGLY_EAT_RANGE: i32 = 300;
    pub const DRONE_HIT_RANGE: i32 = 200;
    pub const FISH_HEARING_RANGE: f64 = (Game::DARK_SCAN_RANGE + Game::LIGHT_SCAN_RANGE) / 2.0;
    pub const DRONE_MOVE_SPEED: f64 = 600.0;
    pub const DRONE_SINK_SPEED: f64 = 300.0;
    pub const DRONE_EMERGENCY_SPEED: f64 = 300.0;
    pub const DRONE_MOVE_SPEED_LOSS_PER_SCAN: f64 = 0.0;
    pub const FISH_SWIM_SPEED: f64 = 200.0;
    pub const FISH_AVOID_RANGE: f64 = 600.0;
    pub const FISH_FLEE_SPEED: f64 = 400.0;
    pub const UGLY_ATTACK_SPEED: f64 = (Game::DRONE_MOVE_SPEED as f64 * 0.9) as f64;
    pub const UGLY_SEARCH_SPEED: f64 = (Game::UGLY_ATTACK_SPEED as f64 / 2.0) as f64;
    pub const FISH_X_SPAWN_LIMIT: f64 = 1000.0;
    pub const FISH_SPAWN_MIN_SEP: f64 = 1000.0;
    pub const CENTER: Vector = Vector {
        x: (Game::WIDTH as f64 - 1.0) as f64 / 2.0,
        y: (Game::HEIGHT as f64 - 1.0) as f64 / 2.0,
    };
    pub const MAX_TURNS: i32 = 20;


    pub const DRONES_PER_PLAYER: i32 = 1;
    pub const ENABLE_UGLIES: bool = false;
    pub const FISH_WILL_FLEE: bool = false;
    pub const FISH_WILL_MOVE: bool = true;
    pub const SIMPLE_SCANS: bool = true;

    

    pub fn new(seed: i64) -> Game {
        let mut ret = Game {
            random: xorshift::new(seed),
            players: vec![Player::new(); 2],
            fishes: Vec::new(),
            uglies: Vec::new(),
            first_to_scan: HashMap::new(),
            first_to_scan_temp: HashMap::new(),
            first_to_scan_all_fish_of_color: HashMap::new(),
            first_to_scan_all_fish_of_color_temp: HashMap::new(),
            first_to_scan_all_fish_of_type: HashMap::new(),
            first_to_scan_all_fish_of_type_temp: HashMap::new(),
            entity_count: 0,
            game_turn: 0,

            chased_fish_count: [0; 2],
            times_aggroed: [0; 2],
            max_turns_spent_with_scan: [0; 2],
            max_y: [0; 2],
            turn_saved_fish: [[-1; 12], [-1; 12]],
            drones_eaten: 0,
            fish_scanned: 0,
        };
        ret.init();
        ret
    }

    pub fn init(&mut self) {
        self.entity_count = 0;

        self.game_turn = 1;
        self.init_players();
        self.init_fish();
        self.init_uglies();

        for player in &mut self.players {
            if Game::SIMPLE_SCANS {
                player.visible_fishes = self.fishes.iter().map(|f| f.id).collect();
            }
        }
    }

    fn init_uglies(&mut self) {
        let ugly_count = if Game::ENABLE_UGLIES { 1 + self.random.next_in_range(3) } else { 0 };

        for _ in 0..ugly_count {
            let x = self.random.next_in_range((Game::WIDTH / 2) as i32)  as u32;

            let y = (Game::HEIGHT / 2) as u32 +self.random.next_in_range((Game::HEIGHT / 2) as i32) as u32;
            for k in 0..2 {
                let mut ugly = Ugly::new(x as f64, y as f64, self.entity_count);
                if k == 1 {
                    ugly.pos = ugly.pos.hsymmetric(Game::CENTER.x);
                }

                self.uglies.push(ugly);
                self.entity_count += 1;
            }
        }
    }

    fn init_fish(&mut self) {

        for col in (0..Game::COLORS_PER_FISH).step_by(2) {
            for type_idx in 0..FishType::variants().len() {
                let mut position_found = false;
                let mut iterations = 0;
                let mut x = 0;
                let mut y = 0;

                let mut low_y = (Game::HEIGHT / 4) as i32;
                let mut high_y = Game::HEIGHT as i32;

                while !position_found {
                    x = self.random.next_in_range((Game::WIDTH - Game::FISH_X_SPAWN_LIMIT as i32 * 2) as i32) as u32 + Game::FISH_X_SPAWN_LIMIT as u32;
                    //eprintln!("{}", (Game::WIDTH as f64 - Game::FISH_X_SPAWN_LIMIT * 2.0));
                    if type_idx == 0 {
                        y = (1.0 * Game::HEIGHT as f64 / 4.0) as i32 + Game::FISH_SPAWN_MIN_SEP as i32;
                        low_y = (1.0 * Game::HEIGHT as f64 / 4.0) as i32;
                        high_y = (2.0 * Game::HEIGHT as f64 / 4.0) as i32;
                    } else if type_idx == 1 {
                        y = (2.0 * Game::HEIGHT as f64 / 4.0) as i32 + Game::FISH_SPAWN_MIN_SEP as i32;
                        low_y = (2.0 * Game::HEIGHT as f64 / 4.0) as i32;
                        high_y = (3.0 * Game::HEIGHT as f64 / 4.0) as i32
                    } else {
                        y = (3.0 * Game::HEIGHT as f64 / 4.0) as i32 + Game::FISH_SPAWN_MIN_SEP as i32;
                        low_y = (3.0 * Game::HEIGHT as f64 / 4.0) as i32;
                        high_y = (4.0 * Game::HEIGHT as f64 / 4.0) as i32;
                    }
                    y += (self.random.next_in_range((Game::HEIGHT / 4 - Game::FISH_SPAWN_MIN_SEP as i32 * 2) as i32) as u32) as i32;

                    let final_x = x;
                    let final_y = y;
                    let too_close = self.fishes.iter().any(|other| other.pos.in_range(&Vector::new(final_x as f64, final_y as f64), Game::FISH_SPAWN_MIN_SEP));
                    let too_close_to_center = (Game::CENTER.x - x as f64).abs() <= Game::FISH_SPAWN_MIN_SEP;
                    if !too_close && !too_close_to_center || iterations > 100 {
                        position_found = true;
                    }
                    iterations += 1;
                }
                let mut f = Fish::new(x as f64, y as f64, &FishType::variants()[type_idx], col, self.entity_count, low_y, high_y);

                let snapped = (self.random.next_in_range(7)) as f64 * std::f64::consts::FRAC_PI_4;
                let direction = Vector::new(snapped.cos(), snapped.sin());

                if Game::FISH_WILL_MOVE {
                    f.speed = direction.mult(Game::FISH_SWIM_SPEED).round();
                }

               
                //TODO
                self.entity_count += 1;

                let other_pos = f.pos.hsymmetric(Game::CENTER.x);
                let other_speed = f.speed.hsymmetric(0.0);
                let mut o = Fish::new(other_pos.x, other_pos.y, &FishType::variants()[type_idx], col + 1, self.entity_count, f.low_y, f.high_y);
                o.speed = other_speed;
                self.fishes.push(o);
                self.fishes.push(f);
                self.entity_count += 1;
            }
        }
    }
        
    fn init_players(&mut self) {
        for i in 0..2 {
            self.players[i].index = i as i32;
        }
        let idxs = [0, 2, 1, 3];
        let mut idx_idx = 0;
        for _ in 0..Game::DRONES_PER_PLAYER {
            let x = Game::WIDTH / (Game::DRONES_PER_PLAYER * 2 + 1) * (idxs[idx_idx] + 1);
            idx_idx += 1;
            for player in &mut self.players {
                let mut drone = Drone::new(x as f64, Game::DRONE_START_Y as f64, self.entity_count, &player);
                drone.move_command = Some(Vector::new(0.0, 0.0));
                if player.get_index() == 1 {
                    drone.pos = drone.pos.hsymmetric(Game::CENTER.x);
                }

                player.drones.push(drone);
                self.entity_count += 1;
            }
        }
    }

    pub fn reset_game_turn_data(&mut self) {
        for player in &mut self.players {
            player.reset();
        }
    }

    

    fn move_entities(&mut self) {
        // Move drones and handle collisions with uglies
        for player in self.players.iter_mut() {
            for drone in player.drones.iter_mut() {
                if drone.dead {
                    continue;
                }
            
                // NOTE: the collision code does not take into account the snap to map borders
                for ugly in  self.uglies.iter() {
                    let col = get_collision(drone, ugly);
                    if col >= 0.0 {
                        drone.dying = true;
                        drone.scans.clear();
                        drone.die_at = col;
                        //self.drones_eaten += 1;
                        // If two uglies hit the drone, let's just keep the first collision, it matters not.
                        //println!("drone was eaten");
                        break;
                    }
                }
            }
        }

        // Move drones and snap to drone zone
        for player in  self.players.iter_mut() {
            for drone in player.drones.iter_mut() {
                let speed = drone.get_speed();
                drone.pos = drone.pos.add(speed);
                snap_to_drone_zone(drone);
            }
        }

        // Move fishes and snap to fish zone
        for fish in  self.fishes.iter_mut() {
            fish.pos = fish.pos.add(fish.get_speed());
            snap_to_fish_zone(fish);
        }

        // Remove fishes that went out of bounds
        let fish_to_remove: Vec<i32> =  self.fishes
            .iter()
            .filter(|fish| fish.get_pos().x > Game::WIDTH as f64 - 1.0 || fish.get_pos().x < 0.0)
            .map(|f| f.id)
            .collect();


        self.fishes.retain(|fish| !fish_to_remove.iter().any(|f_id| *f_id == fish.id));

        // Reset fleeing information for remaining fishes
        for fish in self.fishes.iter_mut() {
            fish.fleeing_from_player = None;
        }

        // Move uglies and snap to ugly zone
        for ugly in self.uglies.iter_mut() {
            ugly.pos = ugly.pos.add(ugly.speed);
            snap_to_ugly_zone(ugly);
        }
    }

    fn snap_to_ugly_zone(ugly: &mut Ugly) {
        if ugly.pos.y > Game::HEIGHT as f64 - 1.0 {
            ugly.pos = Vector::new(ugly.pos.x, Game::HEIGHT as f64 - 1.0);
        } else if ugly.pos.y < Game::UGLY_UPPER_Y_LIMIT as f64 {
            ugly.pos = Vector::new(ugly.pos.x, Game::UGLY_UPPER_Y_LIMIT as f64);
        }
    }

    fn update_ugly_speeds(&mut self) {
        //TODO FIND BETTER SOLUTION
        let uglies_clone = self.uglies.clone(); 
        for ugly in self.uglies.iter_mut() {
            if let Some(target) = ugly.target {
                //eprintln!("uglyka {} chases", ugly.id);
                let mut attack_vec = Vector::from_points(ugly.pos, target);
                if attack_vec.length() > Game::UGLY_ATTACK_SPEED as f64 {
                    attack_vec = attack_vec.normalize().mult(Game::UGLY_ATTACK_SPEED as f64).round();
                }
                ugly.speed = attack_vec;
            } else {
                if ugly.speed.length() > Game::UGLY_SEARCH_SPEED as f64 {
                    ugly.speed = ugly.speed.normalize().mult(Game::UGLY_SEARCH_SPEED as f64).round();
                }

                if !ugly.speed.is_zero() {
                    let closest_uglies = get_closest_to(ugly.pos, uglies_clone.iter().filter(|u| u.id != ugly.id));
                    if !closest_uglies.list.is_empty() && closest_uglies.distance <= Game::FISH_AVOID_RANGE as f64 {
                        let avoid = closest_uglies.get_mean_pos().unwrap();
                        let avoid_dir = Vector::from_points(avoid, ugly.pos).normalize();
                        if !avoid_dir.is_zero() {
                            ugly.speed = avoid_dir.mult(Game::FISH_SWIM_SPEED as f64).round();
                        }
                    }
                }

                let next_pos = ugly.pos.add(ugly.speed);

                if (next_pos.x < 0.0 && next_pos.x < ugly.pos.x) || (next_pos.x > Game::WIDTH as f64 - 1.0 && next_pos.x > ugly.pos.x) {
                    ugly.speed = ugly.speed.hsymmetric(0.0);
                }

                if (next_pos.y < Game::UGLY_UPPER_Y_LIMIT as f64 && next_pos.y < ugly.pos.y)
                    || (next_pos.y > Game::HEIGHT as f64 - 1.0 && next_pos.y > ugly.pos.y)
                {
                    ugly.speed = ugly.speed.vsymmetric(0.0);
                }
            }
        }
    }

    fn update_ugly_targets(&mut self) {
        for ugly in &mut self.uglies {
            let found_target = update_ugly_target(ugly, &self.players);
            ugly.found_target = found_target;
        }
    }

    fn update_fish(&mut self) {
        let fishes_copy = self.fishes.clone(); 
        for fish in &mut self.fishes {
            fish.is_fleeing = false;

            let mut flee_from: Option<Vector> = None;
            if Game::FISH_WILL_FLEE {
                let closest_drones = get_closest_to(
                    fish.pos,
                    self.players.iter().flat_map(|p| p.drones.iter().filter(|d| d.is_engine_on() && !d.dead)),
                );

                if !closest_drones.list.is_empty() && closest_drones.distance <= Game::FISH_HEARING_RANGE as f64 {
                    flee_from = closest_drones.get_mean_pos();
                    let mut fleeing_from_player: Option<i32> = None;
                    for d in closest_drones.list.iter() {
                        let idx = d.owner as i32;
                        if fleeing_from_player.is_none() || fleeing_from_player.unwrap() == idx {
                            fleeing_from_player = Some(idx);
                        } else {
                            fleeing_from_player = Some(i32::MAX);
                        }
                    }
                    fish.fleeing_from_player = fleeing_from_player;
                }
            }

            if let Some(flee_from_vec) = flee_from {
                let flee_dir = Vector::from_points(flee_from_vec, fish.pos).normalize();
                let flee_vec = flee_dir.mult(Game::FISH_FLEE_SPEED as f64);
                fish.speed = flee_vec.round();
                fish.is_fleeing = true;
            } else {
                let mut swim_vec = fish.speed.normalize().mult(Game::FISH_SWIM_SPEED as f64);
                //TODO REMOVE CLONE
                
                let closest_fishes = get_closest_to(fish.pos, fishes_copy.iter().filter(|f| f.id != fish.id));

                if !closest_fishes.list.is_empty() && closest_fishes.distance <= Game::FISH_AVOID_RANGE as f64 {
                    //eprintln!(" fish {} collided  with {}", fish.id, closest_fishes.get().unwrap().id);
                    //eprintln!("distance: {}, calc distance: {}", closest_fishes.distance,  fish.pos.distance( closest_fishes.get().unwrap().pos));
                   
                    //eprintln!("{} {} {} {}", fish.pos.x, fish.pos.y , closest_fishes.get().unwrap().pos.x, closest_fishes.get().unwrap().pos.y);
                    let avoid = closest_fishes.get_mean_pos().unwrap();
                    let avoid_dir = Vector::from_points(avoid, fish.pos).normalize();
                    swim_vec = avoid_dir.mult(Game::FISH_SWIM_SPEED as f64);
                }

                let next_pos = fish.pos.add(swim_vec);

                if (next_pos.x < 0.0 && next_pos.x < fish.pos.x)
                    || (next_pos.x > Game::WIDTH as f64 - 1.0 && next_pos.x > fish.pos.x)
                {
                    swim_vec = swim_vec.hsymmetric(0.0);
                }

                let y_highest = f64::min(Game::HEIGHT as f64 - 1.0, fish.high_y as f64);

                if (next_pos.y < fish.low_y as f64 && next_pos.y < fish.pos.y)
                    || (next_pos.y > y_highest && next_pos.y > fish.pos.y)
                {
                    swim_vec = swim_vec.vsymmetric(0.0);
                }
                fish.speed = swim_vec.epsilon_round().round();
            }
        }
    }



    fn update_drones(&mut self) {
        for player in &mut self.players{
            for drone in player.drones.iter_mut() {
                let move_speed =
                    (Game::DRONE_MOVE_SPEED - Game::DRONE_MOVE_SPEED * Game::DRONE_MOVE_SPEED_LOSS_PER_SCAN * drone.scans.len() as f64) as f64;

                if drone.dead {
                    let float_vec = Vector::new(0.0, -1.0).mult(Game::DRONE_EMERGENCY_SPEED as f64);
                    drone.speed = float_vec;
                } else if let Some(move_pos) = &drone.move_command {
                    let move_vec = Vector::from_points(drone.pos, *move_pos);

                    if move_vec.length() > move_speed {
                        drone.speed = move_vec.normalize().mult(move_speed).round();
                    } else {
                        drone.speed = move_vec.round();
                    }
                } else if drone.pos.y < Game::HEIGHT as f64 - 1.0 {
                    let sink_vec = Vector::new(0.0, 1.0).mult(Game::DRONE_SINK_SPEED as f64);
                    drone.speed = sink_vec;
                }
            }
        }
    }

    pub fn perform_game_update(&mut self, frame_idx: i32) {
        self.clear_player_info();
        self.do_batteries();

        // Update speeds
        self.update_drones();

        // Move
        self.move_entities();

        // Target
        self.update_ugly_targets();

        // Scans
        self.do_scan();
        self.do_report();

        // Upkeep
        self.upkeep_drones();

        // Update speeds
        self.update_fish();
        self.update_ugly_speeds();

        if self.is_game_over() {
            self.compute_score_on_end();
        }

        self.game_turn += 1;
    }

    fn clear_player_info(&mut self) {
        for player in self.players.iter_mut() {
            player.visible_fishes.clear();
        }
    }

    fn upkeep_drones(&mut self) {
        for player in self.players.iter_mut() {
            for drone in player.drones.iter_mut() {
                if drone.dying {
                    drone.dead = true;
                    drone.dying = false;
                } else if drone.dead && drone.get_y() == Game::DRONE_UPPER_Y_LIMIT as f64 {
                    drone.dead = false;
                }

                // Stats
                if drone.scans.is_empty() {
                    drone.turns_spent_with_scan = 0;
                } else {
                    drone.turns_spent_with_scan += 1;
                    if drone.turns_spent_with_scan > drone.max_turns_spent_with_scan {
                        drone.max_turns_spent_with_scan = drone.turns_spent_with_scan;
                    }
                }
                if drone.pos.y > drone.max_y as f64 {
                    drone.max_y = drone.pos.y as i32;
                }
            }
        }
    }

    fn do_batteries(&mut self) {
        for player in self.players.iter_mut() {
            for drone in player.drones.iter_mut() {
                if drone.dead {
                    drone.light_on = false;
                    continue;
                }

                if drone.light_switch && drone.battery >= Game::LIGHT_BATTERY_COST && !drone.dead {
                    drone.light_on = true;
                } else {
                    drone.light_on = false;
                }

                if drone.is_light_on() {
                    drone.drain_battery();
                } else {
                    drone.recharge_battery();
                }
            }
        }
    }

    fn do_scan(&mut self) {
        for player in self.players.iter_mut() {
            for drone in player.drones.iter_mut() {
                if drone.is_dead_or_dying() {
                    continue;
                }

                let scannable_fish: Vec<&Fish> = self.fishes.iter()
                    .filter(|fish| fish.pos.in_range(&drone.pos, if drone.is_light_on() { Game::LIGHT_SCAN_RANGE } else { Game::DARK_SCAN_RANGE }))
                    .collect();

                for fish in scannable_fish.iter() {
                    player.visible_fishes.insert(fish.id);

                    if drone.scans.len() < Game::DRONE_MAX_SCANS as usize {
                        let scan = Scan::new_from_fish(&fish);
                        if !player.scans.contains(&scan) {
                            if !drone.scans.contains(&scan) {
                                drone.scans.insert(scan);
                                drone.fishes_scanned_this_turn.insert(fish.id);
                                //eprintln!("scanned{}", fish.id);
                            }
                        }
                    }
                }

                if Game::SIMPLE_SCANS {
                    player.visible_fishes = self.fishes.iter().map(|f| f.id).collect();
                }
            }
        }
    }

    fn apply_scans_for_report(&mut self, player_index: usize, drone_index: usize) -> bool {
        let player = &mut self.players[player_index];
        for scan in &player.scans {
            if !self.first_to_scan.contains_key(scan) {
                if self.first_to_scan_temp.contains_key(scan) {
                    // Opponent also completed this report this turn, nobody gets the bonus
                    self.first_to_scan_temp.remove(scan);
                } else {
                    self.first_to_scan_temp.insert(scan.clone(), player_index as i32);
                }
            }
            let fish_index = scan.color * 3 + scan.fish_type as i32;
            self.turn_saved_fish[player_index as usize][fish_index as usize] = self.game_turn;
            self.fish_scanned += 1;
        }
        if ! player.drones[drone_index].scans.is_empty() {
            player.count_fish_saved.push( player.drones[drone_index].scans.len() as i32);
        }
        let size_before = player.scans.len();
        player.scans.extend( player.drones[drone_index].scans.drain());

        // TODO NICER WAY
        let scans_clone = player.drones[drone_index].scans.clone();
        for other in player.drones.iter_mut() {
            if drone_index != other.id as usize {
                other.scans.retain(|s| !scans_clone.contains(s));
            }
        }
        size_before < player.scans.len()
    }

    fn detect_first_to_combo_bonuses(&mut self, player_index: usize) {
        for &fish_type in FishType::variants().iter() {
            if !self.first_to_scan_all_fish_of_type.contains_key(&fish_type) {
                if self.player_scanned_all_fish_of_type(player_index, fish_type) {
                    if self.first_to_scan_all_fish_of_type_temp.contains_key(&fish_type) {
                        // Opponent also completed this report this turn, nobody gets the bonus
                        self.first_to_scan_all_fish_of_type_temp.remove(&fish_type);
                    } else {
                        self.first_to_scan_all_fish_of_type_temp.insert(fish_type, player_index as i32);
                    }
                }
            }
        }

        for color in 0..Game::COLORS_PER_FISH {
            if !self.first_to_scan_all_fish_of_color.contains_key(&color) {
                if self.player_scanned_all_fish_of_color(player_index, color) {
                    if self.first_to_scan_all_fish_of_color_temp.contains_key(&color) {
                        // Opponent also completed this report this turn, nobody gets the bonus
                        self.first_to_scan_all_fish_of_color_temp.remove(&color);
                    } else {
                        self.first_to_scan_all_fish_of_color_temp.insert(color, player_index as i32);
                    }
                }
            }
        }
    }

    fn do_report(&mut self) {
         for player_index in 0..self.players.len() { 
            let mut points_scored = false;
            for drone_index in 0..self.players[player_index].drones.len() {
                if self.players[player_index].drones[drone_index].is_dead_or_dying() {
                    continue;
                }
                if Game::SIMPLE_SCANS || (!self.players[player_index].drones[drone_index].scans.is_empty() && self.players[player_index].drones[drone_index].pos.y <= Game::DRONE_START_Y as f64) {
                    let drone_scored = self.apply_scans_for_report(player_index, drone_index);
                    points_scored |= drone_scored;
                    if drone_scored {
                        self.players[player_index].drones[drone_index].did_report = true;
                    }
                }
            }
            self.detect_first_to_combo_bonuses(player_index);
        }

        self.persist_first_to_scan_bonuses();
        for i in 0..2 {
            self.players[i].points = self.compute_player_score(i);
        }
    }

    fn persist_first_to_scan_bonuses(&mut self) {
        let mut player_scans_map: HashMap<i32, Vec<Scan>> = HashMap::new();

        for (scan, &player_index) in &self.first_to_scan_temp {
            self.first_to_scan.entry(scan.clone()).or_insert(player_index);

            player_scans_map.entry(player_index)
                .or_insert_with(|| Vec::new())
                .push(scan.clone());
        }

        for (player_name, player_scans) in player_scans_map {
            //let summary_string = player_scans.iter()
            //    .map(|scan| scan.fish_id.to_string())
            //    .collect::<Vec<String>>()
            //    .join(", ");
        
            /*if player_scans.len() == 1 {
                eprintln!("{} was the first to save the scan of creature {}", player_name, summary_string);
            } else {
                eprintln!(
                        "{} was the first to save the scans of {} creatures: {}", 
                        player_name, 
                        player_scans.len(), 
                        summary_string);
            }*/
        }

        for (fish_type, &player_index) in &self.first_to_scan_all_fish_of_type_temp {
            self.first_to_scan_all_fish_of_type.entry(*fish_type).or_insert(player_index);
            //eprintln!("player{} scanned all of fish type{}", player_index, *fish_type as usize);
        }

        for (color, &player_index) in &self.first_to_scan_all_fish_of_color_temp {
            self.first_to_scan_all_fish_of_color.entry(*color).or_insert(player_index);
            //eprintln!("player{} scanned all of fish color{}", player_index, Game::COLORS[*color as usize]);
        }

        self.first_to_scan_temp.clear();
        self.first_to_scan_all_fish_of_color_temp.clear();
        self.first_to_scan_all_fish_of_type_temp.clear();
    }

    fn player_scanned(&self,  player_index: usize, fish: &Fish) -> bool {
        self.player_scanned_scan(player_index, &Scan::new_from_fish(fish))
    }

    fn player_scanned_scan(&self, player_index: usize, scan: &Scan) -> bool {
        self.players[player_index].scans.contains(scan)
    }

    fn has_scanned_all_remaining_fish(&self,  player_index: usize) -> bool {
        self.fishes.iter().all(|fish| self.player_scanned(player_index, fish))
    }

    fn has_fish_escaped(&self, scan: &Scan) -> bool {
        !self.fishes.iter().any(|fish| fish.color == scan.color && fish.fish_type == scan.fish_type)
    }

    fn is_fish_scanned_by_player_drone(&self, scan: &Scan,  player_index: usize) -> bool {
        self.players[player_index].drones.iter().any(|drone| drone.scans.contains(scan))
    }

    fn is_type_combo_still_possible(&self, player_index: usize, fish_type: &FishType) -> bool {
        if self.player_scanned_all_fish_of_type(player_index, *fish_type) {
            return false;
        }

        for color in 0..Game::COLORS_PER_FISH {
            let scan = Scan::new_from_type_color(*fish_type, color);

            if self.has_fish_escaped(&scan) && !self.is_fish_scanned_by_player_drone(&scan, player_index) && !self.player_scanned_scan(player_index, &scan) {
                return false;
            }
        }
        true
    }

    fn is_color_combo_still_possible(&self, player_index: usize, color: i32) -> bool {
        if self.player_scanned_all_fish_of_color(player_index, color) {
            return false;
        }

        for fish_type in FishType::variants() {
            let scan = Scan::new_from_type_color(*fish_type, color);
            if self.has_fish_escaped(&scan) && !self.is_fish_scanned_by_player_drone(&scan, player_index) && !self.player_scanned_scan(player_index, &scan) {
                return false;
            }
        }
        true
    }

    pub fn compute_max_player_score(&self,  player_index: usize) -> i32 {
        let scanned = -1;
        let mut total = self.compute_player_score(player_index);
       
        

        /*eprintln!("calc max........................{}",  self.players[player_index].scans.len());
        eprintln!("player score: {}", total );
        for s in &self.players[player_index].scans {
            assert!(self.player_scanned_scan(player_index, &s)); 
            eprintln!("player scanned {} {}", s.color, s.fish_type as i32)

        }*/
        for color in 0..Game::COLORS_PER_FISH {
            for fish_type in FishType::variants() {
                let scan = Scan::new_from_type_color(*fish_type, color);
                if !self.player_scanned_scan(player_index, &scan) {
                    //eprintln!("not scanned{} {}", color, *fish_type as i32);
                    if self.is_fish_scanned_by_player_drone(&scan, player_index) || !self.has_fish_escaped(&scan) {
                        //eprintln!("fish type: {}", (*fish_type as i32 + 1));
                        total += (*fish_type as i32 + 1);
                        //eprintln!("{}", total);
                        if !self.first_to_scan.contains_key(&scan) {
                            total += (*fish_type as i32 + 1);
                            //eprintln!("{}", total);
                        }
                    }
                }
                else {
                    //eprint!("player scanned {} {}", scan.color, scan.fish_type as i32)
                }
            }
        }
       
        for fish_type in FishType::variants() {
            if self.is_type_combo_still_possible(player_index, &fish_type) {
                total += Game::COLORS_PER_FISH as i32;
                if !self.first_to_scan_all_fish_of_type.contains_key(&fish_type) {
                    total += Game::COLORS_PER_FISH as i32;
                }
            }
        }
        for color in 0..Game::COLORS_PER_FISH {
            if self.is_color_combo_still_possible(player_index, color) {
                total += FishType::variants().len() as i32;
                if !self.first_to_scan_all_fish_of_color.contains_key(&color) {
                    total += FishType::variants().len() as i32;
                }
            }
        }
        assert!(total <= 96);
        total
    }

    pub fn is_game_over(&self) -> bool {
        if self.both_players_have_scanned_all_remaining_fish() {
            true
        } else {
            //eprintln!("{} vs {}", self.players[0].points, self.compute_max_player_score(0));
            self.game_turn >= Game::MAX_TURNS
                || self.compute_max_player_score(0) < self.players[1].points
                || self.compute_max_player_score(1) < self.players[0].points
        }
    }

    fn both_players_have_scanned_all_remaining_fish(&self) -> bool {
        self.players.iter().all(|player| self.has_scanned_all_remaining_fish(player.index as usize))
    }

    fn player_scanned_all_fish_of_color(&self, player_index: usize, color: i32) -> bool {
        FishType::variants().iter().all(|&fish_type| self.player_scanned_scan(player_index, &Scan::new_from_type_color(fish_type, color)))
    }

    pub fn compute_player_score(&self, player_index: usize) -> i32 {
        let mut total = 0;
        for scan in &self.players[player_index].scans {
            total += scan.fish_type as i32 + 1;
            if self.first_to_scan.contains_key(scan) && *self.first_to_scan.get(scan).unwrap() == player_index as i32 {
                total += scan.fish_type as i32 + 1;
            }
        }

        for fish_type in FishType::variants() {
            if self.player_scanned_all_fish_of_type(player_index, *fish_type) {
                total += Game::COLORS_PER_FISH as i32;
            }
            if self.first_to_scan_all_fish_of_type.contains_key(fish_type) && *self.first_to_scan_all_fish_of_type. get(fish_type).unwrap() == player_index as i32 {
                total += Game::COLORS_PER_FISH as i32;
            }
        }

        for color in 0..Game::COLORS_PER_FISH {
            if self.player_scanned_all_fish_of_color(player_index, color) {
                total += FishType::variants().len() as i32;
            }
            
            if self.first_to_scan_all_fish_of_color.contains_key(&color) && *self.first_to_scan_all_fish_of_color. get(&color).unwrap() == player_index as i32 {
                total += FishType::variants().len() as i32;
            }
        }

        total
    }

    fn player_scanned_all_fish_of_type(&self, player_index: usize, fish_type: FishType) -> bool {
        (0..Game::COLORS_PER_FISH).all(|color| self.player_scanned_scan(player_index, &Scan::new_from_type_color(fish_type, color)))
    }

    fn compute_score_on_end(&mut self) {
         for player_index in 0..self.players.len() {
            for drone_index in 0..self.players[player_index].drones.len() {
                self.apply_scans_for_report(player_index, drone_index);
            }
        
            self.detect_first_to_combo_bonuses(player_index);
        }

        self.persist_first_to_scan_bonuses();

        for i in 0..2 {

            let score = self.compute_player_score(i);
            self.players[i].set_score(score);
            self.players[i].points = score;

        }
    }

    fn has_first_to_scan_bonus(&self, player: &Player, scan: &Scan) -> bool {
        self.first_to_scan.get(scan).map_or(-1, |&val| val) == player.get_index()
    }

    fn has_first_to_scan_all_fish_of_type(&self,player: &Player, fish_type: &FishType) -> bool {
        self.first_to_scan_all_fish_of_type.get(fish_type).map_or(-1, |&val| val) == player.get_index()
    }

    fn has_first_to_scan_all_fish_of_color(&self,player: &Player, color: i32) -> bool {
        self.first_to_scan_all_fish_of_color.get(&color).map_or(-1, |&val| val) == player.get_index()
    }



    pub fn encode(&self, player: usize) -> [f32; STATE_SIZE] {

        let mut inputs: [f32; STATE_SIZE] = [0.0; STATE_SIZE];

        for f in &self.fishes {
            let id =  (f.color + f.fish_type  as i32 * Game::COLORS_PER_FISH) as usize;
            inputs[id * INPUT_PER_FISH + 0] = ( (if player == 0 {f.get_x()} else {9999.0 - f.get_x()}) / 10000.0) as f32;
            inputs[id * INPUT_PER_FISH + 1] = (f.get_y() / 10000.0) as f32;
            inputs[id * INPUT_PER_FISH + 2] = ( (if player == 0{f.speed.x} else {-f.speed.x})  / 400.0) as f32;
            inputs[id * INPUT_PER_FISH + 3] = (f.speed.y / 400.0) as f32;

            let is_scanned_player = self.is_fish_scanned_by_player_drone(&Scan::new_from_type_color(f.fish_type, f.color), player as usize);
            let is_scanned_opp = self.is_fish_scanned_by_player_drone(&Scan::new_from_type_color(f.fish_type, f.color), 1 - player as usize);
           
           
            inputs[id * INPUT_PER_FISH + 4] = if is_scanned_player {1.0} else {0.0};
            inputs[id * INPUT_PER_FISH + 5] = if is_scanned_opp {1.0} else {0.0};
        }

        for player_id in 0..2 {
            let pid =  (if player_id == 0 {player} else {1 - player}) as usize;
            for (i, d) in self.players[pid].drones.iter().enumerate() {
                inputs[player_id as usize * INPUT_PER_DRONE + 0 + INPUT_PER_FISH * 12] = ( (if player == 0 {d.get_x()} else {9999.0 - d.get_x()}) / 10000.0) as f32;
                inputs[player_id as usize * INPUT_PER_DRONE + 1 + INPUT_PER_FISH * 12] = (d.get_y() / 10000.0) as f32;
                inputs[player_id as usize * INPUT_PER_DRONE + 2 + INPUT_PER_FISH * 12] = (d.battery as f32 / 30.0) as f32;
                inputs[player_id as usize * INPUT_PER_DRONE + 3 + INPUT_PER_FISH * 12] = if d.is_light_on() {1.0} else {0.0};
            }
        }
        

        let dif = (INPUT_PER_FISH * 12 + 2 * INPUT_PER_DRONE * (Game::DRONES_PER_PLAYER as usize)) as usize;
        
        for player_id in 0..2 {
            let pid =  (if player_id == 0 {player} else {1 - player}) as usize;

            //for f in &self.fishes {
            //    let id =  (f.color + f.fish_type  as i32 * Game::COLORS_PER_FISH) as usize;
            //    let is_scanned = self.is_fish_scanned_by_player_drone(&Scan::new_from_type_color(f.fish_type, f.color), pid);
            //    inputs[dif + id + player_id as usize * GLOBAL_INPUTS] = if is_scanned {1.0} else {0.0};     
            //}

            inputs[dif + player_id as usize  * GLOBAL_INPUTS + 0] = self.compute_player_score(pid) as f32 / 96.0;
            inputs[dif + player_id as usize  * GLOBAL_INPUTS + 1] = self.compute_max_player_score(pid) as f32 / 96.0;
            inputs[dif + player_id as usize  * GLOBAL_INPUTS + 2] = self.game_turn as f32 / Game::MAX_TURNS as f32;
        }
        inputs
    }

    pub fn score(&self, player: usize) -> f32 {
        if self.is_game_over() {
            if self.compute_player_score(player) > self.compute_player_score( 1 - player){
                1.0
            } else if self.compute_player_score(player) < self.compute_player_score( 1 - player) {
                -1.0
            } else {
                0.0
            }
        } else {
            0.0
            //(self.compute_player_score(player) - self.compute_player_score(1 - player)) as f32 / (96.0)
        }  
    } 

    pub fn step(&mut self, actions: [Action; 2]) {
        for i in 0..2 {
            self.players[i].drones[0].move_command = Some(self.players[i].drones[0].pos.add(actions[i].dir));
            self.players[i].drones[0].light_switch = actions[i].light;
        }  
        self.perform_game_update(0);
    }

    // Add other methods and properties here...
}
