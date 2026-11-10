# Tauri Desktop-App

- Die native Desktop-App liegt unter `tauri/linux/bankiva-app` und verwendet Tauri 2.
- Beim Paketbau liest `src-tauri/build.rs` ausschließlich `APP_URL` aus der `.env` im Repository-Stamm. Die URL muss absolut sein und mit `http://` oder `https://` beginnen.
- `APP_URL` wird zur Build-Zeit in die App eingebettet. Für eine geänderte Zieladresse müssen die Pakete neu gebaut werden.
- Das native Tauri-Fenster lädt die eingebettete URL direkt in seinem WebView.
- Native Pakete werden auf dem jeweiligen Zielbetriebssystem gebaut. Der Workflow `.github/workflows/tauri-desktop.yml` erstellt Linux-, macOS- und Windows-Artefakte.
- Pro Plattform wird ein einzelnes Paket gebaut: AppImage (Linux), DMG (macOS) und NSIS-EXE (Windows). Die Desktop-Hülle verwendet eine statische leere Frontend-Datei; Vue, Vite und zusätzliche Tauri-Plugins sind nicht erforderlich.
- WebView-Laufzeitabhängigkeiten bleiben betriebssystemspezifisch: WebKitGTK/GTK unter Linux, System-WebKit unter macOS und WebView2 unter Windows. Der Windows-Installer lädt WebView2 bei Bedarf über den Bootstrapper.
