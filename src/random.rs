struct random {
    x: usize,
    y: usize,
    z: usize
}

impl random {
    pub fn xorshf96(&mut self) -> usize {
        self.x ^=  &self.x << 16;
        self.x ^= self.x >> 5;
        self.x ^= self.x << 1;
        let mut t= self.x;
        self.x = self.y;
        self.y = self.z;
        self.z = t ^ self.x ^ self.y;
        self.z
    }

    pub fn new() -> random {
        random{
            x:123456789, 
            y:362436069, 
            z:521288629
        }
    }  
}
