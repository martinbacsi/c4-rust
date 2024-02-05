#[derive(Debug, Clone )]
pub struct xorshift {
    x: i64,
    y: i64,
}

impl xorshift {
    pub fn new(seed: i64) -> Self {
        xorshift { x: seed | 1, y: seed }
    }

    pub fn next(&mut self) -> i64 {
        self.x ^= self.x << 13;
        self.x ^= self.x >> 17;
        self.x ^= self.x << 5;
        self.y = self.y.wrapping_add(123456789123456789);
        self.x.wrapping_add(self.y)
    }

    pub fn next_in_range(&mut self, range: i32) -> i32 {
        let mut n = self.next() & 0xFFFFFFF ;
        (n as i32) % range
    }
}