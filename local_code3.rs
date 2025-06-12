#[derive(Debug)]
struct Student {
    name: String,
    assist: u8,
    mean: f32, 
    exp: u32, // Les falto la coma XD
    level: u16,
    penalty: u8,
    bonus: u8,
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

    fn calcular_exp_base(&mut self) {
        self.exp = (self.assist as f32 * self.mean).trunc() as u32;
    }              // Aqui lo coloque como as f32 porque si no no se pueden multiplicar
                   // No lo quize cambiar desde la estruct Student por si necesitaban
                   // Que estrictamente el atributo assist fuera u8
}


struct LevelSystem {
    base_exp: u32,
    growth_factor: f32,
}

impl LevelSystem {
    fn level_up(&mut self, student:Student) -> u32 {
        let xperi = (student.assist as f32 * student.mean * 0.3).trunc() as u32;   
        let level_uper = student.exp - xperi;
        return level_uper
    }
}

// Voy a crear un "lobby" donde esten todos los Student juntos
struct Lobby {
    estudiantes: Vec<Student>,
}

impl Lobby {
    fn new() -> Self {
        Lobby {
            estudiantes: Vec::new(),
        }
    }

    fn anadir_estu(&mut self, student:Student) {
        self.estudiantes.push(student);
    }
}

fn main() {
    let mut stu_1 = Student::new(String::from("Juan"), 85, 5.6);
    let mut central = Lobby::new();
    println!("El estudiante es {:?}", stu_1); 
    stu_1.calcular_exp_base();
    println!("La experiencia base del estudiante seria {}", stu_1.exp);
    let mut lvlsys_stu_1 = LevelSystem {
        base_exp: stu_1.exp,
        growth_factor: 1.0, // Aun no se como ocupar este dato
    };
    println!("La experiencia que gano luego de hacer anything {}", lvlsys_stu_1.level_up(stu_1));
    // Este seria el codigo como YO creo que quieren hacer esto 
    // Obvio pueden hacerle cambios o borrarlo todo y decirme que esta malo XD (D'X)
}