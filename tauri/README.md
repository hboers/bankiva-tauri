# Bankiva Desktop (Tauri)

Die Tauri-App zeigt die in der Projektdatei `.env` konfigurierte Bankiva-Webseite in einem nativen WebView-Fenster. Sie verwendet `APP_URL` (zum Beispiel `APP_URL=https://app4.bankiva.de`). Die Variable wird beim Paketbau aus der `.env` im Repository-Stamm gelesen und in die App eingebettet. Änderungen an `APP_URL` erfordern daher einen erneuten Build.

Pro Betriebssystem wird genau ein Paket erstellt: Linux als AppImage, macOS als DMG und Windows als NSIS-EXE. Das AppImage bündelt die App, benötigt aber weiterhin die passenden WebKitGTK-/GTK-Laufzeitbibliotheken des Linux-Systems. macOS stellt WebKit mit dem Betriebssystem bereit. Unter Windows wird die WebView2-Laufzeit benötigt; der NSIS-Installer verwendet den kleinen WebView2-Bootstrapper und lädt die Laufzeit bei Bedarf aus dem Internet.

## Lokal bauen

Die gemeinsamen Quelltexte liegen unter `tauri/app/bankiva-app`. Plattform-Builds werden getrennt unter `tauri/build/linux`, `tauri/build/macos` und `tauri/build/windows` abgelegt.

Im Verzeichnis `tauri/app/bankiva-app`:

```bash
npm ci
CARGO_TARGET_DIR=../../build/linux npm run tauri build
```

Tauri erzeugt das Paket im jeweiligen Plattform-Bundle-Verzeichnis unter `tauri/build/<plattform>/release/bundle/`. Linux benötigt zum Bauen zusätzlich WebKitGTK 4.1 und die in der [Tauri-Linux-Dokumentation](https://tauri.app/start/prerequisites/#linux) genannten Build-Abhängigkeiten. macOS- und Windows-Pakete müssen auf macOS beziehungsweise Windows gebaut werden.

Der Standardbuild erzeugt das Linux-AppImage. Auf macOS und Windows das Format explizit wählen:

```bash
CARGO_TARGET_DIR=../../build/macos npm run tauri build -- --bundles dmg   # macOS: einzelnes .dmg
CARGO_TARGET_DIR=../../build/windows npm run tauri build -- --bundles nsis  # Windows: einzelne .exe
```

## Builds für alle Plattformen

Der manuelle GitHub Actions Workflow `Tauri Desktop Builds` baut Linux, macOS und Windows separat. Beim Start des Workflows ist `APP_URL` als Eingabe anzugeben; diese URL wird in die drei Pakete eingebettet. Die erzeugten AppImage-, DMG- und EXE-Dateien stehen als Workflow-Artefakte bereit.
