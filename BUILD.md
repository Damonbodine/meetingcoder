+# Build Instructions
+
+This guide walks through every dependency required to build, package, and validate MeetingCoder on macOS, Windows, and Linux.
+
+## 1. Toolchain Overview
+
+- **Rust**: Install via `rustup` (`curl https://sh.rustup.rs -sSf | sh`). Keep it on the latest stable channel.
+- **Bun**: MeetingCoder uses Bun instead of npm/Yarn. Install with `curl -fsSL https://bun.sh/install | bash` and ensure `$HOME/.bun/bin` is on your PATH.
+- **Tauri CLI**: Invoked via Bun (`bunx tauri`). You do not need to install it globally.
+
+```bash
+# one-time prep
+rustup update stable
+curl -fsSL https://bun.sh/install | bash
+source ~/.bashrc   # or your preferred shell profile
+```
+
+## 2. Platform Prerequisites
+
+### macOS
+
+- Command Line Tools: `xcode-select --install`
+- Codesign identity (Developer ID Application) if you plan to notarize builds
+- `brew install ffmpeg yt-dlp` for the optional import helpers
+
+### Windows
+
+- Visual Studio 2019/2022 with the "Desktop development with C++" workload **or** the standalone Build Tools
+- Windows Developer Mode enabled (Settings → For Developers)
+- PowerShell: `winget install Gyan.FFmpeg` and `winget install yt-dlp.yt-dlp`
+
+### Linux
+
+```bash
+# Ubuntu / Debian
+sudo apt update
+sudo apt install build-essential libasound2-dev pkg-config libssl-dev \
+  libvulkan-dev vulkan-tools glslc libgtk-3-dev libwebkit2gtk-4.1-dev \
+  libayatana-appindicator3-dev librsvg2-dev patchelf ffmpeg
+
+# Fedora / RHEL
+sudo dnf groupinstall "Development Tools"
+sudo dnf install alsa-lib-devel pkgconf openssl-devel vulkan-devel \
+  gtk3-devel webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel ffmpeg
+
+# Arch
+sudo pacman -S base-devel alsa-lib pkgconf openssl vulkan-devel \
+  gtk3 webkit2gtk-4.1 libappindicator-gtk3 librsvg ffmpeg
+
+# yt-dlp (any distro)
+pipx install yt-dlp
+```
+
+## 3. Clone & Install
+
+```bash
+git clone git@github.com:Damonbodine/meetingcoder.git
+cd meetingcoder-app
+bun install
+```
+
+The repo intentionally keeps `node_modules/` out of git history; Bun will recreate it locally.
+
+## 4. Model / VAD Assets
+
+MeetingCoder looks for a Silero VAD model under `src-tauri/resources/models`. Download it once per checkout:
+
+```bash
+mkdir -p src-tauri/resources/models
+curl -L -o src-tauri/resources/models/silero_vad_v4.onnx \
+  https://blob.handy.computer/silero_vad_v4.onnx
+```
+
+Speech models (Whisper + Parakeet) are managed inside the app: launch `bunx tauri dev`, open the **Models** sidebar, and click download on the variants you need. The UI tracks download progress and stores the binaries under `~/Library/Application Support/com.meetingcoder.app/models` (platform-specific).
+
+## 5. External Tools for Imports
+
+MeetingCoder now surfaces a pre-flight checklist for `ffmpeg` and `yt-dlp`. Install them before attempting imports:
+
+| Tool   | macOS                      | Windows                       | Linux (example)       |
+| ------ | -------------------------- | ----------------------------- | --------------------- |
+| ffmpeg | `brew install ffmpeg`      | `winget install Gyan.FFmpeg`  | `sudo apt install ffmpeg` |
+| yt-dlp | `brew install yt-dlp`      | `winget install yt-dlp.yt-dlp`| `pipx install yt-dlp` |
+
+The Transcription view exposes a “Re-check tools” button so you can verify PATH changes without restarting.
+
+## 6. Local Development
+
+```bash
+# Start Vite for the React settings UI
+bun run dev
+
+# Launch Tauri (Rust backend + window)
+bunx tauri dev
+```
+
+Useful directories while debugging:
+
+- App data / settings: `~/Library/Application Support/com.meetingcoder.app` (macOS), `%APPDATA%\com.meetingcoder.app` (Windows), `~/.local/share/com.meetingcoder.app` (Linux)
+- Logs: `~/.meetingcoder/logs` per user session
+
+## 7. Production Builds
+
+Tauri builds are driven via Bun as well:
+
+```bash
+# macOS universal
+bunx tauri build --target universal-apple-darwin
+
+# Windows x64
+bunx tauri build --target x86_64-pc-windows-msvc
+
+# Linux (AppImage + deb)
+bunx tauri build --target x86_64-unknown-linux-gnu
+```
+
+### macOS Codesign & Notarization
+
+1. Set `tauri.conf.json > tauri > macOS > signingIdentity`.
+2. Export these env vars (or configure a notarytool profile):
+   - `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`
+3. After the bundle is produced, notarize: `bunx tauri signer notarize --apple-id ...`
+
+### Windows Signing
+
+Use the Windows SDK `signtool.exe`:
+
+```powershell
+signtool sign /fd SHA256 /a /tr http://timestamp.digicert.com \
+  src-tauri/target/release/bundle/msi/MeetingCoder_x64_en-US.msi
+```
+
+### Updater Metadata
+
`tauri.conf.json` now ships with `bundle.createUpdaterArtifacts` disabled so that local builds do not fail when the private signing key is absent. When you are preparing a release, flip that flag back to `true` (or override it in CI), export `TAURI_SIGNING_PRIVATE_KEY` (and its password if applicable), and re-run `bunx tauri build` so the pipeline emits `latest.json`. Publish that file alongside your installers so the in-app updater can fetch release notes and hash checks.
+
+## 8. Verification Checklist
+
+- `bunx tauri dev` launches without missing assets (Silero downloaded).
+- The Import card shows ffmpeg / yt-dlp status and refuses to run when missing.
+- Recorder mode works out of the box; Advanced Automations remain hidden until explicitly enabled.
+- Offline Mode instantly disables GitHub, YouTube, and Claude operations.
+- `bunx tauri build` succeeds on your target platform(s).
+
+When everything above passes, you can promote the artifacts to a release and update `latest.json` in your distribution channel.
