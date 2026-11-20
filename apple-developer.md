# Apple Developer ID für Bankiva Tauri

Diese Anleitung richtet die macOS-Builds für Developer-ID-Signierung und Apple-Notarisierung ein. Der Tauri-Workflow erwartet die fünf unten aufgeführten GitHub-Actions-Secrets. Ohne sie bricht der macOS-Build absichtlich ab.

## Rollen und Zuständigkeiten

Es gibt zwei getrennte Personenrollen:

- **Apple-Administrator / Account Holder:** Hat Zugriff auf den Apple Developer Account. Diese Person erstellt und verwaltet das Developer-ID-Zertifikat, erzeugt die Notarisierungsdaten und hinterlegt die GitHub-Actions-Secrets. Apple verlangt für das Erstellen eines Developer-ID-Zertifikats die Rolle **Account Holder**.
- **Entwickler ohne Apple Account:** Entwickelt die App, erstellt den Git-Tag und startet den Build über GitHub Actions beziehungsweise `bin/build`. Dafür braucht diese Person keinen Apple Account. Sie braucht Schreib-/Workflow-Zugriff auf das GitHub-Repository und eine gültige `gh`-Anmeldung. Der Build verwendet die vom Administrator eingerichteten Secrets automatisch; der Entwickler muss sie weder kennen noch erhalten.

Der Entwickler erstellt also weiterhin die App. Der Apple-Administrator richtet einmalig die Signier- und Notarisierungsidentität ein, die der macOS-Build zur Veröffentlichung benötigt.

## Voraussetzungen

- Ein Apple Developer Program Account. Für Softwareverteilung außerhalb des App Stores wird ein **Developer ID Application**-Zertifikat benötigt.
- Die Person, die das Zertifikat erstellt, muss im Apple Developer Team die Rolle **Account Holder** haben.
- Ein Terminal mit OpenSSL zum Erzeugen der Zertifikatsanforderung und zum Erstellen der PKCS#12-Datei. Der eigentliche Signier- und Notarisierungslauf erfolgt auf dem macOS-GitHub-Runner.
- Zugriff auf das GitHub-Repository `bankiva-tauri` und dessen Actions-Secrets.

Apple beschreibt Developer ID als Signaturweg für Mac-Apps außerhalb des App Stores. Für diesen Workflow wird zusätzlich die Notarisierung eingerichtet, damit Gatekeeper die heruntergeladene App verifizieren kann.

## Einrichtung durch den Apple-Administrator

Die folgenden Schritte 1 bis 4 erledigt der Apple-Administrator beziehungsweise Account Holder. Der Entwickler ohne Apple Account kann direkt mit [„macOS-Build durch den Entwickler starten“](#macos-build-durch-den-entwickler-starten) fortfahren, sobald diese Einrichtung abgeschlossen ist.

### 1. Private Schlüsseldatei und CSR im Terminal erstellen

Die folgenden Befehle funktionieren mit OpenSSL auf macOS oder Linux. Sie legen Schlüssel und CSR außerhalb des Repositorys in `~/DeveloperID-Bankiva` ab und schützen das Verzeichnis sowie den privaten Schlüssel mit restriktiven Dateirechten. Den Organisationsnamen bitte durch den im Apple Developer Account geführten Namen ersetzen.

```bash
umask 077
mkdir -p "$HOME/DeveloperID-Bankiva"
cd "$HOME/DeveloperID-Bankiva"
openssl genrsa -aes256 -out bankiva-developer-id.key 2048
chmod 600 bankiva-developer-id.key
openssl req -new \
  -key bankiva-developer-id.key \
  -out bankiva-developer-id.csr \
  -subj "/CN=Bankiva Developer ID/O=ORGANISATIONSNAME"
```

OpenSSL fragt nach einer Passphrase für die verschlüsselte private Schlüsseldatei. Diese Passphrase wird beim späteren Erstellen der `.p12` benötigt; sie ist **nicht** das `.p12`-Exportpasswort und wird nicht als GitHub-Secret gespeichert.

Apple stellt das Zertifikat weiterhin im Developer-Portal aus:

1. Im [Apple Developer Account](https://developer.apple.com/account/) **Certificates, Identifiers & Profiles → Certificates → +** öffnen.
2. Unter **Software** den Typ **Developer ID** wählen und **Developer ID Application** auswählen. Nicht „Developer ID Installer“ wählen: Der Workflow signiert die App und erstellt eine DMG.
3. `bankiva-developer-id.csr` hochladen, das Zertifikat erstellen und die ausgestellte `.cer`-Datei in `~/DeveloperID-Bankiva` speichern.

Apple dokumentiert Erstellung und Rollenanforderung unter [Developer ID certificates](https://developer.apple.com/help/account/certificates/create-developer-id-certificates). Der Apple-Portal-Schritt benötigt weiterhin den Account Holder; er lässt sich nicht durch einen lokalen Terminalbefehl ersetzen.

### 2. `.p12`-Datei im Terminal erstellen

Apple liefert das Zertifikat üblicherweise als DER-kodierte `.cer`-Datei. In diesem Fall zuerst zu PEM konvertieren und dann das Zertifikat mit dem privaten Schlüssel in eine PKCS#12-Datei bündeln:

```bash
cd "$HOME/DeveloperID-Bankiva"
openssl x509 -inform DER \
  -in bankiva-developer-id.cer \
  -out bankiva-developer-id.pem
openssl pkcs12 -export \
  -inkey bankiva-developer-id.key \
  -in bankiva-developer-id.pem \
  -out bankiva-developer-id.p12 \
  -name "Bankiva Developer ID"
chmod 600 bankiva-developer-id.p12
```

OpenSSL fragt nach der Passphrase des privaten Schlüssels und anschließend nach einem neuen Exportpasswort für die `.p12`. Das Exportpasswort wird später als `APPLE_CERTIFICATE_PASSWORD` gespeichert. Falls OpenSSL meldet, dass das Zertifikat nicht DER-kodiert ist, zuerst das tatsächliche Format prüfen und `-inform DER` weglassen.

Die `.p12`-Datei Base64-kodieren. Der folgende Befehl gibt den kodierten Inhalt aus, den du direkt in das passende GitHub-Secret einfügst:

   ```bash
   base64 -i "$HOME/DeveloperID-Bankiva/bankiva-developer-id.p12" | tr -d '\n'
   ```

   Die `.key`- und `.p12`-Dateien sowie der kodierte Inhalt enthalten den privaten Signaturschlüssel. Nicht ins Repository einchecken, nicht in Logs schreiben und nicht im Chat teilen. Nach erfolgreicher Einrichtung sicher verschlüsselt aufbewahren; die private Schlüsseldatei wird zum Erneuern oder Wiederherstellen des Zertifikats benötigt.

### 3. Apple-Daten für die Notarisierung vorbereiten

1. Die **Team ID** im Apple Developer Account auf der Membership-Seite nachsehen.
2. Für den Apple Account eine **App-spezifische Passphrase** erstellen. Der Account muss Zwei-Faktor-Authentifizierung verwenden. Auf [account.apple.com](https://account.apple.com/) unter **Sign-In and Security → App-Specific Passwords** eine Passphrase erstellen. Dafür nicht das normale Apple-Account-Passwort verwenden.

Die Passphrase wird als `APPLE_PASSWORD` gespeichert. Apple erklärt das Erstellen unter [App-spezifische Passwörter](https://support.apple.com/en-us/102654).

### 4. GitHub-Actions-Secrets eintragen

Im Repository `bankiva-tauri` **Settings → Secrets and variables → Actions → New repository secret** öffnen und diese Secrets anlegen:

| Secret | Wert |
| --- | --- |
| `APPLE_CERTIFICATE` | Base64-kodierter Inhalt der exportierten `.p12`-Datei |
| `APPLE_CERTIFICATE_PASSWORD` | Passwort, das beim `.p12`-Export gesetzt wurde |
| `APPLE_ID` | E-Mail-Adresse des Apple Accounts |
| `APPLE_PASSWORD` | App-spezifische Apple-Passphrase |
| `APPLE_TEAM_ID` | Team ID aus dem Apple Developer Account |

Der Workflow läuft auf einem macOS-Runner. Er erzeugt dort temporär einen Keychain, importiert das Zertifikat, sucht eine gültige **Developer ID Application**-Identität und übergibt die Apple-Daten an Tauri für die Notarisierung. Anschließend prüft er die Codesignatur, Gatekeeper-Freigabe und das Notarisierungsticket. Ein separates `KEYCHAIN_PASSWORD`-Secret wird nicht benötigt; der Workflow erzeugt dieses Passwort für den temporären Keychain selbst. Die lokale Linux-Umgebung kann die Apple-Codesignatur und Gatekeeper-Prüfung nicht ausführen; dafür sind die macOS-Schritte des GitHub-Workflows maßgeblich.

## macOS-Build durch den Entwickler starten

Der Entwickler ohne Apple Account startet den Workflow **Tauri App Builds** auf dem Git-Tag, dessen dreiteilige Versionsnummer als `APP_VERSION` angegeben wird. Alternativ kann er mit gültiger GitHub-CLI-Anmeldung die Builds starten und die Artefakte nach `4.0/public/download` verschieben:

```bash
gh auth login -h github.com
bin/build macos macos-intel
```

Das Script benötigt eine gültige GitHub-CLI-Anmeldung des Entwicklers und die in der Tauri-README beschriebene `.env` mit `APP_URL`. Es verschiebt die gebauten macOS-Dateien lokal nach `4.0/public/download`; es lädt sie nicht auf den Remote-Server hoch. Apple-Account-Zugangsdaten oder Zugriff auf den Inhalt der Actions-Secrets sind für den Entwickler nicht erforderlich.

## Fehlerbehebung

- **„Missing required GitHub Actions secret“**: Secret-Name und Repository prüfen; Secret-Werte sind nicht zwischen Repositories geteilt.
- **Keine Developer-ID-Identität gefunden**: Prüfen, dass die `.p12` sowohl Zertifikat als auch privaten Schlüssel enthält und vom Typ **Developer ID Application** ist.
- **Zertifikat lässt sich nicht importieren**: `APPLE_CERTIFICATE_PASSWORD` muss exakt dem beim `.p12`-Export festgelegten Passwort entsprechen.
- **Notarisierung schlägt fehl**: `APPLE_ID`, app-spezifische `APPLE_PASSWORD` und `APPLE_TEAM_ID` prüfen. Nach Änderung des normalen Apple-Account-Passworts kann Apple die app-spezifischen Passphrasen widerrufen.
- **App zeigt weiterhin „beschädigt“**: Im Actions-Lauf müssen sowohl die Signatur- als auch die Notarisierungsprüfung erfolgreich sein. Eine Ad-hoc-Signatur ersetzt Developer-ID-Signierung und Notarisierung für einen reibungslosen Download nicht.
