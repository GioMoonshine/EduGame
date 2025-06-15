import requests

req = requests.session()

username = "example@alumnos.uahurtado.cl"
password = "example"

cookieMonster = req.get("https://ucampus.uahurtado.cl/")

sess = cookieMonster.cookies["_ucampus"]


data = {
	"servicio" : "ucampus",
	"debug":"0",
	"_sess": sess,
	"_LB": "uah02-int",
	"lang":"es",
	"username": username,
	"password": password,
	"recordar":"1",
}

data2 = {
	"User-Agent":"GamificationEngineRuntime/7.44.1"
}

login = req.post("https://ucampus.uahurtado.cl/auth/api", data=data, headers=data2)

loginData = login.json()

if loginData["status"] != 200:
	print("Login failed. Please check your username and password.")
else:
	mainPage = req.get(loginData["u"])

	grades1 = req.get("https://ucampus.uahurtado.cl/uah/2025/1/CSI0168/2/notas/alumno")
	assaignment1 = req.get("https://ucampus.uahurtado.cl/uah/2025/1/CSI0168/2/asistencias2/")


	assaignment1Data = assaignment1.text

	grades1Data = grades1.text


	grade1 = grades1Data.split('<h1 class="strong"><span class="">')[1].split('</span></h1>')[0]
	grade2 = grades1Data.split('<h1 class="strong"><span class="">')[2].split('</span></h1>')[0]
	name = grades1Data.split("alias: '")[1].split("',")[0]

	assitencia1 = assaignment1Data.split("<th>Asistencia")[1].split("%</h1>")[0][-3:]

	grades2 = req.get("https://ucampus.uahurtado.cl/uah/2025/1/CSI0169/1/notas/alumno")
	assaignment2 = req.get("https://ucampus.uahurtado.cl/uah/2025/1/CSI0169/1/asistencias2/")


	assaignment2Data = assaignment2.text

	grades2Data = grades2.text

	grade3 = grades2Data.split('<h1 class="strong"><span class="">')[1].split('</span></h1>')[0]
	grade4 = grades2Data.split('<h1 class="strong"><span class="">')[2].split('</span></h1>')[0]
	grade5 = grades2Data.split('<h1 class="strong"><span class="">')[3].split('</span></h1>')[0]
	grade6 = grades2Data.split('<h1 class="strong"><span class="">')[4].split('</span></h1>')[0]

	assitencia2 = assaignment2Data.split("<th>Asistencia")[1].split("%</h1>")[0][-3:].replace(">","")

	print("Electivo:")
	print(grade1)
	print(grade2)

	print(assitencia1)

	print("Habilidades")
	print(grade3)
	print(grade4)
	print(grade5)
	print(grade6)

	print(assitencia2)
	print(name)
