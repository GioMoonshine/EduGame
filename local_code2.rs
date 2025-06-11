#[derive(Debug)]
struct Student {
    name: String,
    assist: u8,
    mean: f32 
    exp: u32,
    level: u16,
    penalty: u8,
    bonus: u8,
}

struct LevelSystem {
    base_exp: u32,
    growth_factor: f32,
}

impl Student {
    fn new(name: String, assist: u8, mean: f32) -> Self {
        Student {
            name,
            assist,
            mean,
            exp: 0,
            level: 1,
            penalty: 0,
            bonus: 0,
        }
    }

    fn define_lvl(&mut self) {

    }

    fn calcular_exp_base(&mut self) {
        self.exp = (self.assist * self.mean).trunc() as u32;
    }
}