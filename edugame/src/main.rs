use rocket::serde::{Deserialize, Serialize, json::Json};
use rocket::{get, post, routes, launch, form::Form, response::content::RawHtml, State};
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use rocket::tokio::sync::RwLock;
use rocket::serde::json::serde_json;
use rand::Rng;


/* Estrctura para el logeo, si ocupamos muchas estructuras, originalmente tenia pensado en tener un
 * csv con todo guardado, pero nos decantamos por usar estructuras para todo
 * y evitar guardar las contraseñas de los usuarios que ingresen, como contra 
 * el leaderboard y datos de los que jugaron mueren junto con el termino de la ejecion de codigo
 */

#[derive(Debug, Serialize, Deserialize, rocket::form::FromForm)]
#[serde(crate = "rocket::serde")]
struct LoginForm {
	username: String,
	password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse {
	status: i32,
	u: Option<String>,
}


// Estructura principal de usuario

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
struct Student {
	name: String,
	assist: u8,
	grades: u16,
	mean: f32,
	exp: u32,
	level: u16,
	penalty: u8,
	bonus: u8,
	coins: u32,
}
#[derive(Debug)]
struct LevelSystem;

/*impl LevelSystem {
 *    fn level_for_exp(&self, exp: u32) -> u16 {
 *        match exp {
 *            0..=99 => 1,
 *            100..=299 => 2,
 *            300..=599 => 3,
 *            600..=999 => 4,
 *            1000..=1999 => 5,
 *            2000..=2999 => 6,
 *            3000..=3999 => 7,
 *            4000..=4999 => 8,
 *            5000..=5999 => 9,
 *            _ => 10,
 *        }
 *    }
 *}
 */

// Implementación del sistema de niveles, el viejo era un crimen contra la humanidad y empezamos a utlizar u16
impl LevelSystem {
	fn level_for_exp(&self, exp: u32) -> u16 {
		if exp < 100 {
			1
		} else {
			2 + (((exp as f64 / 2.0).sqrt().floor()) as u16 + 1) as u16 // Sistema para subir de nivel en base a la experiencia, mientras mas experiencia mas alto el nivel
		}
	}
}



//Coinflip? Una mausqueherramienta misteriosa que nos ayudará más tarde
 /*
 * Estructura para la seccion de cara o sello, * donde el usuario puede apostar monedas y ganar o perder, pero
 * solo se utiliza para manejo interno de datos, en el formulario de la API /play-coinflip
 */
#[derive(Debug, Serialize, Deserialize, rocket::form::FromForm)]
#[serde(crate = "rocket::serde")]
struct CoinFlipForm {
	username: String, //El username usamos el correo, así para todo
	bet_amount: u32,
	choice: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct CoinFlipResult {
	result: String,
	won: bool,
	coins_won: u32,
	coins_lost: u32,
	new_balance: u32,
}

 /*
 * Estructura para el segundo juego que tiene EduGame, API /play-slots
 */

#[derive(Debug, Serialize, Deserialize, rocket::form::FromForm)]
#[serde(crate = "rocket::serde")]
struct SlotsForm {
	username: String,
	amount: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct SlotsResult {
	won: bool,
	symbols: Vec<String>, 
	new_balance: u32,
	payout: u32,			//Nadie nunca debería tener balance,monto apostado, pago negativo
	amount_wagered: u32,
	win_type: Option<String>, // Dos iguales, Tres iguales

}

 /*
 * Estructura para la tienda de EduGame
 */

#[derive(Debug, Serialize, Deserialize, rocket::form::FromForm)]
#[serde(crate = "rocket::serde")]
struct PurchaseForm {
	username: String,
	item_type: String,
	quantity: u32,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct ShopItem {
	id: String,
	name: String,
	description: String,
	price: u32,
	max_quantity: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct PurchaseResult {
	success: bool,
	message: String,
	coins_spent: u32,
	new_balance: u32,
	item_received: String,
	quantity: u32,
}


 /*
 * Estructura para la tienda de EduGame
 */

type StudentStorage = Arc<RwLock<HashMap<String, Student>>>; 
/* El hashmap que guarda los estudiantes, y se comparte entre todas las peticiones, supuestamente thread-safe https://doc.rust-lang.org/std/sync/struct.Arc.html
 * https://docs.rs/tokio/latest/tokio/sync/struct.Mutex.html#which-kind-of-mutex-should-you-use tokio dice que deberíamos usar mutex pero
 * de mi mala experiencia con threading y flask en pyhthon ya no confío en lo que dice la documentacion
 */


impl Student {

	// Calcular la media de cada curso, entrará como vector y saldra en float, esto se hará para cada una de las materias, algebra, calculo, habilidades y electivo
	fn calculate_course_mean(grades: &Vec<String>) -> f32 {
		let mut total = 0.0;
		let mut count = 0;
		
		for grade in grades {
			if let Ok(grade_val) = grade.parse::<f32>() {
				total += grade_val;
				count += 1;
			}
		}
		
		if count > 0 { 
			return total / count as f32 
		} else { 
			return 0.0
		}
	}
	

	// Aplicar la gamificación del curso, se le pasa el promedio de curso, la asistencia, y se le aplica el sistema de niveles
	fn apply_course_gamification(&mut self, course_mean: f32, course_attendance: f32, level_system: &LevelSystem) {
		let base_exp = ((course_mean - 1.0) / 6.0) * 50.0 + (course_attendance / 100.0) * 50.0; // En base al promedio del curso, a su asistencia, se saca un puntaje, nos inventamos esta formula que calcula la experiencia 
		let mut gained_exp = base_exp.max(0.0).round() as u32; // Experiencia extra que se dará de bonificacion al usuario si, 
		
		if course_mean >= 5.5 { // Tiene promedio mayor igual a 5.5
			gained_exp += 100;
			self.bonus += 1;
		}
		if course_mean >= 6.0 { // Tiene promedio mayor igual a 6.0
			gained_exp += 200;
			self.bonus += 1;
		}
		if course_mean >= 6.5 { // Tiene promedio mayor igual a 6.5
			gained_exp += 350;
			self.bonus += 1;
		}
		
		// Todas se aplican mas de una vez, osea el que tiene promedio 7.0 ganara los primeros 100 de xp, luego otros 200 y otros 350


		if course_attendance >= 80.0 { // Tiene asistencia mayor igual a 80%
			gained_exp += 225;
			self.bonus += 1;
		}
		if course_attendance >= 85.0 { // Tiene asistencia mayor igual a 85%
			
			gained_exp += 100;
			self.bonus += 1;
		}
		if course_attendance >= 90.0 { // Tiene asistencia mayor igual a 90%
			gained_exp += 225;
			self.bonus += 2;
		}
		
		// Penalizaciones, si el promedio es menor a 4.5 o la asistencia es menor a 65% se le resta experiencia y se le aplica una penalizacion

		if course_mean < 4.5 {
			gained_exp = gained_exp.saturating_sub(125);
			self.penalty += 1;
		}
		if course_attendance < 65.0 {
			gained_exp = gained_exp.saturating_sub(125);
			self.penalty += 1;
		}
		
		self.exp += gained_exp; //Se le suma la experiencia
		self.level = level_system.level_for_exp(self.exp); //Se le calcula el nivel

		let coin_bonus = (self.level as u32 * 2) + (gained_exp / 10);
		self.coins += (coin_bonus * self.exp) / 100;
		// Sistema para bonificacion de monedas, mientras mas XP mas monedas
	}
	

	// Sistema de bonificacion aplicado
	fn apply_full_gamification(&mut self, 
		electivo_grades: &Vec<String>, //Por cada nota de cada materia elegida se calculara apply_course_gamification
		electivo_attendance: f32,
		habilidades_grades: &Vec<String>, 
		habilidades_attendance: f32,
		algebralineal_grades: &Vec<String>, 
		algebralineal_attendance: f32,
		calculointegral_grades: &Vec<String>, 
		calculointegral_attendance: f32,
		level_system: &LevelSystem) {
		
		self.exp = 0;
		self.level = 1;
		self.penalty = 0;
		self.bonus = 0;
		self.coins = 100;	//La unica base que se da son las 100 monedas
		
		let electivo_mean = Self::calculate_course_mean(electivo_grades);				//Todo es muy rebundnte pero no se me ocurrio otra manera de hacerlo, aquí se calcula la media de las notas de los vectores que almacenan las notas
		let habilidades_mean = Self::calculate_course_mean(habilidades_grades);
		let algebralineal_mean = Self::calculate_course_mean(algebralineal_grades);
		let calculointegral_mean = Self::calculate_course_mean(calculointegral_grades);
		
		self.apply_course_gamification(electivo_mean, electivo_attendance, level_system);		
		self.apply_course_gamification(habilidades_mean, habilidades_attendance, level_system);
		self.apply_course_gamification(algebralineal_mean, algebralineal_attendance, level_system);
		self.apply_course_gamification(calculointegral_mean, calculointegral_attendance, level_system);
		
		let total_grades = electivo_mean + habilidades_mean + algebralineal_mean + calculointegral_mean; 
		self.mean = total_grades / 4.0;
		

		let mut total_individual_grades = 0.0;
		
		// Se suman todas las notas de cada materia, se multiplica por 10 para que sea un valor mas alto y se pueda usar en el sistema de niveles
		for grades_vec in [electivo_grades, habilidades_grades, algebralineal_grades, calculointegral_grades].iter() {
			for grade in grades_vec.iter() {
				if let Ok(grade_val) = grade.parse::<f32>() {
					total_individual_grades += grade_val;
				}
			}
		}
		
		self.grades = (total_individual_grades * 10.0) as u16;
		// Se multiplica por 10 para que sea un enetro

		self.assist = ((electivo_attendance + habilidades_attendance + algebralineal_attendance + calculointegral_attendance) / 4.0) as u8;
		//Guardamos la media de la asistencia entre los 4 cursos
	}
}

async fn scrape_ucampus(username: String, password: String) -> Result<Student, Box<dyn std::error::Error + Send + Sync>> {

	/*
	 * Armado de el cliente para las peticiones web de ucampus PD: Todo lo saque y probe con postman interceptor, despues con flask en python y ahí lo traduje a oxido
	 * donde su equivalencia es reqwest, tambien usamos tokio y serde para las respuestas en json
 	 */

	let client = Client::builder()
		.cookie_store(true)
		.build()?;

	let cookie_monster = client
		.get("https://ucampus.uahurtado.cl/")
		.send()
		.await?;

	/*
	 * Extraigo las coockies que dan para usar posterior en las otras peticiones que almacenan una sola sesion
 	 */

	let cookies = cookie_monster.cookies().collect::<Vec<_>>();
	let sess_cookie = cookies
		.iter()
		.find(|c| c.name() == "_ucampus")
		.ok_or("Session cookie not found")?; //No deberia pasar NUNCA a no ser que pase un problema

	let mut login_data = HashMap::new();
	login_data.insert("servicio", "ucampus");
	login_data.insert("debug", "0");
	login_data.insert("_sess", sess_cookie.value());
	login_data.insert("_LB", "uah02-int");
	login_data.insert("lang", "es");
	login_data.insert("username", &username);
	login_data.insert("password", &password);
	login_data.insert("recordar", "1");

	let login_response = client
		.post("https://ucampus.uahurtado.cl/auth/api")
		.header("User-Agent", "GamificationEngineRuntime/7.44.1") //No funciona y llega el correo a quien lo usa que ingresaron de un dispositivo unknown
		.form(&login_data)
		.send()
		.await?;

	let login_json: ApiResponse = login_response.json().await?;

	if login_json.status != 200 {
		return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Error. Verifica tu usuario y contraseña.")));
	}

	//Si login_json.u tiene algo entonces se guarda en main_url, la no existencia de valores nulos nos deja con el uso de Some
	// el uso de some lo utilizo para varias veces luego cuando tenga que extraer las notas de las materias
	if let Some(main_url) = login_json.u {
		client.get(&main_url).send().await?;
	}

	/*
	 *  Seccion - Inicio
	 *       Electivo especialidad
	 */

	let mut grades1_response = client
		.get("https://ucampus.uahurtado.cl/uah/2025/1/CSI0168/1/notas/alumno")
		.send()
		.await?;
	let mut grades1_text = grades1_response.text().await?;
	if grades1_text.contains("No tienes permisos para ver esta") { 
		//En caso de que no esten en la seccion 1 prueba a ver en la seccion 2, el formato de la url es CSIXXXX/[seccion]/notas/alumno
		grades1_response = client.get("https://ucampus.uahurtado.cl/uah/2025/1/CSI0168/2/notas/alumno")
		.send()
		.await?;
		grades1_text = grades1_response.text().await?;
	}


	let mut attendance1_response = client
		.get("https://ucampus.uahurtado.cl/uah/2025/1/CSI0168/1/asistencias2/")
		.send()
		.await?;
	let mut attendance1_text = attendance1_response.text().await?;
	if attendance1_text.contains("No tienes permisos para ver esta") {
		attendance1_response = client.get("https://ucampus.uahurtado.cl/uah/2025/1/CSI0168/2/asistencias2/")
		.send()
		.await?;
		attendance1_text = attendance1_response.text().await?;
	}

	let mut base_1 = 1;
	if grades1_text.contains("Examen") {
		base_1 = base_1 + 1;
	}

	/*
	 *  Cuando alguien tiene que dar examen aparece un formulario vacio donde antes estaba la primera nota, esto lo arregla
	 */

	let mut electivo_grades = vec![];

	if let Some(grade) = extract_nth_between(&grades1_text.to_string().replace("wrong", ""), r#"<h1 class="strong"><span class="">"#, "</span></h1>", base_1 as usize) {
		electivo_grades.push(grade);
	}
	if let Some(grade) = extract_nth_between(&grades1_text.to_string().replace("wrong", ""), r#"<h1 class="strong"><span class="">"#, "</span></h1>", base_1 + 1) {
		electivo_grades.push(grade); // En caso de no existir no queda un vector con un valor vacio y queda solo con la longitud de los mismos valores que encontro
	}


	let electivo_attendance = extract_attendance(&attendance1_text, "<th>Asistencia", "%</h1>")
		.unwrap_or("0".to_string()).replace(">", "");

	/*
	 *  Lo que explique arriba ahora 3 veces mas
	 */

	/*
	 *  Seccion - Fin
	 *       Electivo especialidad
	 */

	/*
	 *  Seccion - Inicio
	 *       Habilidades III
	 */
	let mut grades2_response = client
		.get("https://ucampus.uahurtado.cl/uah/2025/1/CSI0169/1/notas/alumno")
		.send()
		.await?;
	let mut grades2_text = grades2_response.text().await?;
	if grades2_text.contains("No tienes permisos para ver esta") {
		grades2_response = client.get("https://ucampus.uahurtado.cl/uah/2025/1/CSI0169/2/notas/alumno")
		.send()
		.await?;
		grades2_text = grades2_response.text().await?;
	}

	let mut attendance2_response = client
		.get("https://ucampus.uahurtado.cl/uah/2025/1/CSI0169/1/asistencias2/")
		.send()
		.await?;
	let mut attendance2_text = attendance2_response.text().await?;
	if attendance2_text.contains("No tienes permisos para ver esta") {
		attendance2_response = client.get("https://ucampus.uahurtado.cl/uah/2025/1/CSI0169/2/asistencias2/")
		.send()
		.await?;
		attendance2_text = attendance2_response.text().await?;
	}
	
	let mut base_2 = 1;
	if grades2_text.contains("Examen") {
		base_2 = base_2 + 1;
	}


	let mut habilidades_grades = vec![];

	if let Some(grade) = extract_nth_between(&grades2_text.to_string().replace("wrong", ""), r#"<h1 class="strong"><span class="">"#, "</span></h1>", base_2 as usize) {
		habilidades_grades.push(grade);
	}
	if let Some(grade) = extract_nth_between(&grades2_text.to_string().replace("wrong", ""), r#"<h1 class="strong"><span class="">"#, "</span></h1>", base_2 + 1) {
		habilidades_grades.push(grade);
	}
	if let Some(grade) = extract_nth_between(&grades2_text.to_string().replace("wrong", ""), r#"<h1 class="strong"><span class="">"#, "</span></h1>", base_2 + 2) {
		habilidades_grades.push(grade);
	}
	if let Some(grade) = extract_nth_between(&grades2_text.to_string().replace("wrong", ""), r#"<h1 class="strong"><span class="">"#, "</span></h1>", base_2 + 3) {
		habilidades_grades.push(grade);
	}
	if let Some(grade) = extract_nth_between(&grades2_text.to_string().replace("wrong", ""), r#"<h1 class="strong"><span class="">"#, "</span></h1>", base_2 + 4) {
		habilidades_grades.push(grade);
	}

	let habilidades_attendance = extract_attendance(&attendance2_text, "<th>Asistencia", "%</h1>")
		.unwrap_or("0".to_string()).replace(">", "");

	/*
	 *  Seccion - Fin
	 *       Habilidades III
	 */

	/*
	 *  Seccion - Inicio
	 *       Algebra lineal
	 */

	let mut grades3_response = client
		.get("https://ucampus.uahurtado.cl/uah/2025/1/CSI0165/1/notas/alumno")
		.send()
		.await?;
	let mut grades3_text = grades3_response.text().await?;
	if grades3_text.contains("No tienes permisos para ver esta") {
		grades3_response = client.get("https://ucampus.uahurtado.cl/uah/2025/1/CSI0165/2/notas/alumno")
		.send()
		.await?;
		grades3_text = grades3_response.text().await?;

	}
	let mut attendance3_response = client
		.get("https://ucampus.uahurtado.cl/uah/2025/1/CSI0165/1/asistencias2/")
		.send()
		.await?;
	let mut attendance3_text = attendance3_response.text().await?;
	if attendance3_text.contains("No tienes permisos para ver esta") {
		attendance3_response = client.get("https://ucampus.uahurtado.cl/uah/2025/1/CSI0165/2/asistencias2/")
		.send()
		.await?;
		attendance3_text = attendance3_response.text().await?;
	}

	let mut base_3 = 1;
	if grades3_text.contains("Examen") {
		base_3 = base_3 + 1;
	}

	let mut algebralineal_grades = vec![];

	if let Some(grade) = extract_nth_between(&grades3_text.to_string().replace("wrong", ""), r#"<h1 class="strong"><span class="">"#, "</span></h1>", base_3) {
		algebralineal_grades.push(grade);
	}
	if let Some(grade) = extract_nth_between(&grades3_text.to_string().replace("wrong", ""), r#"<h1 class="strong"><span class="">"#, "</span></h1>", base_3 + 1) {
		algebralineal_grades.push(grade);
	}
	if let Some(grade) = extract_nth_between(&grades3_text.to_string().replace("wrong", ""), r#"<h1 class="strong"><span class="">"#, "</span></h1>", base_3 + 2) {
		algebralineal_grades.push(grade);
	}


	let algebralineal_attendance = extract_attendance(&attendance3_text, "<th>Asistencia", "%</h1>")
		.unwrap_or("0".to_string()).replace(">", "");

	/*
	 *  Seccion - Fin
	 *       Algebra lineal
	 */

	/*
	 *  Seccion - Inicio
	 *       Calculo integral
	 */


	let mut grades4_response = client
		.get("https://ucampus.uahurtado.cl/uah/2025/1/CSI0167/1/notas/alumno")
		.send()
		.await?;
	let mut grades4_text = grades4_response.text().await?;
	if grades4_text.contains("No tienes permisos para ver esta") {
		grades4_response = client.get("https://ucampus.uahurtado.cl/uah/2025/1/CSI0167/2/notas/alumno")
		.send()
		.await?;
		grades4_text = grades4_response.text().await?;
	}
	let mut attendance4_response = client
		.get("https://ucampus.uahurtado.cl/uah/2025/1/CSI0167/1/asistencias2/")
		.send()
		.await?;
	let mut attendance4_text = attendance4_response.text().await?;
	if attendance4_text.contains("No tienes permisos para ver esta") {
		attendance4_response = client.get("https://ucampus.uahurtado.cl/uah/2025/1/CSI0167/2/asistencias2/")
		.send()
		.await?;
		attendance4_text = attendance4_response.text().await?;
	}

	let mut base_4 = 1;
	if grades4_text.contains("Examen") {
		base_4 = base_4 + 1;
	}

	let mut calculointegral_grades = vec![];

	if let Some(grade) = extract_nth_between(&grades4_text.to_string().replace("wrong", ""), r#"<h1 class="strong"><span class="">"#, "</span></h1>", base_4 as usize) {
		calculointegral_grades.push(grade);
	}
	if let Some(grade) = extract_nth_between(&grades4_text.to_string().replace("wrong", ""), r#"<h1 class="strong"><span class="">"#, "</span></h1>", base_4 + 1) {
		calculointegral_grades.push(grade);
	}


	let acalculointegral_attendance = extract_attendance(&attendance4_text, "<th>Asistencia", "%</h1>")
		.unwrap_or("0".to_string()).replace(">", "");
	/*
	 *  Seccion - Fin
	 *       Calculo integral
	 */

	let name = extract_nth_between(&grades1_text, "alias: '", "',", 1)
		.unwrap_or("Desconocido".to_string()); //De aquí sale el nombre del estudiante


	let mut student = Student {
		name,
		assist: 0,
		grades: 0,
		mean: 0.0,
		exp: 0,
		level: 1,
		penalty: 0,
		bonus: 0,
		coins: 100,
	};

	//Convertimos todas las asistencias en flotantes para el vector, en caso de no poder convertirlo se pone 0.0
	let electivo_att_val = electivo_attendance.parse::<f32>().unwrap_or(0.0); 
	let habilidades_att_val = habilidades_attendance.parse::<f32>().unwrap_or(0.0);
	let algebralineal_att_val = algebralineal_attendance.parse::<f32>().unwrap_or(0.0);
	let calculointegral_att_val = acalculointegral_attendance.parse::<f32>().unwrap_or(0.0);


	//Super debug information
	println!("Elect_val, habil_val {:?},{:?}", electivo_att_val, habilidades_att_val);
	println!(" Username: {:?}\n Grades_Electivo_raw|AsistenciaR: {:?} | {:?}\n Grades_Habilidades_raw|Asistencia: {:?} | {:?}", username,electivo_grades, electivo_attendance, habilidades_grades, habilidades_attendance);
	print!(" Grades_Algebra_lineal_raw|Asistencia: {:?} | {:?}\n Grades_Calculo_integral_raw|Asistencia: {:?} | {:?}\n==============================\n", algebralineal_grades, algebralineal_attendance, calculointegral_grades, acalculointegral_attendance);


	//Aplicacion de el sistema de nivel para cada materia, se le pasa el vector de notas, la asistencia y el sistema de niveles
	let level_system = LevelSystem;
	student.apply_full_gamification(
		&electivo_grades, 
		electivo_att_val,
		&habilidades_grades, 
		habilidades_att_val,
		&algebralineal_grades, 
		algebralineal_att_val,
		&calculointegral_grades, 
		calculointegral_att_val,
		&level_system
	);

	Ok(student)
}

/*
* def extract_between_r( s, first, last ):
*	 start = s.rindex( first ) + len( first )
*	 end = s.rindex( last, start )
*	 return s[start:end]
*/
// Lo que en python hacia con assitencia2 = assaignment2Data.split("<th>Asistencia")[1].split("%</h1>")[0][-3:].replace(">","") aquí tengo que usar esta funcion, nacida a base de find_between_r de https://stackoverflow.com/questions/3368969/find-string-between-two-substrings

fn extract_nth_between(text: &str, start: &str, end: &str, n: usize) -> Option<String> {
	let mut current_pos = 0;
	for i in 1..=n {
		if let Some(start_idx) = text[current_pos..].find(start) {
			let start_pos = start_idx + current_pos + start.len();
			if let Some(end_idx) = text[start_pos..].find(end) {
				let end_pos = end_idx + start_pos;
				if i == n {
					return Some(text[start_pos..end_pos].to_string());
				}
				current_pos = end_pos + end.len();
			} else {
				return None;
			}
		} else {
			return None;
		}
	}
	None
}

// Extrae la asistencia, es como la otra funcion pero devuelve los ultimos 3 caracteres
fn extract_attendance(text: &str, start: &str, end: &str) -> Option<String> {
	let start_pos = text.find(start)? + start.len();
	let end_pos = text[start_pos..].find(end)? + start_pos;
	let full_text = &text[start_pos..end_pos];
	
	if full_text.len() >= 3 {
		Some(full_text[full_text.len()-3..].to_string())
	} else {
		Some(full_text.to_string())
	}
}

#[get("/")]
fn index() -> RawHtml<&'static str> {
	RawHtml(r#"
	<!DOCTYPE html>
	<html>
	<head>
		<title>UCampus EduGame</title>
		<style>
			body {
				font-family: Arial, sans-serif;
				max-width: 600px;
				margin: 50px auto;
				padding: 20px;
				background: #f5f5f5;
			}
			.container {
				background: white;
				padding: 30px;
				border-radius: 10px;
				box-shadow: 0 2px 10px rgba(0,0,0,0.1);
			}
			h1 {
				color: #333;
				text-align: center;
				margin-bottom: 30px;
			}
			.navigation {
				text-align: center;
				margin-bottom: 20px;
			}
			.nav-button {
				display: inline-block;
				margin: 0 10px 10px;
				padding: 10px 20px;
				background: #28a745;
				color: white;
				text-decoration: none;
				border-radius: 5px;
				transition: background 0.3s;
			}
			.nav-button:hover {
				background: #1e7e34;
			}
			.form-group {
				margin-bottom: 20px;
			}
			label {
				display: block;
				margin-bottom: 5px;
				font-weight: bold;
				color: #555;
			}
			input[type="email"], input[type="password"] {
				width: 100%;
				padding: 12px;
				border: 1px solid #ddd;
				border-radius: 5px;
				box-sizing: border-box;
				font-size: 16px;
			}
			button {
				width: 100%;
				padding: 12px;
				background: #007bff;
				color: white;
				border: none;
				border-radius: 5px;
				font-size: 16px;
				cursor: pointer;
				transition: background 0.3s;
			}
			button:hover {
				background: #0056b3;
			}
			button:disabled {
				background: #ccc;
				cursor: not-allowed;
			}
			.result {
				margin-top: 30px;
				padding: 20px;
				background: #f8f9fa;
				border-radius: 5px;
				border-left: 4px solid #007bff;
			}
			.error {
				color: #dc3545;
				background: #f8d7da;
				border-left-color: #dc3545;
			}
			.success {
				color: #155724;
				background: #d4edda;
				border-left-color: #28a745;
			}
			.grade-section {
				margin-bottom: 20px;
			}
			.grade-section h3 {
				color: #007bff;
				margin-bottom: 10px;
			}
			.grade-item {
				margin: 5px 0;
				padding: 5px 10px;
				background: white;
				border-radius: 3px;
				border: 1px solid #ddd;
			}
			.loading {
				text-align: center;
				color: #666;
			}
			.gamification {
				background: #e8f5e8;
				border-left-color: #28a745;
				margin-top: 20px;
			}
			.stats {
				display: grid;
				grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
				gap: 10px;
				margin-top: 15px;
			}
			.stat-item {
				background: white;
				padding: 10px;
				border-radius: 5px;
				text-align: center;
				border: 1px solid #ddd;
			}
		</style>
	</head>
	<body>
		<div class="container">
			<h1>UCampus Gamificacion EduGame</h1>
			
			<div class="navigation">
				<a href="/leaderboard" class="nav-button">🏆 Leaderboard</a>
				<a href="/coinflip" class="nav-button">🪙 Coin Flip Game</a>
				<a href="/shop" class="nav-button">🛒 Tienda</a>
				<a href="/slots" class="nav-button">🎰 Slots</a>
			</div>
			
			<form id="loginForm">
				<div class="form-group">
					<label for="username">Email:</label>
					<input type="email" id="username" name="username" required 
						   placeholder="example@alumnos.uahurtado.cl">
				</div>
				<div class="form-group">
					<label for="password">Contraseña:</label>
					<input type="password" id="password" name="password" required>
				</div>
				<button type="submit" id="submitBtn">Obtener puntaje</button>
			</form>
			<div id="result"></div>
		</div>

		<script>
			document.getElementById('loginForm').addEventListener('submit', async (e) => {
				e.preventDefault();
				
				const submitBtn = document.getElementById('submitBtn');
				const resultDiv = document.getElementById('result');
				
				submitBtn.disabled = true;
				submitBtn.textContent = 'Cargando...';
				resultDiv.innerHTML = '<div class="result loading">Sacando datos, espera...</div>';
				
				const formData = new FormData(e.target);
				const urlEncodedData = new URLSearchParams(formData);
				
				try {
					const response = await fetch('/scrape', {
						method: 'POST',
						headers: {
							'Content-Type': 'application/x-www-form-urlencoded',
						},
						body: urlEncodedData
					});
					
					if (response.ok) {
						const data = await response.json();
						displayResults(data);
					} else {
						const error = await response.text();
						resultDiv.innerHTML = `<div class="result error">Error: ${error}</div>`;
					}
				} catch (error) {
					resultDiv.innerHTML = `<div class="result error">Error de la red: ${error.message}</div>`;
				} finally {
					submitBtn.disabled = false;
					submitBtn.textContent = 'Obtener puntaje';
				}
			});
			
			function displayResults(data) {
				const resultDiv = document.getElementById('result');
				let statusMessage = data.is_new_user ? 
					'<div class="result success">¡Bienvenido! Tu cuenta ha sido registrada en el sistema.</div>' :
					'<div class="result">Datos actualizados en el sistema.</div>';
				
				resultDiv.innerHTML = statusMessage + `
					<div class="result">
						<h2>Estudiante: ${data.name}</h2>
						
						<div class="grade-section">
							<h3>Rendimiento academico</h3>
							<div class="grade-item">Puntos totales por notas: ${data.grades}</div>
							<div class="grade-item">Notas promedio [Electivo y habilidades]: ${data.mean.toFixed(2)}</div>
							<div class="grade-item">Asistencia: ${data.assist}%</div>
						</div>
						
						<div class="result gamification">
							<h3>🎮 Estadisticas de gamificacion</h3>
							<div class="stats">
								<div class="stat-item">
									<strong>Nivel</strong><br>
									${data.level}
								</div>
								<div class="stat-item">
									<strong>Experiencia</strong><br>
									${data.exp} XP
								</div>
								<div class="stat-item">
									<strong>Monedas</strong><br>
									${data.coins} 🪙
								</div>
								<div class="stat-item">
									<strong>Bonus</strong><br>
									${data.bonus}
								</div>
								<div class="stat-item">
									<strong>Castigos</strong><br>
									${data.penalty}
								</div>
							</div>
						</div>
					</div>
				`;
			}
		</script>
	</body>
	</html>
	"#)
}


#[get("/leaderboard")]
async fn leaderboard(storage: &State<StudentStorage>) -> RawHtml<String> {
	let students = storage.read().await; // Lectura al almacenamiento de estudiantes supuestamente threadsafe

	// Clona los datos de los estudiantes en un vector para poder ordenarlos
	let mut sorted_students: Vec<_> = students.values().cloned().collect();


 	// Ordena los estudiantes: primero por experiencia, luego por nivel, luego por monedas
	sorted_students.sort_by(|a, b| {
		b.exp.cmp(&a.exp)
			.then_with(|| b.level.cmp(&a.level))
			.then_with(|| b.coins.cmp(&a.coins))
	});


	//Generar el html de la tabla sorted_students, despues el string se inyecta en RawHtml cuando hay ya estudiantes participando, de lo contrario div de estudiantes registrados = 0 será mostrado en pantalla
	let leaderboard_html = sorted_students
		.iter()
		.enumerate()
		.map(|(i, student)| {
			let rank_emoji = match i { //Asigna un emoji dependiendo del puesto en el leaderboard
				0 => "🥇",
				1 => "🥈", 
				2 => "🥉",
				_ => "🏅",
			};
			format!(
				r#"
				<div class="leaderboard-item rank-{}">
					<span class="rank">{} #{}</span>
					<span class="name">{}</span>
					<div class="stats">
						<span>Nivel: {}</span>
						<span>EXP: {}</span>
						<span>Monedas: {}</span>
						<span class="average-toggle" onclick="toggleAverage(this)" data-value="{:.2}">Copuchentear</span>
					</div>
				</div>
				"#,
				if i < 3 { "top" } else { "normal" }, // Cambia el estilo dependiendo del puesto
				rank_emoji,
				i + 1,
				student.name,
				student.level,
				student.exp,
				student.coins,
				student.mean
			)
		})
		.collect::<Vec<_>>()
		.join("");  //https://www.reddit.com/r/rust/comments/6q4uqc/help_whats_the_best_way_to_join_an_iterator_of/

	RawHtml(format!(r#"
	<!DOCTYPE html>
	<html>
	<head>
		<title>Leaderboard - UCampus EduGame</title>
		<style>
			body {{
				font-family: Arial, sans-serif;
				max-width: 800px;
				margin: 50px auto;
				padding: 20px;
				background: #f5f5f5;
			}}
			.container {{
				background: white;
				padding: 30px;
				border-radius: 10px;
				box-shadow: 0 2px 10px rgba(0,0,0,0.1);
			}}
			h1 {{
				color: #333;
				text-align: center;
				margin-bottom: 30px;
			}}
			.navigation {{
				text-align: center;
				margin-bottom: 30px;
			}}
			.nav-button {{
				display: inline-block;
				margin: 0 10px 10px;
				padding: 10px 20px;
				background: #007bff;
				color: white;
				text-decoration: none;
				border-radius: 5px;
				transition: background 0.3s;
			}}
			.nav-button:hover {{
				background: #0056b3;
			}}
			.leaderboard-item {{
				display: flex;
				justify-content: space-between;
				align-items: center;
				padding: 15px;
				margin: 10px 0;
				border-radius: 8px;
				border-left: 4px solid #ddd;
			}}
			.leaderboard-item.rank-top {{
				background: linear-gradient(135deg, #fff3cd, #ffeaa7);
				border-left-color: #f39c12;
				box-shadow: 0 2px 8px rgba(243, 156, 18, 0.3);
			}}
			.leaderboard-item.rank-normal {{
				background: #f8f9fa;
				border-left-color: #6c757d;
			}}
			.rank {{
				font-size: 18px;
				font-weight: bold;
				min-width: 80px;
			}}
			.name {{
				font-weight: bold;
				flex-grow: 1;
				margin: 0 20px;
				color: #333;
			}}
			.stats {{
				display: flex;
				gap: 15px;
				font-size: 14px;
				color: #666;
			}}
			.stats span {{
				background: white;
				padding: 5px 10px;
				border-radius: 15px;
				border: 1px solid #ddd;
			}}
			.average-toggle {{
				cursor: pointer;
				transition: all 0.3s ease;
				user-select: none;
				background: #ffeaa7 !important;
				color: #d63031;
				font-weight: bold;
				border: 1px solid #fdcb6e !important;
			}}
			.average-toggle:hover {{
				background: #fdcb6e !important;
				transform: scale(1.05);
			}}
			.average-toggle.revealed {{
				background: white !important;
				color: #666;
				font-weight: normal;
				border: 1px solid #ddd !important;
			}}
			.average-toggle.revealed:hover {{
				background: #e9ecef !important;
				transform: none;
			}}
			.empty-state {{
				text-align: center;
				color: #666;
				padding: 40px;
				font-style: italic;
			}}
		</style>
		<script>
			function toggleAverage(element) {{
				if (element.classList.contains('revealed')) {{
					// If already revealed, hide it again
					element.textContent = 'Copuchentear';
					element.classList.remove('revealed');
				}} else {{
					// Reveal the actual grade
					const actualValue = element.getAttribute('data-value');
					element.textContent = 'Promedio: ' + actualValue;
					element.classList.add('revealed');
				}}
			}}
		</script>
	</head>
	<body>
		<div class="container">
			<h1>🏆 Leaderboard - Top Estudiantes</h1>
			
			<div class="navigation">
				<a href="/" class="nav-button">🏠 Inicio</a>
				<a href="/coinflip" class="nav-button">🪙 Coin Flip Game</a>
				<a href="/shop" class="nav-button">🛒 Tienda</a>
				<a href="/slots" class="nav-button">🎰 Slots</a>
			</div>
			
			<div class="leaderboard">
				{}
			</div>
			
			{}
		</div>
	</body>
	</html>
	"#, 
	if leaderboard_html.is_empty() {
		r#"<div class="empty-state">No hay estudiantes registrados aún. ¡Sé el primero en unirte!</div>"#.to_string()
	} else {
		leaderboard_html
	},
	if sorted_students.is_empty() {
		"".to_string()
	} else {
		format!(r#"<p style="text-align: center; margin-top: 20px; color: #666;"><small>Total de estudiantes registrados: {}</small></p>"#, sorted_students.len())
	}))
}

#[get("/coinflip")]
fn coinflip_page() -> RawHtml<&'static str> {
	RawHtml(r#"
	<!DOCTYPE html>
	<html>
	<head>
		<title>Coin Flip - UCampus Gamification</title>
		<style>
			body {
				font-family: Arial, sans-serif;
				max-width: 600px;
				margin: 50px auto;
				padding: 20px;
				background: #f5f5f5;
			}
			.container {
				background: white;
				padding: 30px;
				border-radius: 10px;
				box-shadow: 0 2px 10px rgba(0,0,0,0.1);
			}
			h1 {
				color: #333;
				text-align: center;
				margin-bottom: 30px;
			}
			.navigation {
				text-align: center;
				margin-bottom: 30px;
			}
			.nav-button {
				display: inline-block;
				margin: 0 10px 10px;
				padding: 10px 20px;
				background: #007bff;
				color: white;
				text-decoration: none;
				border-radius: 5px;
				transition: background 0.3s;
			}
			.nav-button:hover {
				background: #0056b3;
			}
			.form-group {
				margin-bottom: 20px;
			}
			label {
				display: block;
				margin-bottom: 5px;
				font-weight: bold;
				color: #555;
			}
			input[type="email"], input[type="number"], select {
				width: 100%;
				padding: 12px;
				border: 1px solid #ddd;
				border-radius: 5px;
				box-sizing: border-box;
				font-size: 16px;
			}
			.choice-buttons {
				display: flex;
				gap: 20px;
				margin: 20px 0;
			}
			.choice-btn {
				flex: 1;
				padding: 20px;
				font-size: 18px;
				border: 2px solid #ddd;
				background: white;
				border-radius: 10px;
				cursor: pointer;
				transition: all 0.3s;
			}
			.choice-btn:hover {
				background: #f0f0f0;
			}
			.choice-btn.selected {
				background: #007bff;
				color: white;
				border-color: #007bff;
			}
			button {
				width: 100%;
				padding: 15px;
				background: #28a745;
				color: white;
				border: none;
				border-radius: 5px;
				font-size: 18px;
				cursor: pointer;
				transition: background 0.3s;
			}
			button:hover {
				background: #1e7e34;
			}
			button:disabled {
				background: #ccc;
				cursor: not-allowed;
			}
			.result {
				margin-top: 30px;
				padding: 20px;
				background: #f8f9fa;
				border-radius: 5px;
				border-left: 4px solid #007bff;
				text-align: center;
			}
			.win {
				background: #d4edda;
				border-left-color: #28a745;
				color: #155724;
			}
			.lose {
				background: #f8d7da;
				border-left-color: #dc3545;
				color: #721c24;
			}
			.error {
				background: #f8d7da;
				border-left-color: #dc3545;
				color: #721c24;
			}
			.coin {
				font-size: 100px;
				margin: 20px 0;
				animation: flip 1s ease-in-out;
			}
			@keyframes flip {
				0% { transform: rotateY(0deg); }
				50% { transform: rotateY(180deg); }
				100% { transform: rotateY(360deg); }
			}
			.balance {
				background: #e3f2fd;
				border: 1px solid #2196f3;
				padding: 15px;
				border-radius: 5px;
				margin-bottom: 20px;
				text-align: center;
				font-size: 18px;
				font-weight: bold;
			}
		</style>
	</head>
	<body>
		<div class="container">
			<h1>🪙 Coin Flip Game</h1>
			
			<div class="navigation">
				<a href="/" class="nav-button">🏠 Inicio</a>
				<a href="/leaderboard" class="nav-button">🏆 Leaderboard</a>
				<a href="/shop" class="nav-button">🛒 Tienda</a>
				<a href="/slots" class="nav-button">🎰 Slots</a>
			</div>
			
			<div id="balanceDiv" class="balance" style="display: none;">
				Monedas disponibles: <span id="currentBalance">0</span> 🪙
			</div>
			
			<form id="coinFlipForm">
				<div class="form-group">
					<label for="username">Email:</label>
					<input type="email" id="username" name="username" required 
						   placeholder="example@alumnos.uahurtado.cl">
				</div>
				<div class="form-group">
					<label for="bet_amount">Cantidad a apostar:</label>
					<input type="number" id="bet_amount" name="bet_amount" min="1" max="1000" required 
						   placeholder="Ej: 10">
				</div>
				<div class="form-group">
					<label>Elige tu apuesta:</label>
					<div class="choice-buttons">
						<div class="choice-btn" data-choice="heads">
							<div style="font-size: 30px;">🪙</div>
							<div>Cara</div>
						</div>
						<div class="choice-btn" data-choice="tails">
							<div style="font-size: 30px;">⚫</div>
							<div>Sello</div>
						</div>
					</div>
				</div>
				<button type="submit" id="flipBtn">🎲 Lanzar Moneda</button>
			</form>
			<div id="result"></div>
		</div>

		<script>
			let selectedChoice = '';
			
			document.querySelectorAll('.choice-btn').forEach(btn => {
				btn.addEventListener('click', () => {
					document.querySelectorAll('.choice-btn').forEach(b => b.classList.remove('selected'));
					btn.classList.add('selected');
					selectedChoice = btn.dataset.choice;
				});
			});

			document.getElementById('username').addEventListener('blur', async (e) => {
				const username = e.target.value;
				if (username) {
					try {
						const response = await fetch(`/balance/${encodeURIComponent(username)}`);
						if (response.ok) {
							const data = await response.json();
							document.getElementById('currentBalance').textContent = data.coins;
							document.getElementById('balanceDiv').style.display = 'block';
							document.getElementById('bet_amount').max = data.coins;
						} else {
							document.getElementById('balanceDiv').style.display = 'none';
						}
					} catch (error) {
						console.log('Usuario no encontrado en el sistema');
						document.getElementById('balanceDiv').style.display = 'none';
					}
				}
			});
			
			document.getElementById('coinFlipForm').addEventListener('submit', async (e) => {
				e.preventDefault();
				
				if (!selectedChoice) {
					alert('Por favor, elige cara o sello');
					return;
				}
				
				const flipBtn = document.getElementById('flipBtn');
				const resultDiv = document.getElementById('result');
				
				flipBtn.disabled = true;
				flipBtn.textContent = 'Lanzando...';
				
				const formData = new FormData(e.target);
				formData.append('choice', selectedChoice);
				const urlEncodedData = new URLSearchParams(formData);
				
				try {
					const response = await fetch('/play-coinflip', {
						method: 'POST',
						headers: {
							'Content-Type': 'application/x-www-form-urlencoded',
						},
						body: urlEncodedData
					});
					
					if (response.ok) {
						const data = await response.json();
						displayFlipResult(data);
						document.getElementById('currentBalance').textContent = data.new_balance;
						document.getElementById('bet_amount').max = data.new_balance;
					} else {
						const error = await response.text();
						resultDiv.innerHTML = `<div class="result error">Error: ${error}</div>`;
					}
				} catch (error) {
					resultDiv.innerHTML = `<div class="result error">Error de la red: ${error.message}</div>`;
				} finally {
					flipBtn.disabled = false;
					flipBtn.textContent = '🎲 Lanzar Moneda';
				}
			});
			
			function displayFlipResult(data) {
				const resultDiv = document.getElementById('result');
				const coinEmoji = data.result === 'heads' ? '🪙' : '⚫';
				const resultClass = data.won ? 'win' : 'lose';
				const message = data.won ? 
					`¡Ganaste! +${data.coins_won} monedas` : 
					`Perdiste -${data.coins_lost} monedas`;
				
				resultDiv.innerHTML = `
					<div class="result ${resultClass}">
						<div class="coin">${coinEmoji}</div>
						<h3>Resultado: ${data.result === 'heads' ? 'Cara' : 'Sello'}</h3>
						<p>${message}</p>
						<p><strong>Monedas restantes: ${data.new_balance} 🪙</strong></p>
					</div>
				`;
			}
		</script>
	</body>
	</html>
	"#)
}

//API - Aquí se devuelve el json
#[get("/balance/<username>")]
async fn get_balance(username: String, storage: &State<StudentStorage>) -> Result<Json<Student>, rocket::response::status::NotFound<String>> {
	let students = storage.read().await;
	match students.get(&username) {
		Some(student) => Ok(Json(student.clone())),
		None => Err(rocket::response::status::NotFound("Student not found".to_string()))
	}
}

//API - Verificacion de usuario, posteriormente se jugará coinflip
#[post("/play-coinflip", data = "<form>")]
async fn play_coinflip(form: Form<CoinFlipForm>, storage: &State<StudentStorage>) -> Result<Json<CoinFlipResult>, rocket::response::status::Custom<String>> {
	let mut students = storage.write().await;
	//lo mismo de antes, se obtiene el estudiante por su username

	let student = students.get_mut(&form.username)
		.ok_or_else(|| rocket::response::status::Custom(
			rocket::http::Status::NotFound,
			"Estudiante no encontrado. Ingresa a la palaforma primero.".to_string()
		))?;

	//Apuesta solo lo que tienes, no se permite apostar con deuda!
	if student.coins < form.bet_amount {
		return Err(rocket::response::status::Custom(
			rocket::http::Status::BadRequest,
			"Monedas insuficientes.".to_string()
		));
	}

	// Simula el lanzamiento de la moneda con 50% de posibilidad
	let flip_result = if rand::thread_rng().gen_bool(0.5) { "heads" } else { "tails" };
	

	// Determina si el jugador gana
	let won = flip_result == form.choice;

	//Si gana se suma a sus monedas, si pierde cuanto pierde, y cuanto le queda de monedas
	let (coins_won, coins_lost, new_exp) = if won {
		let won_amount = form.bet_amount * 2;
		(won_amount, 0, student.coins + won_amount)
	} else {
		(0, form.bet_amount, student.coins - form.bet_amount)
	};

	student.coins = new_exp;
	//Actualizacion del estudiante

	Ok(Json(CoinFlipResult {
		result: flip_result.to_string(),
		won,
		coins_won,
		coins_lost,
		new_balance: student.coins,
	}))
}

#[get("/shop")]
fn shop_page() -> RawHtml<&'static str> {
	RawHtml(r#"
	<!DOCTYPE html>
	<html>
	<head>
		<title>Tienda - UCampus Gamification</title>
		<style>
			body {
				font-family: Arial, sans-serif;
				max-width: 800px;
				margin: 50px auto;
				padding: 20px;
				background: #f5f5f5;
			}
			.container {
				background: white;
				padding: 30px;
				border-radius: 10px;
				box-shadow: 0 2px 10px rgba(0,0,0,0.1);
			}
			h1 {
				color: #333;
				text-align: center;
				margin-bottom: 30px;
			}
			.navigation {
				text-align: center;
				margin-bottom: 30px;
			}
			.nav-button {
				display: inline-block;
				margin: 0 10px 10px;
				padding: 10px 20px;
				background: #007bff;
				color: white;
				text-decoration: none;
				border-radius: 5px;
				transition: background 0.3s;
			}
			.nav-button:hover {
				background: #0056b3;
			}
			.balance {
				background: #e3f2fd;
				border: 1px solid #2196f3;
				padding: 15px;
				border-radius: 5px;
				margin-bottom: 20px;
				text-align: center;
				font-size: 18px;
				font-weight: bold;
			}
			.shop-items {
				display: grid;
				grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
				gap: 20px;
				margin-bottom: 30px;
			}
			.shop-item {
				border: 2px solid #ddd;
				border-radius: 10px;
				padding: 20px;
				background: white;
				transition: all 0.3s;
			}
			.shop-item:hover {
				border-color: #007bff;
				box-shadow: 0 4px 12px rgba(0,123,255,0.2);
			}
			.item-header {
				text-align: center;
				margin-bottom: 15px;
			}
			.item-icon {
				font-size: 40px;
				margin-bottom: 10px;
			}
			.item-name {
				font-size: 20px;
				font-weight: bold;
				color: #333;
				margin-bottom: 5px;
			}
			.item-price {
				font-size: 18px;
				color: #28a745;
				font-weight: bold;
			}
			.item-description {
				color: #666;
				margin: 15px 0;
				text-align: center;
			}
			.purchase-form {
				display: flex;
				gap: 10px;
				align-items: center;
				justify-content: center;
			}
			.quantity-input {
				width: 60px;
				padding: 8px;
				border: 1px solid #ddd;
				border-radius: 5px;
				text-align: center;
			}
			.buy-btn {
				padding: 10px 20px;
				background: #28a745;
				color: white;
				border: none;
				border-radius: 5px;
				cursor: pointer;
				transition: background 0.3s;
			}
			.buy-btn:hover {
				background: #1e7e34;
			}
			.buy-btn:disabled {
				background: #ccc;
				cursor: not-allowed;
			}
			.form-group {
				margin-bottom: 20px;
			}
			label {
				display: block;
				margin-bottom: 5px;
				font-weight: bold;
				color: #555;
			}
			input[type="email"] {
				width: 100%;
				padding: 12px;
				border: 1px solid #ddd;
				border-radius: 5px;
				box-sizing: border-box;
				font-size: 16px;
			}
			.result {
				margin-top: 20px;
				padding: 15px;
				border-radius: 5px;
				border-left: 4px solid #007bff;
			}
			.success {
				background: #d4edda;
				border-left-color: #28a745;
				color: #155724;
			}
			.error {
				background: #f8d7da;
				border-left-color: #dc3545;
				color: #721c24;
			}
		</style>
	</head>
	<body>
		<div class="container">
			<h1>🛒 UAHShop</h1>
			
			<div class="navigation">
				<a href="/" class="nav-button">🏠 Inicio</a>
				<a href="/leaderboard" class="nav-button">🏆 Leaderboard</a>
				<a href="/coinflip" class="nav-button">🪙 Coin Flip</a>
				<a href="/slots" class="nav-button">🎰 Slots</a>
			</div>
			
			<div class="form-group">
				<label for="username">Email:</label>
				<input type="email" id="username" required 
					   placeholder="example@alumnos.uahurtado.cl">
			</div>
			
			<div id="balanceDiv" class="balance" style="display: none;">
				Monedas disponibles: <span id="currentBalance">0</span> 🪙
			</div>
			
			<div class="shop-items">
				<div class="shop-item">
					<div class="item-header">
						<div class="item-icon">📊</div>
						<div class="item-name">Décima Extra</div>
						<div class="item-price">250 🪙 cada una</div>
					</div>
					<div class="item-description">
						Agrega 0.1 puntos a tu promedio general. ¡Mejora tus notas!
					</div>
					<div class="purchase-form">
						<input type="number" class="quantity-input" min="1" max="10" value="1" 
							   id="decimal-quantity">
						<button class="buy-btn" onclick="purchaseItem('decimal')">
							Comprar
						</button>
					</div>
				</div>
				
				<div class="shop-item">
					<div class="item-header">
						<div class="item-icon">⚡</div>
						<div class="item-name">Experiencia Boost</div>
						<div class="item-price">150 🪙 por 100 XP</div>
					</div>
					<div class="item-description">
						Gana experiencia instantánea para subir de nivel más rápido.
					</div>
					<div class="purchase-form">
						<input type="number" class="quantity-input" min="1" max="20" value="1" 
							   id="experience-quantity">
						<button class="buy-btn" onclick="purchaseItem('experience')">
							Comprar
						</button>
					</div>
				</div>
			</div>
			
			<div id="result"></div>
		</div>

		<script>
			document.getElementById('username').addEventListener('blur', async (e) => {
				const username = e.target.value;
				if (username) {
					try {
						const response = await fetch(`/balance/${encodeURIComponent(username)}`);
						if (response.ok) {
							const data = await response.json();
							document.getElementById('currentBalance').textContent = data.coins;
							document.getElementById('balanceDiv').style.display = 'block';
						} else {
							document.getElementById('balanceDiv').style.display = 'none';
						}
					} catch (error) {
						console.log('Usuario no encontrado en el sistema');
						document.getElementById('balanceDiv').style.display = 'none';
					}
				}
			});

			async function purchaseItem(itemType) {
				const username = document.getElementById('username').value;
				if (!username) {
					alert('Por favor, ingresa tu email primero');
					return;
				}

				const quantity = document.getElementById(`${itemType}-quantity`).value;
				const resultDiv = document.getElementById('result');

				try {
					const response = await fetch('/purchase', {
						method: 'POST',
						headers: {
							'Content-Type': 'application/x-www-form-urlencoded',
						},
						body: new URLSearchParams({
							username: username,
							item_type: itemType,
							quantity: quantity
						})
					});

					if (response.ok) {
						const data = await response.json();
						if (data.success) {
							resultDiv.innerHTML = `
								<div class="result success">
									<h3>¡Compra exitosa!</h3>
									<p>${data.message}</p>
									<p>Monedas gastadas: ${data.coins_spent} 🪙</p>
									<p>Monedas restantes: ${data.new_balance} 🪙</p>
								</div>
							`;
							document.getElementById('currentBalance').textContent = data.new_balance;
						} else {
							resultDiv.innerHTML = `<div class="result error">${data.message}</div>`;
						}
					} else {
						const error = await response.text();
						resultDiv.innerHTML = `<div class="result error">Error: ${error}</div>`;
					}
				} catch (error) {
					resultDiv.innerHTML = `<div class="result error">Error de red: ${error.message}</div>`;
				}
			}
		</script>
	</body>
	</html>
	"#)
}


// Estructura para el formulario de compra
#[post("/purchase", data = "<form>")]
async fn purchase_item(form: Form<PurchaseForm>, storage: &State<StudentStorage>) -> Result<Json<PurchaseResult>, rocket::response::status::Custom<String>> {
	let mut students = storage.write().await;

	// Verifica si el estudiante existe
	let student = students.get_mut(&form.username)
		.ok_or_else(|| rocket::response::status::Custom( //https://api.rocket.rs/master/rocket/response/status/struct.Custom interesante, se puede responder cualquier status
			rocket::http::Status::NotFound,
			"Estudiante no encontrado. Ingresa a la plataforma primero.".to_string()
		))?;

	// Verifica si el tipo de item es válido y obtiene el precio, cantidad maxima
	let (price_per_unit, max_quantity, item_name) = match form.item_type.as_str() {
		"decimal" => (250 as u32, 10 as u32, "Décimas"), //para las decimas solo dejamos 10 de una sola compra por que si no serían muchos puntos, pero si se meten a la tienda nuevamente les deja comprar otra vez
		"experience" => (150 as u32, 20 as u32, "Experiencia (100 XP)"),
		_ => return Err(rocket::response::status::Custom(
			rocket::http::Status::BadRequest,
			"Tipo de item inválido.".to_string()
		))
	};

	//Limite de la compra
	if form.quantity > max_quantity {
		return Err(rocket::response::status::Custom(
			rocket::http::Status::BadRequest,
			format!("Cantidad máxima permitida: {}", max_quantity)
		));
	}

	let total_cost = price_per_unit * form.quantity;

	//Limite de la compra en caso de no existir saldo para lo elejido
	if student.coins < total_cost {
		return Ok(Json(PurchaseResult {
			success: false,
			message: format!("Monedas insuficientes. Necesitas {} monedas.", total_cost),
			coins_spent: 0,
			new_balance: student.coins,
			item_received: "".to_string(),
			quantity: 0,
		}));
	}

	//Substraccion de los costos y dumpeo de datos a student
	student.coins -= total_cost;
	
	match form.item_type.as_str() {
		"decimal" => {
			let decimal_boost = form.quantity as f32 * 0.1;
			student.mean += decimal_boost;
			student.grades += (decimal_boost * 10.0) as u16;
		},
		"experience" => {
			let exp_boost = form.quantity * 100;
			student.exp += exp_boost;
			let level_system = LevelSystem;
			student.level = level_system.level_for_exp(student.exp); //Actualizacion dinamica del nivel en base a los puntos
		},
		_ => {}
	}

	Ok(Json(PurchaseResult {
		success: true,
		message: format!("Has comprado {} {} exitosamente!", form.quantity, item_name),
		coins_spent: total_cost,
		new_balance: student.coins,
		item_received: item_name.to_string(),
		quantity: form.quantity,
	}))
}

//API - Obtencion de items de la tienda
#[get("/shop/items")]
fn get_shop_items() -> Json<Vec<ShopItem>> {
	let items = vec![
		ShopItem {
			id: "decimal".to_string(),
			name: "Décima Extra".to_string(),
			description: "Agrega 0.1 puntos a tu promedio general".to_string(),
			price: 50,
			max_quantity: 10,
		},
		ShopItem {
			id: "experience".to_string(),
			name: "Experiencia Boost".to_string(),
			description: "Gana 100 XP instantáneos".to_string(),
			price: 30,
			max_quantity: 20,
		},
	];
	Json(items)
}

//API - Slots, aquí se elije si gano o perdio en la maquina tragamonedas
#[post("/play-slots", data = "<form>")]
async fn play_slots(form: Form<SlotsForm>, storage: &State<StudentStorage>) -> Result<Json<SlotsResult>, rocket::response::status::Custom<String>> {
	let mut students = storage.write().await;
	
	let student = students.get_mut(&form.username)
		.ok_or_else(|| rocket::response::status::Custom(
			rocket::http::Status::NotFound, //Otra vez manejo de errores
			"Estudiante no encontrado. Ingresa a la plataforma primero.".to_string()
		))?;

	if student.coins < form.amount {
		return Err(rocket::response::status::Custom(
			rocket::http::Status::BadRequest,
			"Monedas insuficientes.".to_string()
		));
	}

	// Lista donde le doy nombre a los simbolos
	let symbols = vec![
		"IHatePyhisics", "IDontLikeAlgebra", "ILikeCounterStrike", "IHateVisualStudio", 
		"ILikeSublimeText", "ILikeCaffeine", "PythonIsTrash", "NobodyWillReadThisxD"
	];

	// uso rnd para tener 3 aleatorios, se me olvido tambien comentar que uso rnd para la eleccion de cara o sello, es la otra dependencia que inegramos al inicio
	let mut rng = rand::thread_rng();
	let result_symbols: Vec<String> = (0..3)
		.map(|_| symbols[rng.gen_range(0..symbols.len())].to_string())
		.collect();

	//De los 3 aleatorios se asignan a su correspondiente variable
	let symbol1 = &result_symbols[0];
	let symbol2 = &result_symbols[1];
	let symbol3 = &result_symbols[2];

	//Verificacion de exito o perdida

	let (won, payout, win_type) = if symbol1 == symbol2 && symbol2 == symbol3 {
		// 3 iguales = pago x 10
		(true, form.amount * 10, Some("3 Iguales".to_string()))
	} else if symbol1 == symbol2 || symbol2 == symbol3 || symbol1 == symbol3 {
		// 2 iguales = pago x 3
		(true, form.amount * 3, Some("2 Iguales".to_string()))
	} else {
		// plop
		(false, 0, None)
	};

	let new_balance = if won {
		student.coins + payout - form.amount
	} else {
		student.coins - form.amount
	};

	student.coins = new_balance;

	Ok(Json(SlotsResult {
		won,
		symbols: result_symbols,
		new_balance,
		payout,
		amount_wagered: form.amount,
		win_type,
	}))
}

#[get("/slots")]
fn slots_page() -> RawHtml<&'static str> {
	RawHtml(r#"
	<!DOCTYPE html>
	<html>
	<head>
		<title>Slots Game - UCampus EduGame</title>
		<style>
			/* Reutilizar estilos del index más estilos específicos para slots */
			body {
				font-family: Arial, sans-serif;
				max-width: 800px;
				margin: 50px auto;
				padding: 20px;
				background: #f5f5f5;
			}
			.container {
				background: white;
				padding: 30px;
				border-radius: 10px;
				box-shadow: 0 2px 10px rgba(0,0,0,0.1);
			}
			h1 {
				color: #333;
				text-align: center;
				margin-bottom: 30px;
			}
			.navigation {
				text-align: center;
				margin-bottom: 20px;
			}
			.nav-button {
				display: inline-block;
				margin: 0 10px 10px;
				padding: 10px 20px;
				background: #007bff;
				color: white;
				text-decoration: none;
				border-radius: 5px;
				transition: background 0.3s;
			}
			.nav-button:hover {
				background: #0056b3;
			}
			.slots-machine {
				display: flex;
				justify-content: center;
				gap: 10px;
				margin: 30px 0;
				padding: 20px;
				background: #333;
				border-radius: 10px;
				position: relative;
			}
			.reel {
				width: 80px;
				height: 80px;
				background: white;
				border: 3px solid #ffd700;
				border-radius: 10px;
				display: flex;
				align-items: center;
				justify-content: center;
				font-size: 24px;
				font-weight: bold;
				position: relative;
				overflow: hidden;
			}
			.reel-content {
				display: flex;
				flex-direction: column;
				align-items: center;
				justify-content: center;
				height: 100%;
				transition: transform 0.1s ease;
			}
			.reel.spinning .reel-content {
				animation: spin 0.1s linear infinite;
			}
			.reel.stopping .reel-content {
				animation: none;
			}
			@keyframes spin {
				0% { transform: translateY(0); }
				100% { transform: translateY(-100px); }
			}
			@keyframes flash {
				0%, 100% { background: white; }
				50% { background: #ffd700; }
			}
			.reel.winner {
				animation: flash 0.5s ease-in-out 3;
			}
			.controls {
				text-align: center;
				margin: 20px 0;
			}
			.bet-input {
				padding: 10px;
				margin: 0 10px;
				border: 1px solid #ddd;
				border-radius: 5px;
				width: 100px;
			}
			.spin-btn {
				padding: 15px 30px;
				background: #28a745;
				color: white;
				border: none;
				border-radius: 5px;
				font-size: 18px;
				cursor: pointer;
				margin: 0 10px;
				transition: all 0.3s;
			}
			.spin-btn:disabled {
				background: #ccc;
				cursor: not-allowed;
			}
			.spin-btn:hover:not(:disabled) {
				background: #218838;
				transform: scale(1.05);
			}
			.result {
				text-align: center;
				margin: 20px 0;
				padding: 15px;
				border-radius: 5px;
				opacity: 0;
				transform: translateY(20px);
				transition: all 0.5s ease;
			}
			.result.show {
				opacity: 1;
				transform: translateY(0);
			}
			.win { 
				background: #d4edda; 
				color: #155724; 
				border: 2px solid #28a745;
			}
			.lose { 
				background: #f8d7da; 
				color: #721c24; 
				border: 2px solid #dc3545;
			}
			.coins-animation {
				position: absolute;
				top: 50%;
				left: 50%;
				transform: translate(-50%, -50%);
				font-size: 30px;
				color: #ffd700;
				animation: coinsFall 2s ease-out forwards;
				pointer-events: none;
			}
			@keyframes coinsFall {
				0% {
					opacity: 1;
					transform: translate(-50%, -50%) scale(1);
				}
				100% {
					opacity: 0;
					transform: translate(-50%, 50px) scale(1.5);
				}
			}
		</style>
	</head>
	<body>
		<div class="container">
			<h1>🎰 Slots Game</h1>
			<div class="navigation">
				<a href="/" class="nav-button">🏠 Inicio</a>
				<a href="/leaderboard" class="nav-button">🏆 Leaderboard</a>
				<a href="/coinflip" class="nav-button">🪙 Coin Flip Game</a>
				<a href="/shop" class="nav-button">🛒 Tienda</a>
			</div>
			
			<div id="balance" style="text-align: center; font-size: 18px; margin-bottom: 20px;">
				Monedas disponibles: <span id="balanceAmount">-</span> 🪙
			</div>
			
			<div class="slots-machine" id="slotsContainer">
				<div class="reel" id="reel1">
					<div class="reel-content">?</div>
				</div>
				<div class="reel" id="reel2">
					<div class="reel-content">?</div>
				</div>
				<div class="reel" id="reel3">
					<div class="reel-content">?</div>
				</div>
			</div>
			
			<div class="controls">
				<input type="text" id="username" placeholder="Tu email" style="margin-right: 10px; padding: 10px;">
				<input type="number" id="betAmount" placeholder="Apuesta" min="1" class="bet-input">
				<button id="spinBtn" class="spin-btn">Girar</button>
			</div>
			
			<div id="result"></div>
			
			<div style="text-align: center; margin-top: 30px; font-size: 14px;">
				<p><strong>Pagos:</strong></p>
				<p>3 símbolos iguales: 10x tu apuesta</p>
				<p>2 símbolos iguales: 3x tu apuesta</p>
			</div>
		</div>

		<script>
			const symbolEmojis = {
				'IHatePyhisics': '💻',
				'IDontLikeAlgebra': '🔥',
				'ILikeCounterStrike': '⚡',
				'IHateVisualStudio': '🌟',
				'ILikeSublimeText': '🎭',
				'ILikeCaffeine': '⚔️',
				'PythonIsTrash': '📊',
				'NobodyWillReadThisxD': '🌐'
			};

			const allSymbols = Object.keys(symbolEmojis);
			let isSpinning = false;

			document.getElementById('spinBtn').addEventListener('click', async () => {
				if (isSpinning) return;

				const username = document.getElementById('username').value;
				const betAmount = document.getElementById('betAmount').value;
				
				if (!username || !betAmount) {
					alert('Por favor ingresa tu email y monto de apuesta');
					return;
				}

				isSpinning = true;
				const spinBtn = document.getElementById('spinBtn');
				spinBtn.disabled = true;
				spinBtn.textContent = 'SPINNING...';

				const resultDiv = document.getElementById('result');
				resultDiv.classList.remove('show');
				resultDiv.innerHTML = '';

				try {
					startSpinAnimation();

					const response = await fetch('/play-slots', {
						method: 'POST',
						headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
						body: `username=${encodeURIComponent(username)}&amount=${betAmount}`
					});

					if (response.ok) {
						const result = await response.json();
						await displayResult(result);
					} else {
						const error = await response.text();
						stopAllSpins();
						document.getElementById('result').innerHTML = 
							`<div class="result lose show">Error: ${error}</div>`;
					}
				} catch (error) {
					stopAllSpins();
					document.getElementById('result').innerHTML = 
						`<div class="result lose show">Error de red: ${error.message}</div>`;
				} finally {
					isSpinning = false;
					spinBtn.disabled = false;
					spinBtn.textContent = 'SPIN';
				}
			});

			function startSpinAnimation() {
				for (let i = 1; i <= 3; i++) {
					const reel = document.getElementById(`reel${i}`);
					reel.classList.add('spinning');
					reel.classList.remove('winner');
					
					const content = reel.querySelector('.reel-content');
					const spinInterval = setInterval(() => {
						const randomSymbol = allSymbols[Math.floor(Math.random() * allSymbols.length)];
						content.textContent = symbolEmojis[randomSymbol];
					}, 100);
					
					reel.spinInterval = spinInterval;
				}
			}

			function stopReel(reelNumber, symbol, delay = 0) {
				return new Promise((resolve) => {
					setTimeout(() => {
						const reel = document.getElementById(`reel${reelNumber}`);
						const content = reel.querySelector('.reel-content');
						
						reel.classList.remove('spinning');
						reel.classList.add('stopping');
						
						if (reel.spinInterval) {
							clearInterval(reel.spinInterval);
							reel.spinInterval = null;
						}
						
						content.textContent = symbolEmojis[symbol] || symbol;
						
						setTimeout(() => {
							reel.classList.remove('stopping');
							resolve();
						}, 200);
					}, delay);
				});
			}

			function stopAllSpins() {
				for (let i = 1; i <= 3; i++) {
					const reel = document.getElementById(`reel${i}`);
					reel.classList.remove('spinning', 'stopping');
					if (reel.spinInterval) {
						clearInterval(reel.spinInterval);
						reel.spinInterval = null;
					}
				}
			}

			async function displayResult(result) {
				await stopReel(1, result.symbols[0], 500);
				await stopReel(2, result.symbols[1], 500);
				await stopReel(3, result.symbols[2], 500);

				document.getElementById('balanceAmount').textContent = result.new_balance;

				setTimeout(() => {
					const resultDiv = document.getElementById('result');
					if (result.won) {
						highlightWinningReels(result.symbols);
						
						showCoinsAnimation(result.payout);
						
						resultDiv.innerHTML = `
							<div class="result win">
								<h3>¡GANASTE! ${result.win_type}</h3>
								<p>Ganaste: ${result.payout} 🪙</p>
								<p>Nuevo balance: ${result.new_balance} 🪙</p>
							</div>
						`;
					} else {
						resultDiv.innerHTML = `
							<div class="result lose">
								<h3>No hay suerte esta vez</h3>
								<p>Perdiste: ${result.amount_wagered} 🪙</p>
								<p>Nuevo balance: ${result.new_balance} 🪙</p>
							</div>
						`;
					}
					
					setTimeout(() => {
						resultDiv.querySelector('.result').classList.add('show');
					}, 100);
				}, 300);
			}

			function highlightWinningReels(symbols) {
				const symbol1 = symbols[0];
				const symbol2 = symbols[1];
				const symbol3 = symbols[2];
				
				if (symbol1 === symbol2 && symbol2 === symbol3) {
					for (let i = 1; i <= 3; i++) {
						document.getElementById(`reel${i}`).classList.add('winner');
					}
				} else if (symbol1 === symbol2) {
					document.getElementById('reel1').classList.add('winner');
					document.getElementById('reel2').classList.add('winner');
				} else if (symbol2 === symbol3) {
					document.getElementById('reel2').classList.add('winner');
					document.getElementById('reel3').classList.add('winner');
				} else if (symbol1 === symbol3) {
					document.getElementById('reel1').classList.add('winner');
					document.getElementById('reel3').classList.add('winner');
				}
			}

			function showCoinsAnimation(payout) {
				const container = document.getElementById('slotsContainer');
				const coinsEl = document.createElement('div');
				coinsEl.className = 'coins-animation';
				coinsEl.textContent = `+${payout} 🪙`;
				container.appendChild(coinsEl);
				
				setTimeout(() => {
					container.removeChild(coinsEl);
				}, 2000);
			}

			document.getElementById('username').addEventListener('blur', async (e) => {
				if (e.target.value) {
					try {
						const response = await fetch(`/balance/${encodeURIComponent(e.target.value)}`);
						if (response.ok) {
							const student = await response.json();
							document.getElementById('balanceAmount').textContent = student.coins;
						}
					} catch (error) {
						console.log('No se pudo cargar el balance');
					}
				}
			});
		</script>
	</body>
	</html>
	"#)
}


//API - Formulario de login para scrapeo con los datos de usuario y contraseña
#[post("/scrape", data = "<form>")]
async fn scrape_handler(form: Form<LoginForm>, storage: &State<StudentStorage>) -> Result<Json<serde_json::Value>, rocket::response::status::Custom<String>> {
	match scrape_ucampus(form.username.clone(), form.password.clone()).await {
		Ok(student_data) => {
			let mut students = storage.write().await;
			let is_new_user = !students.contains_key(&form.username);
			students.insert(form.username.clone(), student_data.clone());
			
			let mut response = serde_json::to_value(&student_data).unwrap();
			response["is_new_user"] = serde_json::Value::Bool(is_new_user);
			
			Ok(Json(response))
		},
		Err(e) => Err(rocket::response::status::Custom(
			rocket::http::Status::InternalServerError,
			e.to_string()
		)),
	}
}

#[launch]
fn rocket() -> _ {
	let storage: StudentStorage = Arc::new(RwLock::new(HashMap::new()));

	rocket::build()
		.manage(storage)
		.mount("/", routes![index, scrape_handler, coinflip_page, leaderboard, get_balance, play_coinflip, shop_page, purchase_item, get_shop_items, slots_page, play_slots])
}