use std::ops::Div;

use crate::{entity::*, vector::*};
// Assuming you already have the Entity and Vector structs from the previous examples

#[derive(Debug, Clone)]
pub struct Closest<T: Entity> {
    pub list: Vec<T>,
    pub distance: f64,
}

impl<T: Entity> Closest<T> {
    fn new(list: Vec<T>, distance: f64) -> Self {
        Closest { list, distance }
    }

    pub fn get(&self) -> Option<&T> {
        self.list.get(0)
    }

    pub fn has_one(&self) -> bool {
        self.list.len() == 1
    }

    pub fn get_pos(&self) -> Option<Vector> {
        self.list.get(0).map(|entity| entity.get_pos())
    }

    pub fn get_mean_pos(&self) -> Option<Vector> {
        if self.has_one() {
            return self.get_pos();
        }

        let (sum_x, sum_y) = self
            .list
            .iter()
            .fold((0.0, 0.0), |(acc_x, acc_y), entity| {
                let pos = entity.get_pos();
                (acc_x + pos.x, acc_y + pos.y)
            });

        let mean_x = sum_x / self.list.len() as f64;
        let mean_y = sum_y / self.list.len() as f64;

        Some(Vector { x: mean_x, y: mean_y })
    }
}

pub fn get_closest_to<T>(from: Vector, targets: impl Iterator<Item = T>) -> Closest<T>
where
    T: Entity,
{
    let mut closests = Vec::new();
    let mut min_dist = 0.0;

    for t in targets {
        let dist = t.get_pos().sqr_euclidean_to(from);
        if closests.is_empty() || dist < min_dist {
            closests.clear();
            closests.push(t);
            min_dist = dist;
        } else if dist == min_dist {
            closests.push(t);
        }
    }

    Closest::new(closests, min_dist.sqrt())
}