# Bankiva App (Tauri)

Die Tauri-App zeigt die in der Projektdatei `.env` konfigurierte Bankiva-Webseite in einem nativen WebView-Fenster. Sie verwendet `APP_URL` (zum Beispiel `APP_URL=https://app4.bankiva.de`). Die Variable wird beim Paketbau aus der `.env` im Repository-Stamm gelesen und in die App eingebettet. Änderungen an `APP_URL` erfordern daher einen erneuten Build.

Pro Desktop-Betriebssystem wird ein Paket erstellt: Linux als AppImage, macOS als DMG und Windows als NSIS-EXE. Android erzeugt APK und AAB. Der iOS-Workflow erzeugt eine App für den iOS-Simulator als ZIP. Für eine auf iPhone oder iPad installierbare IPA sind Apple-Signing-Zertifikate und ein Provisioning-Profil erforderlich; sie werden hier nicht eingerichtet.

Das AppImage bündelt die App, benötigt aber weiterhin die passenden WebKitGTK-/GTK-Laufzeitbibliotheken des Linux-Systems. macOS stellt WebKit mit dem Betriebssystem bereit. Unter Windows wird die WebView2-Laufzeit benötigt; der NSIS-Installer verwendet den kleinen WebView2-Bootstrapper und lädt die Laufzeit bei Bedarf aus dem Internet.

## Lokal bauen

Die gemeinsamen Quelltexte liegen unter `app/bankiva-app`. Plattform-Builds werden getrennt unter `build/<plattform>` abgelegt.

Die App benötigt kein separates Frontend und keine Node.js-Abhängigkeiten. Die leere statische WebView-Datei dient nur als Tauri-Build-Eingabe; das Fenster lädt direkt die beim Build eingebettete `APP_URL`.

Im Verzeichnis `app/bankiva-app`:

```bash
cargo install tauri-cli --version '^2' --locked
CARGO_TARGET_DIR=../../build/linux cargo tauri build
```

Tauri erzeugt das Paket im jeweiligen Plattform-Bundle-Verzeichnis unter `build/<plattform>/release/bundle/`. Linux benötigt zum Bauen zusätzlich WebKitGTK 4.1 und die in der [Tauri-Linux-Dokumentation](https://tauri.app/start/prerequisites/#linux) genannten Build-Abhängigkeiten. macOS- und Windows-Pakete müssen auf macOS beziehungsweise Windows gebaut werden.

Der Standardbuild erzeugt das Linux-AppImage. Auf macOS und Windows das Format explizit wählen:

```bash
CARGO_TARGET_DIR=../../build/macos cargo tauri build --bundles dmg    # macOS: .dmg
CARGO_TARGET_DIR=../../build/windows cargo tauri build --bundles nsis # Windows: .exe
```

Für Android werden Android Studio/SDK, NDK und Java benötigt. Für iOS sind macOS, Xcode und CocoaPods erforderlich. Tauri initialisiert die nativen Projekte beim CI-Build:

```bash
cargo tauri android init --ci
CARGO_TARGET_DIR=../../build/android cargo tauri android build --apk --aab

cargo tauri ios init --ci
CARGO_TARGET_DIR=../../build/ios cargo tauri ios build --target aarch64-sim --ci
```

Die Android-Ausgaben liegen in `src-tauri/gen/android/app/build/outputs/`. Die iOS-Simulator-App liegt unter `src-tauri/gen/apple/build/`.

## Builds für alle Plattformen

Der manuelle GitHub Actions Workflow `Tauri App Builds` baut standardmäßig Linux, macOS, Windows, Android und iOS-Simulator separat. Beim Start des Workflows sind `APP_URL` und `APP_VERSION` anzugeben. `APP_VERSION` muss aus drei numerischen Teilen bestehen und zum Git-Tag passen, zum Beispiel `0.1.1`. Android liefert APK/AAB, iOS eine Simulator-App als ZIP. Eine signierte IPA zur Installation auf realen Geräten erfordert zusätzlich Apple-Zertifikate und ein Provisioning-Profil.

Mit `bin/build` lassen sich Plattformen auswählen. `bin/build all` baut alle unterstützten Plattformen. Das Script liest `APP_URL` aus `.env`, verwendet das numerische Git-Tag am aktuellen Commit als Paketversion und startet den Workflow auf diesem Tag. Commit und Tag müssen bereits auf GitHub liegen. Das Script wartet auf den Abschluss und lädt die Pakete nach `install/<workflow-run-id>/` herunter. Der Commit mit dem Workflow muss auf GitHub liegen und das Tag muss dorthin gepusht sein. Voraussetzung sind eine gültige `gh`-Anmeldung (`gh auth login`) und eine `.env` mit gültiger `APP_URL`.

```bash
bin/build linux
bin/build windows macos
bin/build android ios
bin/build all
bin/build --help
```

Ein Release-Tag enthält ausschließlich drei Zahlen, getrennt durch Punkte. Nach dem Commit wird das Tag angelegt und zusammen mit dem Commit veröffentlicht. Der Workflow akzeptiert nur Builds, bei denen die angegebene Paketversion exakt dem ausgewählten Tag entspricht:

```bash
git tag 0.1.1
git push origin main 0.1.1
bin/build all
```
