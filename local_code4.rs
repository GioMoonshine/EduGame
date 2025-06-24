use rocket::serde::{Deserialize, Serialize, json::Json};
use rocket::{get, post, routes, launch, form::Form, response::content::RawHtml};
use reqwest::Client;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, rocket::form::FromForm)]
#[serde(crate = "rocket::serde")]
struct LoginForm {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
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
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse {
    status: i32,
    u: Option<String>,
}

#[derive(Debug)]
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

async fn scrape_ucampus(username: String, password: String) -> Result<StudentData, Box<dyn std::error::Error>> {
    let client = Client::builder()
        .cookie_store(true)
        .build()?;

    let cookie_response = client
        .get("https://ucampus.uahurtado.cl/")
        .send()
        .await?;

    let cookies = cookie_response.cookies().collect::<Vec<_>>();
    let sess_cookie = cookies
        .iter()
        .find(|c| c.name() == "_ucampus")
        .ok_or("Session cookie not found")?;

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
        .header("User-Agent", "GamificationEngineRuntime/7.44.1")
        .form(&login_data)
        .send()
        .await?;

    let login_json: ApiResponse = login_response.json().await?;

    if login_json.status != 200 {
        return Err("Login failed. Please check your username and password.".into());
    }

    if let Some(main_url) = login_json.u {
        client.get(&main_url).send().await?;
    }

    let grades1_response = client
        .get("https://ucampus.uahurtado.cl/uah/2025/1/CSI0168/2/notas/alumno")
        .send()
        .await?;
    let grades1_text = grades1_response.text().await?;
    println!("Grades 1 response: {}", grades1_text);

    let attendance1_response = client
        .get("https://ucampus.uahurtado.cl/uah/2025/1/CSI0168/2/asistencias2/")
        .send()
        .await?;
    let attendance1_text = attendance1_response.text().await?;
    println!("============================================================================\nAttendance 1 response============================================================================\n{}", attendance1_text);
    
    let grades2_response = client
        .get("https://ucampus.uahurtado.cl/uah/2025/1/CSI0169/1/notas/alumno")
        .send()
        .await?;
    let grades2_text = grades2_response.text().await?;
    println!("============================================================================\nGrade 2 response============================================================================\n{}", grades2_text);

    let attendance2_response = client
        .get("https://ucampus.uahurtado.cl/uah/2025/1/CSI0169/1/asistencias2/")
        .send()
        .await?;
    let attendance2_text = attendance2_response.text().await?;
    println!("============================================================================\nAttendance 2 response============================================================================\n{}", attendance2_text);

    let name = extract_between(&grades1_text, "alias: '", "',")
        .unwrap_or("Unknown".to_string());

    let electivo_grades = vec![
        extract_between(&grades1_text, r#"<h1 class="strong"><span class="">"#, "</span></h1>")
            .unwrap_or("N/A".to_string()),
        extract_nth_between(&grades1_text, r#"<h1 class="strong"><span class="">"#, "</span></h1>", 2)
            .unwrap_or("N/A".to_string()),
    ];

    let electivo_attendance = extract_attendance(&attendance1_text, "<th>Asistencia", "%</h1>")
        .unwrap_or("N/A".to_string());

    let habilidades_grades = vec![
        extract_between(&grades2_text, r#"<h1 class="strong"><span class="">"#, "</span></h1>")
            .unwrap_or("N/A".to_string()),
        extract_nth_between(&grades2_text, r#"<h1 class="strong"><span class="">"#, "</span></h1>", 2)
            .unwrap_or("N/A".to_string()),
        extract_nth_between(&grades2_text, r#"<h1 class="strong"><span class="">"#, "</span></h1>", 3)
            .unwrap_or("N/A".to_string()),
        extract_nth_between(&grades2_text, r#"<h1 class="strong"><span class="">"#, "</span></h1>", 4)
            .unwrap_or("N/A".to_string()),
    ];

    let habilidades_attendance = extract_attendance(&attendance2_text, "<th>Asistencia", "%</h1>")
        .unwrap_or("N/A".to_string())
        .replace(">", "");

    let mut notas = 0.0;
    let mut cuenta = 0;
    for i in electivo_grades.len() {
        notas += electivo_grades[i];
        cuenta += 1;
    }
    for i in habilidades_grades.len() {
        notas += habilidades_grades[i];
        cuenta += 1;
    }
    grades = notas * 10 as u16;
    mean = (notas/cuenta) as f32;

    let mut asistencia = (electivo_attendance + habilidades_attendance)/2;
    assist = asistencia as u8;

    Ok(Student {
        name,
        assist,
        grades,
        mean,
    })
}

}

impl Student {
    fn new(name: String, assist: u8, grades: u16) -> Self {
        Student {
            name,
            assist,
            grades,
            mean: 0.0
            exp: 0,
            level: 1,
            penalty: 0,
            bonus: 0,
        }
    }

    fn apply_gammification(&mut self, level_system: &LevelSystem) {
        let base_exp = ((self.grades - 1.0) / 6.0) * 50.0 + (self.assist as f32 / 100.0) * 50.0;
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

fn extract_between(text: &str, start: &str, end: &str) -> Option<String> {
    let start_pos = text.find(start)? + start.len();
    let end_pos = text[start_pos..].find(end)? + start_pos;
    Some(text[start_pos..end_pos].to_string())
}

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

fn extract_all_grades(html: &str) -> Vec<String> {
    let mut grades = Vec::new();
    let pattern = r#"<h1 class="strong"><span class="">"#;
    let end_pattern = "</span></h1>";
    
    for i in 1..=6 {
        if let Some(grade) = extract_nth_between(html, pattern, end_pattern, i) {
            let clean_grade = grade.trim().to_string();
            if !clean_grade.is_empty() && 
               clean_grade != "-" && 
               clean_grade != "0.0" && 
               clean_grade != "0" &&
               !clean_grade.chars().all(|c| c.is_whitespace()) {
                grades.push(clean_grade);
            }
        } else {
            break;
        }
    }
    
    grades
}

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
        <title>UCampus Scraper</title>
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
        </style>
    </head>
    <body>
        <div class="container">
            <h1>UCampus Gamificacion EduGame</h1>
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
                <button type="submit" id="submitBtn">Get Grades</button>
            </form>
            <div id="result"></div>
        </div>

        <script>
            document.getElementById('loginForm').addEventListener('submit', async (e) => {
                e.preventDefault();
                
                const submitBtn = document.getElementById('submitBtn');
                const resultDiv = document.getElementById('result');
                
                submitBtn.disabled = true;
                submitBtn.textContent = 'Loading...';
                resultDiv.innerHTML = '<div class="result loading">Scraping data, please wait...</div>';
                
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
                    submitBtn.textContent = 'Get Grades';
                }
            });
            
            function displayResults(data) {
                const resultDiv = document.getElementById('result');
                resultDiv.innerHTML = `
                    <div class="result">
                        <h2>Student: ${data.name}</h2>
                        
                        <div class="grade-section">
                            <h3>Electivo</h3>
                            ${data.electivo_grades.map((grade, i) => 
                                `<div class="grade-item">Grade ${i + 1}: ${grade}</div>`
                            ).join('')}
                            <div class="grade-item">Attendance: ${data.electivo_attendance}%</div>
                        </div>
                        
                        <div class="grade-section">
                            <h3>Habilidades</h3>
                            ${data.habilidades_grades.map((grade, i) => 
                                `<div class="grade-item">Grade ${i + 1}: ${grade}</div>`
                            ).join('')}
                            <div class="grade-item">Attendance: ${data.habilidades_attendance}%</div>
                        </div>
                    </div>
                `;
            }
        </script>
    </body>
    </html>
    "#)
}

#[post("/scrape", data = "<form>")]
async fn scrape_handler(form: Form<LoginForm>) -> Result<Json<StudentData>, rocket::response::status::Custom<String>> {
    match scrape_ucampus(form.username.clone(), form.password.clone()).await {
        Ok(data) => Ok(Json(data)),
        Err(e) => Err(rocket::response::status::Custom(
            rocket::http::Status::InternalServerError,
            e.to_string()
        )),
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, scrape_handler])
        .attach(rocket_cors::CorsOptions::default().to_cors().unwrap())
}