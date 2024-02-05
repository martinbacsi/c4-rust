use crate::{vector::* };
pub trait Entity {
    fn get_pos(&self) -> Vector;
    fn get_speed(&self) -> Vector;
}