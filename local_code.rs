#[derive(Debug)]
struct Student {
    nombre: String,
    puntaje: i32,
    penalty: u8,
    bonus: u16,
    bonus_puesto: i16,
    asistencia: f32,
    promedio: f32,
    puesto: usize,
}

impl Student {
    fn new(nombre: String, asistencia: f32, promedio: f32, puesto: usize) -> Self {
        Student {
            nombre,
            puntaje: 500,
            penalty: 0,
            bonus: 0,
            bonus_puesto: 0,
            asistencia,
            promedio,
            puesto,
        }
    }

    fn calcular_bonus(&mut self) {
        let multiplicador = if self.asistencia >= 1.0 {
            1.00
        } else if self.asistencia >= 0.90 {
            0.99
        } else {
            0.0
        };

        let puntos = (75.0 * multiplicador).trunc() as u16;
        self.bonus = puntos;
    }

    fn calcular_penalty(&mut self) {
        self.penalty = if self.promedio <= 4.0 {
            3
        } else if self.promedio <= 5.0 {
            2
        } else if self.promedio <= 6.0 {
            1
        } else {
            0
        };
    }

    fn calcular_bonus_puesto(&mut self) {
        if self.puesto <= 5 {
            let puntos = (50.0 * self.asistencia).trunc() as i16;
            self.bonus_puesto = puntos;
        } else {
            self.bonus_puesto = 0;
        }
    }

    fn actualizar_puntaje(&mut self) {
        self.calcular_bonus();
        self.calcular_penalty();
        self.calcular_bonus_puesto();

        let total_bonus = self.bonus as i32 + self.bonus_puesto as i32;
        let penalizacion = match self.penalty {
            1 => 25,
            2 => 50,
            3 => 75,
            _ => 0,
        };

        self.puntaje += total_bonus;
        self.puntaje -= penalizacion;
    }
}

fn main() {
    let mut estudiante1 = Student::new("María".to_string(), 1.0, 6.5, 3);
    let mut estudiante2 = Student::new("Pedro".to_string(), 0.92, 5.8, 7);
    let mut estudiante3 = Student::new("Lucía".to_string(), 0.88, 4.3, 2);

    estudiante1.actualizar_puntaje();
    estudiante2.actualizar_puntaje();
    estudiante3.actualizar_puntaje();

    println!("Datos del estudiante 1: {:?}", estudiante1);
    println!("Datos del estudiante 2: {:?}", estudiante2);
    println!("Datos del estudiante 3: {:?}", estudiante3);
}