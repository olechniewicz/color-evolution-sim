/// Xoshiro256** PRNG for deterministic, reproducible simulations
pub struct Xoshiro256 {
    state: [u64; 4],
}

impl Xoshiro256 {
    pub fn new(seed: u64) -> Self {
        let mut state = [seed; 4];
        // SplitMix64 to generate initial state
        for i in 0..4 {
            state[i] = Self::splitmix64(&mut state[i]);
        }
        Self { state }
    }

    fn splitmix64(x: &mut u64) -> u64 {
        *x = x.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = *x;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }

    pub fn next_u64(&mut self) -> u64 {
        let result = (self.state[1].wrapping_mul(5)).rotate_left(7).wrapping_mul(9);
        let t = self.state[1] << 17;
        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];
        self.state[2] ^= t;
        self.state[3] = self.state[3].rotate_left(45);
        result
    }

    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    pub fn next_f32(&mut self) -> f32 {
        ((self.next_u64() >> 40) as f32) * (1.0 / 16777216.0)
    }

    pub fn next_in_range(&mut self, min: usize, max: usize) -> usize {
        if min >= max {
            return min;
        }
        min + (self.next_u64() as usize) % (max - min)
    }
}
