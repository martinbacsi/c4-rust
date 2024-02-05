use crate::{ugly::*, entity::*, drone::*, game::* };

pub fn get_collision(drone: &Drone, ugly: &Ugly) -> f64 {
    // Check instant collision
    if ugly.get_pos().in_range(&drone.get_pos(), Game::DRONE_HIT_RANGE as f64 + Game::UGLY_EAT_RANGE as f64) {
        return 0.0;
    }

    // Both units are motionless
    if drone.get_speed().is_zero() && ugly.get_speed().is_zero() {
        return -1.0;
    }

    // Change referential
    let x = ugly.get_pos().x;
    let y = ugly.get_pos().y;
    let ux = drone.get_pos().x;
    let uy = drone.get_pos().y;

    let x2 = x - ux;
    let y2 = y - uy;
    let r2 = Game::UGLY_EAT_RANGE as f64 + Game::DRONE_HIT_RANGE as f64;
    let vx2 = ugly.get_speed().x - drone.get_speed().x;
    let vy2 = ugly.get_speed().y - drone.get_speed().y;

    // Resolving: sqrt((x + t*vx)^2 + (y + t*vy)^2) = radius <=> t^2*(vx^2 + vy^2) + t*2*(x*vx + y*vy) + x^2 + y^2 - radius^2 = 0
    // at^2 + bt + c = 0;
    // a = vx^2 + vy^2
    // b = 2*(x*vx + y*vy)
    // c = x^2 + y^2 - radius^2

    let a = vx2 * vx2 + vy2 * vy2;

    if a <= 0.0 {
        return -1.0;
    }

    let b = 2.0 * (x2 * vx2 + y2 * vy2);
    let c = x2 * x2 + y2 * y2 - r2 * r2;
    let delta = b * b - 4.0 * a * c;

    if delta < 0.0 {
        return -1.0;
    }

    let t = (-b - delta.sqrt()) / (2.0 * a);

    if t <= 0.0 {
        return -1.0;
    }

    if t > 1.0 {
        return -1.0;
    }
    t
}