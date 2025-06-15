from flask import Flask, render_template, request, redirect, flash
from scraper import UCampusScraper

app = Flask(__name__)

@app.route("/", methods=["GET", "POST"])
def index():
    if request.method == "POST":
        username = request.form.get("username")
        password = request.form.get("password")

        scraper = UCampusScraper(username, password)
        if not scraper.login():
            flash("Login fallido. Revisa tus credenciales.", "error")
            return redirect("/")

        scraper.fetch_grades_and_attendance()
        scraper.save_to_csv()

        flash("Datos obtenidos y guardados correctamente, ahora espera a que aparezcas en el scoreboard", "success")
        return redirect("/")

    return render_template("index.html")

if __name__ == "__main__":
    app.run(debug=True)
