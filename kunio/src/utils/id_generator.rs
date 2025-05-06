// Not care about overflow
pub struct IdGenerator(u64);

impl IdGenerator {
    pub fn new() -> Self {
        IdGenerator(0)
    }

    pub fn gen_id(&mut self) -> u64 {
        self.0 += 1;
        self.0
    }
}
