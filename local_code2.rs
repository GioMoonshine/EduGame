#[derive(Debug)]
struct Student {
    name: String,
    assist: u8,
    mean: f32,
    exp: u32,
    level: u16,
    penalty: u8,
    bonus: u8,
}

struct LevelSystem {
    exp_table: Vec<u32>,
}

impl LevelSystem {
    fn new(max_level: u32) -> Self {
        let mut exp_table = Vec::with_capacity(max_level as usize);
        let mut total_exp = 0;

        for level in 1..=max_level {
            let exp = (level as f32).powf(3.0).round() as u32;
            total_exp += exp;
            exp_table.push(total_exp);
        }

        Self {exp_table}
    }

    fn level_for_exp(&self, exp: u32) -> u16 {
        for (i, &required_exp) in self.exp_table.iter().enumerate() {
            if exp < required_exp {
                return (i as u16) + 1;
            }
        }
        self.exp_table.len() as u16
    }

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

    fn apply_gammification(&mut self, level_system: &LevelSystem) {
        let base_exp = ((self.mean - 1.0) / 6.0) * 50.0 + (self.assist as f32 / 100.0) * 50.0;
        let mut gained_exp = base_exp.round() as u32;

        self.bonus = 0;
        if self.mean >= 6.0 {
            gained_exp += 20;
            self.bonus += 1;
        }
        if self.assist >= 90 {
            gained_exp += 20;
            self.bonus += 1;
        }

        self.penalty = 0;
        if self.mean < 4.0 {
            gained_exp = gained_exp.saturating_sub(15);
            self.penalty += 1;
        }
        if self.assist < 60 {
            gained_exp = gained_exp.saturating_sub(15);
            self.penalty += 1;
        }

        self.exp += gained_exp;
        self.level = level_system.level_for_exp(self.exp);
    }
}

fn main() {
    let level_system = LevelSystem::new(100);

    let mut stu1 = Student::new(String::from("Luis"), 82, 6.1);

    stu1.apply_gammification(&level_system);

    println!("Estudiante: {}", stu1.name);
    println!("Nivel: {}", stu1.level);
    println!("Exp: {}", stu1.exp);
    println!("Bonificaciones: {}", stu1.bonus);
    println!("Penalizaciones: {}", stu1.penalty);
}