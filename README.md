# Bankiva Desktop (Tauri)

Die Tauri-App zeigt die in der Projektdatei `.env` konfigurierte Bankiva-Webseite in einem nativen WebView-Fenster. Sie verwendet `APP_URL` (zum Beispiel `APP_URL=https://app4.bankiva.de`). Die Variable wird beim Paketbau aus der `.env` im Repository-Stamm gelesen und in die App eingebettet. Änderungen an `APP_URL` erfordern daher einen erneuten Build.

Pro Betriebssystem wird genau ein Paket erstellt: Linux als AppImage, macOS als DMG und Windows als NSIS-EXE. Das AppImage bündelt die App, benötigt aber weiterhin die passenden WebKitGTK-/GTK-Laufzeitbibliotheken des Linux-Systems. macOS stellt WebKit mit dem Betriebssystem bereit. Unter Windows wird die WebView2-Laufzeit benötigt; der NSIS-Installer verwendet den kleinen WebView2-Bootstrapper und lädt die Laufzeit bei Bedarf aus dem Internet.

## Lokal bauen

Die gemeinsamen Quelltexte liegen unter `app/bankiva-app`. Plattform-Builds werden getrennt unter `build/linux`, `build/macos` und `build/windows` abgelegt.

Die App benötigt kein separates Frontend und keine Node.js-Abhängigkeiten. Die leere statische WebView-Datei dient nur als Tauri-Build-Eingabe; das Fenster lädt direkt die beim Build eingebettete `APP_URL`.

Im Verzeichnis `app/bankiva-app`:

```bash
cargo install tauri-cli --version '^2' --locked
CARGO_TARGET_DIR=../../build/linux cargo tauri build
```

Tauri erzeugt das Paket im jeweiligen Plattform-Bundle-Verzeichnis unter `build/<plattform>/release/bundle/`. Linux benötigt zum Bauen zusätzlich WebKitGTK 4.1 und die in der [Tauri-Linux-Dokumentation](https://tauri.app/start/prerequisites/#linux) genannten Build-Abhängigkeiten. macOS- und Windows-Pakete müssen auf macOS beziehungsweise Windows gebaut werden.

Der Standardbuild erzeugt das Linux-AppImage. Auf macOS und Windows das Format explizit wählen:

```bash
CARGO_TARGET_DIR=../../build/macos cargo tauri build --bundles dmg   # macOS: einzelnes .dmg
CARGO_TARGET_DIR=../../build/windows cargo tauri build --bundles nsis  # Windows: einzelne .exe
```

## Builds für alle Plattformen

Der manuelle GitHub Actions Workflow `Tauri Desktop Builds` baut standardmäßig Linux, macOS und Windows separat. Beim Start des Workflows ist `APP_URL` als Eingabe anzugeben; diese URL wird in die Pakete eingebettet. Die erzeugten AppImage-, DMG- und EXE-Dateien stehen als Workflow-Artefakte bereit.

Mit `bin/build.sh` lassen sich einzelne Plattformen auswählen. Das Script liest `APP_URL` aus `.env`, startet den Workflow auf dem aktuellen Git-Branch, wartet auf den Abschluss und lädt die Pakete nach `install/<workflow-run-id>/` herunter. Der Branch samt Workflow-Änderungen muss auf GitHub verfügbar sein. Voraussetzung sind eine gültige `gh`-Anmeldung (`gh auth login`) und eine `.env` mit gültiger `APP_URL`.

```bash
bin/build.sh linux
bin/build.sh windows macos
bin/build.sh --help
```
