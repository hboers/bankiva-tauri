# Tauri unter Debian 13 installieren und ausprobieren

## 1. Systempakete installieren

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

Diese Pakete stellen WebKitGTK und die Build-Werkzeuge bereit, die Tauri für Linux benötigt.

## 2. Rust und Node.js bereitstellen

Prüfe zuerst, ob die Werkzeuge bereits installiert sind:

```bash
rustc --version
cargo --version
node --version
npm --version
```

Falls Rust fehlt, installiere es mit `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Danach das Terminal neu öffnen oder die Umgebungsdateien laden. Für Node.js wird eine aktuelle Version benötigt.

## 3. Beispiel-App erstellen und starten

```bash
npm create tauri-app@latest
```

Im Assistenten kannst du zum Beispiel **Vanilla** als Frontend wählen. Anschließend:

```bash
cd tauri-app
npm install
npm run tauri dev
```

Das sollte ein eigenes App-Fenster öffnen.

## 4. Eigene Webseite einbinden

Zum schnellen Ausprobieren kannst du in `src-tauri/tauri.conf.json` unter `build.frontendDist` die URL deiner Website eintragen. Dann lädt die App diese Adresse. Die Website muss online erreichbar sein.

Für eine verteilbare App ist es meist besser, die gebauten Webseitendateien in die App einzupacken.

## Hinweis zum Bauen für andere Linux-Systeme

Tauri empfiehlt, Linux-Pakete auf der ältesten Linux-Version zu bauen, die du unterstützen möchtest. So lassen sich Probleme mit unterschiedlichen Systembibliotheken vermeiden.
