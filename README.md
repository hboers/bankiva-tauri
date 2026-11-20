# Bankiva App (Tauri)

Die Tauri-App zeigt die in der Projektdatei `.env` konfigurierte Bankiva-Webseite in einem nativen WebView-Fenster. Sie verwendet `APP_URL` (zum Beispiel `APP_URL=https://app4.bankiva.de`). Die Variable wird beim Paketbau aus der `.env` im Repository-Stamm gelesen und in die App eingebettet. Änderungen an `APP_URL` erfordern daher einen erneuten Build.

Beim Laden setzt die App auf genau dieser Origin den Session-Cookie `bankiva_tauri` auf `windows`, `macos`, `linux`, `android`, `ios` oder `other`. Die Webseite kann so Tauri-Aufrufe und das Betriebssystem der App erkennen. Der Cookie wird per JavaScript gesetzt, ist daher nicht `HttpOnly` und kein Sicherheitsnachweis; der erste HTTP-Aufruf enthält ihn noch nicht.

Pro Desktop-Plattform wird ein Paket erstellt: Linux als AppImage, macOS als DMG für Apple Silicon und zusätzlich als DMG für Intel-Macs, Windows als NSIS-EXE. Android erzeugt ein signiertes APK zur Geräteinstallation und ein signiertes AAB für Google Play. iOS erzeugt eine Simulator-App als ZIP und eine signierte IPA für echte iPhones und iPads sowie App Store Connect. Der iOS-Build benötigt ein Apple-Distribution-Zertifikat und ein App-Store-Provisioning-Profil. Selbstsignierte „Snakeoil“-Zertifikate funktionieren für iOS nicht.

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
CARGO_TARGET_DIR=../../build/macos-intel cargo tauri build --target x86_64-apple-darwin --bundles dmg # Intel-Mac: .dmg
CARGO_TARGET_DIR=../../build/windows cargo tauri build --bundles nsis # Windows: .exe
```

Für Android werden Android Studio/SDK, NDK und Java benötigt. Für iOS sind macOS, Xcode und CocoaPods erforderlich. Tauri initialisiert die nativen Projekte beim CI-Build:

```bash
cargo tauri android init --ci
CARGO_TARGET_DIR=../../build/android cargo tauri android build --apk --aab

cargo tauri ios init --ci
CARGO_TARGET_DIR=../../build/ios cargo tauri ios build --target aarch64-sim --ci
CARGO_TARGET_DIR=../../build/ios cargo tauri ios build --target aarch64 --export-method app-store-connect --ci
```

Die Android-Ausgaben liegen in `src-tauri/gen/android/app/build/outputs/`. Die iOS-Simulator-App und die exportierte IPA liegen unter `src-tauri/gen/apple/build/`.

Für lokale Android-Release-Pakete muss im Verzeichnis `src-tauri/gen/android/` zusätzlich eine `keystore.properties` mit `keyAlias`, `password` und `storeFile` liegen; der Keystore-Pfad muss in `storeFile` stehen. Danach `python3 bin/configure-android-signing` aus dem Repository-Stamm ausführen. Der CI-Workflow erstellt diese Datei temporär aus den beschriebenen GitHub Secrets.

## Build- und Veröffentlichungsdateien

| Datei / Einstellung | Zweck |
| --- | --- |
| `.env` im Repository-Stamm | Lokale Build-Zieladresse als `APP_URL=https://...`; wird in die App eingebettet. |
| `dotenv.example` | Vorlage für `.env`; enthält `APP_URL`. |
| `app/bankiva-app/src-tauri/tauri.conf.json` | Produktname, App-ID (`de.bankiva.app4`), WebView-Fenster und Bundle-Grundeinstellungen. Die App-ID muss auch in Apple App Store Connect registriert sein. |
| `app/bankiva-app/src-tauri/icons/` | Tauri-Icons für Desktop und Mobile, abgeleitet aus `../../4.0/public/logo512.png`. |
| `app/bankiva-app/src-tauri/Cargo.toml` und `Cargo.lock` | Tauri-Abhängigkeiten und festgelegte Rust-Paketversionen. |
| `.github/workflows/tauri-desktop.yml` | CI-Builds für Desktop, Android und iOS; URL-/Versions-Einbettung, Signierung und Artefakt-Upload. |
| `bin/build` | Startet den getaggten GitHub Actions Build und lädt Artefakte nach `install/<workflow-run-id>/`. |
| `bin/configure-android-signing` | Ergänzt nach `tauri android init` die Release-Signierung in der generierten Gradle-Datei. |

### Android-Signierung

Der Workflow benötigt eine dauerhaft aufbewahrte Android-Upload-Keystore-Datei. Ein selbstsigniertes Android-Zertifikat (hier als Snakeoil/Testzertifikat verwendet) kann als Upload-Key dienen, sofern derselbe Schlüssel für künftige Updates erhalten bleibt und sein Zertifikat in Google Play registriert wird. Die Datei und Passwörter gehören **nicht ins Repository**.

Einmalig lokal ein Keystore-Verzeichnis und einen Upload-Key erzeugen (Passwort interaktiv eingeben):

```bash
mkdir -p ~/.config/bankiva-app/signing
chmod 700 ~/.config/bankiva-app/signing
keytool -genkeypair -v \
  -keystore ~/.config/bankiva-app/signing/android-upload-snakeoil.jks \
  -alias bankiva-upload -keyalg RSA -keysize 2048 -validity 10000
```

Die gesamte `.jks`-Datei wird als Base64-Secret `ANDROID_KEY_BASE64` hinterlegt. Dazu kommen `ANDROID_KEY_PASSWORD` (Keystore- und Schlüsselpasswort) und `ANDROID_KEY_ALIAS` (hier `bankiva-upload`). Unter Linux kann der Base64-Wert ohne Zeilenumbrüche mit `base64 -w 0 ~/.config/bankiva-app/signing/android-upload-snakeoil.jks` erzeugt werden; unter macOS mit `base64 -i ~/.config/bankiva-app/signing/android-upload-snakeoil.jks`.

Die drei Werte gehören in GitHub unter **Repository → Settings → Secrets and variables → Actions → New repository secret**. Im Workflow werden sie nur temporär als `src-tauri/gen/android/keystore.properties` und `$RUNNER_TEMP/bankiva-upload-key.jks` abgelegt. Der generierte Gradle-Build verwendet sie für Release-APK und AAB.

Für produktive Google-Play-Veröffentlichungen empfiehlt sich ein dedizierter, gesicherter Upload-Key und Play App Signing. Wenn ein echter Upload-Key ersetzt wird, muss Google Play den neuen Upload-Zertifikat-Fingerprint akzeptieren. Den Schlüssel nicht wechseln oder verlieren, ohne zuvor den Play-Console-Ablauf für einen Upload-Key-Reset zu erledigen.

### iOS/iPadOS-Signierung

Apple akzeptiert keine selbstsignierten Zertifikate für Installationen auf realen Geräten oder App Store Connect. Erforderlich sind ein Apple Developer Program Team, die registrierte explizite App-ID `de.bankiva.app4`, ein **Apple Distribution** Zertifikat als exportierte `.p12`-Datei und ein dazu passendes **App Store Connect** Provisioning-Profil (`.mobileprovision`). Das Zertifikat muss den privaten Schlüssel enthalten; das Profil muss zu App-ID und Zertifikat passen.

Die Dateien nicht in Git ablegen. Als GitHub Repository Actions Secrets speichern:

| Secret | Inhalt |
| --- | --- |
| `IOS_CERTIFICATE` | Base64 der `.p12` Datei |
| `IOS_CERTIFICATE_PASSWORD` | Export-Passwort der `.p12` Datei |
| `IOS_MOBILE_PROVISION` | Base64 der App-Store-Connect `.mobileprovision` Datei |

Auf macOS kann Base64 mit `base64 -i datei.p12` beziehungsweise `base64 -i profil.mobileprovision` erstellt werden. Tauri liest die Werte aus Umgebungsvariablen; die Dateien müssen nicht im Checkout abgelegt werden. Der Workflow bricht den iOS-Geräte-/Store-Build mit einer verständlichen Meldung ab, wenn eines der drei Secrets fehlt. Apple App Store Connect API-Schlüssel sind für das Erstellen der IPA nicht nötig; sie wären zusätzlich erforderlich, falls der Workflow künftig auch den Store-Upload automatisieren soll.

Die iOS-IPA ist für echte iPhones und iPads über App Store Connect/TestFlight vorgesehen. Der Simulator-ZIP-Build bleibt zum Testen im Simulator. Google Play bzw. Apple können die Pakete nach dem Upload noch prüfen; der CI-Build selbst veröffentlicht nicht automatisch.

### Bild-Uploads

Die Tauri-App verwendet die Tauri-Standardbehandlung für Berechtigungsanfragen der geladenen Webseite. Es sind keine zusätzlichen `Info.plist`-Einträge, Entitlements oder Android-Berechtigungen konfiguriert.

Nach dem Laden der Bankiva-Seite komprimiert die App große Bilddateien in der WebView vor dem Absenden: maximal 2000 Pixel Kantenlänge und JPEG-Qualität 78 %. Kleine Bilder bleiben unverändert. Die Komprimierung gilt für Bilddateien in Datei-Upload-Feldern, etwa bei Verlustmeldungen und Dokumenten; andere Dateien wie PDFs werden unverändert übertragen. Browser/WebView-Kompatibilität und tatsächliche Upload-Validierung müssen auf den Zielgeräten geprüft werden.

## Builds für alle Plattformen

Der manuelle GitHub Actions Workflow `Tauri App Builds` baut standardmäßig Linux, macOS für Apple Silicon und Intel, Windows, Android und iOS-Simulator separat. Der Intel-macOS-Build läuft auf `macos-15-intel` und verwendet Rust-Target `x86_64-apple-darwin`. Beim Start des Workflows sind `APP_URL` und `APP_VERSION` anzugeben. `APP_VERSION` muss aus drei numerischen Teilen bestehen und zum Git-Tag passen, zum Beispiel `0.1.1`. Android liefert APK/AAB, iOS eine Simulator-App als ZIP. Eine signierte IPA zur Installation auf realen Geräten erfordert zusätzlich Apple-Zertifikate und ein Provisioning-Profil.

Mit `bin/build` lassen sich Plattformen auswählen. `bin/build all` baut alle unterstützten Plattformen. `bin/build desktop` baut Windows, Linux sowie macOS für Apple Silicon und Intel. Auch einzelne Desktop-Plattformen wie `bin/build macos macos-intel` werden nach erfolgreichem Build nach `4.0/public/download` verschoben. Das Script kopiert keine Dateien auf den Anwendungsserver. Vor dem Verschieben prüft es Dateinamen und überschreibt keine gleichnamigen Pakete. Ältere Pakete derselben Plattform sichert es nach `archive/previous-downloads/<workflow-run-id>/` und entfernt sie anschließend aus `4.0/public/download`. Danach wird `install/<workflow-run-id>/` gelöscht.

macOS-Downloads benötigen eine Apple-Developer-ID-Application-Zertifikatsdatei im PKCS#12-Format sowie Notarisierungsdaten. Als GitHub Actions Secrets sind `APPLE_CERTIFICATE` (die Base64-kodierte `.p12`-Datei), `APPLE_CERTIFICATE_PASSWORD`, `APPLE_ID`, `APPLE_PASSWORD` (ein App-spezifisches Apple-Passwort) und `APPLE_TEAM_ID` einzurichten. Ohne diese Secrets bricht der macOS-Build vor dem Paketieren ab. Der Workflow importiert das Zertifikat in einen temporären Keychain, signiert und notarisiert die Apps und prüft Signatur, Gatekeeper-Freigabe und Notarisierungsticket.

Das Script liest `APP_URL` aus `.env`, verwendet das numerische Git-Tag am aktuellen Commit als Paketversion und startet den Workflow auf diesem Tag. Commit und Tag müssen bereits auf GitHub liegen. Das Script wartet auf den Abschluss und lädt die Pakete nach `install/<workflow-run-id>/` herunter. Der Commit mit dem Workflow muss auf GitHub liegen und das Tag muss dorthin gepusht sein. Voraussetzung sind eine gültige `gh`-Anmeldung (`gh auth login`) und eine `.env` mit gültiger `APP_URL`.

```bash
bin/build linux
bin/build windows macos macos-intel
bin/build android ios
bin/build desktop
bin/build all
bin/build --help
```

Ein Release-Tag enthält ausschließlich drei Zahlen, getrennt durch Punkte. Nach dem Commit wird das Tag angelegt und zusammen mit dem Commit veröffentlicht. Der Workflow akzeptiert nur Builds, bei denen die angegebene Paketversion exakt dem ausgewählten Tag entspricht:

```bash
git tag 0.1.1
git push origin main 0.1.1
bin/build all
```
