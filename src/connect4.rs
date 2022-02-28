
/*
+----------------------------+
| 6 13 20 27 34 41 48 55 62 |
| 5 12 19 26 33 40 47 54 61 |
| 4 11 18 25 32 39 46 53 60 |
| 3 10 17 24 31 38 45 52 59 |
| 2  9 16 23 30 37 44 51 58 |
| 1  8 15 22 29 36 43 50 57 |
| 0  7 14 21 28 35 42 49 56 | 63
+----------------------------+
*/

use std::arch::x86_64::_popcnt64;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

const WIDTH: usize = 9;
const HEIGHT: usize = 7;

const FAB_COL: u64 = 0b1111111;
const FAB_ROW: u64 = (1 << (7 * 0))
    | (1 << (7 * 1))
    | (1 << (7 * 2))
    | (1 << (7 * 3))
    | (1 << (7 * 4))
    | (1 << (7 * 5))
    | (1 << (7 * 6))
    | (1 << (7 * 7))
    | (1 << (7 * 8));

const COLS: [u64; WIDTH] = [
    FAB_COL << (7 * 0),
    FAB_COL << (7 * 1),
    FAB_COL << (7 * 2),
    FAB_COL << (7 * 3),
    FAB_COL << (7 * 4),
    FAB_COL << (7 * 5),
    FAB_COL << (7 * 6),
    FAB_COL << (7 * 7),
    FAB_COL << (7 * 8),
];

const ROWS: [u64; HEIGHT] = [
    FAB_ROW << 0,
    FAB_ROW << 1,
    FAB_ROW << 2,
    FAB_ROW << 3,
    FAB_ROW << 4,
    FAB_ROW << 5,
    FAB_ROW << 6,
];

const D1_MASK: u64 = (COLS[0] | COLS[1] | COLS[2] | COLS[3] | COLS[4] | COLS[5])
    & (ROWS[3] | ROWS[4] | ROWS[5] | ROWS[6]);
const D2_MASK: u64 = (COLS[0] | COLS[1] | COLS[2] | COLS[3] | COLS[4] | COLS[5])
    & (ROWS[0] | ROWS[1] | ROWS[2] | ROWS[3]);
const H_MASK: u64 = COLS[0] | COLS[1] | COLS[2] | COLS[3] | COLS[4] | COLS[5];
const V_MASK: u64 = ROWS[0] | ROWS[1] | ROWS[2] | ROWS[3];

const fn won(bb: u64) -> bool {
    let d1 = bb & (bb >> 6) & (bb >> 12) & (bb >> 18) & D1_MASK;
    let d2 = bb & (bb >> 8) & (bb >> 16) & (bb >> 24) & D2_MASK;
    let h = bb & (bb >> 7) & (bb >> 14) & (bb >> 21) & H_MASK;
    let v = bb & (bb >> 1) & (bb >> 2) & (bb >> 3) & V_MASK;
    v + h + d1 + d2 > 0
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Outcome {
    Win,
    Draw,
    None
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Connect4 {
    pub my_bb: u64,
    pub op_bb: u64,
    height: [u8; WIDTH],
    player: u8,
    pub outcome: Outcome,
    pub lastMove: u8
}

impl Connect4 {
    pub fn hash(&self) -> usize {
        let mut hasher = DefaultHasher::new();
        hasher.write_u64(self.my_bb);
        hasher.write_u64(self.op_bb);
        hasher.finish() as usize
    }

    pub fn OnValidActions<T>(&self, func: &mut T)    
    where T: FnMut(u8) {
        (0..WIDTH).for_each(| i| {if self.height[i] < HEIGHT as u8  {func(i as u8)};});
    }

    pub fn new() -> Self {
        Self {
            my_bb: 0,
            op_bb: 0,
            height: [0; WIDTH],
            player: 0,
            outcome: Outcome::None,
            lastMove: u8::max_value()
        }
    }
    
    fn full (self) -> bool {
        self.turn() == (WIDTH * HEIGHT) as i32      
    }

    pub fn turn(self) -> i32 {
        unsafe {
            _popcnt64((self.my_bb | self.op_bb) as i64)
        }
    }

    pub fn step(&mut self, action: u8) {
        self.my_bb ^= 1 << (self.height[action as usize] as usize + HEIGHT * (action as usize));
        self.height[action as usize] += 1;
        std::mem::swap(&mut self.my_bb, &mut self.op_bb);
        self.player = 1 - self.player;
        self.lastMove = action;
        if won(self.op_bb) {
            self.outcome = Outcome::Win;
        } else if self.full() {
            self.outcome = Outcome::Draw;
        } 
    }

    /*const DIMS: &'static [i64] = &[1, 1, HEIGHT as i64, WIDTH as i64];
    type Features = [[[f32; WIDTH]; HEIGHT]; 1];
    fn features(&self) -> Self::Features {
        let mut s = Self::Features::default();
        for row in 0..HEIGHT {
            for col in 0..WIDTH {
                let index = 1 << (row + HEIGHT * col);
                if self.my_bb & index != 0 {
                    s[0][row][col] = 1.0;
                } else if self.op_bb & index != 0 {
                    s[0][row][col] = -1.0;
                } else {
                    s[0][row][col] = -0.1;
                }
            }
        }
        for col in 0..WIDTH {
            let h = self.height[col] as usize;
            if h < HEIGHT {
                s[0][h][col] = 0.1;
            }
        }
        s
    }*/

    pub fn print(&self) {
        for row in (0..HEIGHT).rev() {
            for col in 0..WIDTH {
                let index = 1 << (row + HEIGHT * col);
                print!(
                    "{} ",
                    if self.my_bb & index != 0 {
                        "O"
                    } else if self.op_bb & index != 0 {
                        "X"
                    } else {
                        "."
                    }
                );
            }
            println!();
        }
        println!("0 1 2 3 4 5 6 7 8");
    }
}

/*#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_wins() {
        let mut game = Connect4::new();
        assert!(!game.step(&Column(0)));
        assert!(!game.step(&Column(1)));
        assert!(!game.step(&Column(0)));
        assert!(!game.step(&Column(1)));
        assert!(!game.step(&Column(0)));
        assert!(!game.step(&Column(1)));
        assert!(game.step(&Column(0)));
        assert!(game.is_over());
        assert_eq!(game.winner(), Some(PlayerId::Red));
        assert_eq!(game.reward(game.player()), -1.0);
        assert_eq!(game.player(), PlayerId::Black);
        assert_eq!(game.reward(PlayerId::Black), -1.0);
        assert_eq!(game.reward(PlayerId::Red), 1.0);
    }

    #[test]
    fn test_second_wins() {
        let mut game = Connect4::new();
        assert!(!game.step(&Column(0)));
        assert!(!game.step(&Column(1)));
        assert!(!game.step(&Column(2)));
        assert!(!game.step(&Column(1)));
        assert!(!game.step(&Column(2)));
        assert!(!game.step(&Column(1)));
        assert!(!game.step(&Column(2)));
        assert!(game.step(&Column(1)));
        assert!(game.is_over());
        assert_eq!(game.winner(), Some(PlayerId::Black));
        assert_eq!(game.reward(game.player()), -1.0);
        assert_eq!(game.player(), PlayerId::Red);
        assert_eq!(game.reward(PlayerId::Black), 1.0);
        assert_eq!(game.reward(PlayerId::Red), -1.0);
    }

    #[test]
    fn test_draw() {
        /*
        +-------------------+
        | r b r b r b r b r |
        | r b r b r b r b b |
        | r b r b r b r b r |
        | b r b r b r b r b |
        | b r b r b r b r r |
        | r b r b r b r b b |
        | r b r b r b r b r |
        +-------------------+
        */

        let mut game = Connect4::new();
        assert!(!game.step(&Column(0)));
        assert!(!game.step(&Column(1)));
        assert!(!game.step(&Column(0)));
        assert!(!game.step(&Column(1)));
        assert!(!game.step(&Column(1)));
        assert!(!game.step(&Column(0)));
        assert!(!game.step(&Column(1)));
        assert!(!game.step(&Column(0)));
        assert!(!game.step(&Column(0)));
        assert!(!game.step(&Column(1)));
        assert!(!game.step(&Column(0)));
        assert!(!game.step(&Column(1)));

        assert!(game.iter_actions().position(|c| c == Column(0)).is_some());
        assert!(!game.step(&Column(0)));
        assert!(game.iter_actions().position(|c| c == Column(0)).is_none());

        assert!(game.iter_actions().position(|c| c == Column(1)).is_some());
        assert!(!game.step(&Column(1)));
        assert!(game.iter_actions().position(|c| c == Column(1)).is_none());

        assert!(!game.step(&Column(2)));
        assert!(!game.step(&Column(3)));
        assert!(!game.step(&Column(2)));
        assert!(!game.step(&Column(3)));
        assert!(!game.step(&Column(3)));
        assert!(!game.step(&Column(2)));
        assert!(!game.step(&Column(3)));
        assert!(!game.step(&Column(2)));
        assert!(!game.step(&Column(2)));
        assert!(!game.step(&Column(3)));
        assert!(!game.step(&Column(2)));
        assert!(!game.step(&Column(3)));

        assert!(game.iter_actions().position(|c| c == Column(2)).is_some());
        assert!(!game.step(&Column(2)));
        assert!(game.iter_actions().position(|c| c == Column(2)).is_none());

        assert!(game.iter_actions().position(|c| c == Column(3)).is_some());
        assert!(!game.step(&Column(3)));
        assert!(game.iter_actions().position(|c| c == Column(3)).is_none());

        assert!(!game.step(&Column(4)));
        assert!(!game.step(&Column(5)));
        assert!(!game.step(&Column(4)));
        assert!(!game.step(&Column(5)));
        assert!(!game.step(&Column(5)));
        assert!(!game.step(&Column(4)));
        assert!(!game.step(&Column(5)));
        assert!(!game.step(&Column(4)));
        assert!(!game.step(&Column(4)));
        assert!(!game.step(&Column(5)));
        assert!(!game.step(&Column(4)));
        assert!(!game.step(&Column(5)));

        assert!(game.iter_actions().position(|c| c == Column(4)).is_some());
        assert!(!game.step(&Column(4)));
        assert!(game.iter_actions().position(|c| c == Column(4)).is_none());

        assert!(game.iter_actions().position(|c| c == Column(5)).is_some());
        assert!(!game.step(&Column(5)));
        assert!(game.iter_actions().position(|c| c == Column(5)).is_none());

        assert!(!game.step(&Column(6)));
        assert!(!game.step(&Column(7)));
        assert!(!game.step(&Column(6)));
        assert!(!game.step(&Column(7)));
        assert!(!game.step(&Column(7)));
        assert!(!game.step(&Column(6)));
        assert!(!game.step(&Column(7)));
        assert!(!game.step(&Column(6)));
        assert!(!game.step(&Column(6)));
        assert!(!game.step(&Column(7)));
        assert!(!game.step(&Column(6)));
        assert!(!game.step(&Column(7)));

        assert!(game.iter_actions().position(|c| c == Column(6)).is_some());
        assert!(!game.step(&Column(6)));
        assert!(game.iter_actions().position(|c| c == Column(6)).is_none());

        assert!(game.iter_actions().position(|c| c == Column(7)).is_some());
        assert!(!game.step(&Column(7)));
        assert!(game.iter_actions().position(|c| c == Column(7)).is_none());

        assert!(!game.step(&Column(8)));
        assert!(!game.step(&Column(8)));
        assert!(!game.step(&Column(8)));
        assert!(!game.step(&Column(8)));
        assert!(!game.step(&Column(8)));
        assert!(!game.step(&Column(8)));
        assert!(game.iter_actions().position(|c| c == Column(8)).is_some());
        assert!(game.step(&Column(8)));
        assert!(game.is_over());
        assert_eq!(game.winner(), None);
        assert_eq!(game.reward(PlayerId::Red), 0.0);
        assert_eq!(game.reward(PlayerId::Black), 0.0);
    }

    #[test]
    fn test_horz_wins() {
        for row in 0..HEIGHT {
            let mut bb =
                (1 << (row + 0)) | (1 << (row + 7)) | (1 << (row + 14)) | (1 << (row + 21));
            for _i in 0..6 {
                assert!(won(bb));
                bb <<= 7;
            }
        }
    }

    #[test]
    fn test_vert_wins() {
        for col in 0..WIDTH {
            let mut bb = (1 << (7 * col + 0))
                | (1 << (7 * col + 1))
                | (1 << (7 * col + 2))
                | (1 << (7 * col + 3));
            for _i in 0..4 {
                assert!(won(bb));
                bb <<= 1;
            }
        }
    }

    #[test]
    fn test_d1_wins() {
        for row in 3..HEIGHT {
            let mut bb = (1 << row) | (1 << (row + 6)) | (1 << (row + 12)) | (1 << (row + 18));
            for _i in 0..6 {
                assert!(won(bb));
                bb <<= 7;
            }
        }
    }

    #[test]
    fn test_d2_wins() {
        for col in 0..6 {
            let mut bb = (1 << (7 * col + 0))
                | (1 << (7 * (col + 1) + 1))
                | (1 << (7 * (col + 2) + 2))
                | (1 << (7 * (col + 3) + 3));
            for _i in 0..4 {
                assert!(won(bb));
                bb <<= 1;
            }
        }
    }
}*/
