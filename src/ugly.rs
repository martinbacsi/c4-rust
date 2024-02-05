use crate::{vector::*, entity::* };
#[derive(Debug, Clone)]
pub struct Ugly {
    pub id: i32,
    pub pos: Vector,
    pub speed: Vector,
    pub target: Option<Vector>,
    pub found_target: bool,
}



impl Ugly {
    pub fn new(x: f64, y: f64, id: i32) -> Ugly {
        Ugly {
            id,
            pos: Vector::new(x, y),
            speed: Vector::ZERO,
            target: None,
            found_target: false,
        }
    }

    pub fn get_x(&self) -> f64 {
        self.pos.x
    }

    pub fn get_y(&self) -> f64 {
        self.pos.y
    }
}

impl Entity for &Ugly {
    fn get_pos(&self) -> Vector {
        self.pos
    }

    fn get_speed(&self) -> Vector {
        self.speed
    }
}

impl Entity for Ugly {
    fn get_pos(&self) -> Vector {
        self.pos
    }

    fn get_speed(&self) -> Vector {
        self.speed
    }
}