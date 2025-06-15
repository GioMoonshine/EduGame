import requests
import csv
import re
from bs4 import BeautifulSoup

class UCampusScraper:

    def __init__(self, username, password):

        self.username = username
        self.password = password
        self.session = requests.Session()
        self.session.headers.update({
            "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"
        })
        self.name = ""
        self.data = []

    def login(self):
        try:
            initial_page_url = "https://ucampus.uahurtado.cl/"
            initial = self.session.get(initial_page_url)
            initial.raise_for_status()
            sess_cookie = initial.cookies.get("_ucampus")

            if not sess_cookie:
                print("Error: Could not retrieve session cookie.")
                return False

            auth_api_url = "https://ucampus.uahurtado.cl/auth/api"
            payload = {
                "servicio": "ucampus",
                "debug": "0",
                "_sess": sess_cookie,
                "_LB": "uah02-int",
                "lang": "es",
                "username": self.username,
                "password": self.password,
                "recordar": "1"
            }

            response = self.session.post(auth_api_url, data=payload)
            response.raise_for_status()
            login_data = response.json()

            if login_data.get("status") == 200 and "u" in login_data:
                self.session.get(login_data["u"])
                return True
            else:
                print(f"Error al login. Status: {login_data.get('status')}. Message: {login_data.get('m')}")
                return False
        except requests.exceptions.RequestException as e:
            print(f"Error al ingresar (Nivel local): {e}")
            return False
        except ValueError:
            print("Fallo la wea de json.")
            return False


    def fetch_grades_and_attendance(self):

        cursos = [
            ("CSI0168", 2, "Electivo de Especialidad I"),
            ("CSI0169", 1, "Habilidades")
        ]

        for codigo, seccion, nombre in cursos:
            try:
                base_url = "https://ucampus.uahurtado.cl/uah/2025/1"
                grades_url = f"{base_url}/{codigo}/{seccion}/notas/alumno"
                att_url = f"{base_url}/{codigo}/{seccion}/asistencias2/"

                grades_resp = self.session.get(grades_url)
                grades_resp.raise_for_status()
                soup_grades = BeautifulSoup(grades_resp.text, "html.parser")

                att_resp = self.session.get(att_url)
                att_resp.raise_for_status()
                soup_att = BeautifulSoup(att_resp.text, "html.parser")

                if not self.name:
                    match = re.search(r"alias:\s*'([^']*)'", grades_resp.text)
                    if match:
                        self.name = match.group(1)

                grades = [tag.get_text(strip=True) for tag in soup_grades.select("h1.strong > span")]

                asistencia = "N/A"
                asistencia_header = soup_att.find("th", string=re.compile(r"\s*Asistencia\s*"))
                if asistencia_header:
                    asistencia_h1 = asistencia_header.find_next("h1")
                    if asistencia_h1:
                       asistencia_raw = asistencia_h1.get_text(strip=True)
                       asistencia_match = re.search(r'(\d+)', asistencia_raw)
                       if asistencia_match:
                           asistencia = asistencia_match.group(1) + "%"


                self.data.append({
                    "curso": nombre,
                    "notas": grades,
                    "asistencia": asistencia
                })

            except requests.exceptions.RequestException as e:
                print(f"Error mientras se obtenia la info de {nombre} con el error: {e}")
            except Exception as e:
                print(f"Error mientras se fraseaba la info de {nombre} con ele error: {e}")

    def save_to_csv(self, filename="datos_ucampus.csv"):

        if not self.data:
            return

        header = [
            "Nombre",
            "Curso habilidades", "Asistencia habilidades", "Notas habilidades",
            "Curso electivo", "Asistencia electivo", "Notas electivo"
        ]
        
        row_data = {"Nombre": self.name}
        for entry in self.data:
            course_name = entry["curso"]
            notes_str = ", ".join(entry["notas"])
            asistencia = entry["asistencia"]

            if "Habilidades" in course_name:
                row_data["Curso habilidades"] = course_name
                row_data["Asistencia habilidades"] = asistencia
                row_data["Notas habilidades"] = notes_str
            elif "Electivo" in course_name:
                row_data["Curso electivo"] = course_name
                row_data["Asistencia electivo"] = asistencia
                row_data["Notas electivo"] = notes_str

        try:
            with open(filename, mode="w", newline="", encoding="utf-8") as f:
                writer = csv.DictWriter(f, fieldnames=header)
                writer.writeheader()
                writer.writerow(row_data)
            print(f"Info guardada con exito en {filename}")
        except IOError as e:
            print(f"Error escribiendo el archivo {filename}: {e}")
