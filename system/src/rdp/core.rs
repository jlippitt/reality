pub trait Bus {}

pub struct Core {
    //
}

impl Core {
    pub fn new() -> Self {
        Self {}
    }

    pub fn step(&mut self, bus: &mut impl Bus) {
        //
    }
}
