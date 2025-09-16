Copy the file `rust-analyzer-settings.json` into your local workspace at `.vscode/settings.json` to enable recommended rust-analyzer settings for this repository.

Why:
- Enables proc macro support and loads `outDir` from cargo check so the Rust analyzer can correctly resolve macro-generated code (e.g. clap derive).
- Sets `checkOnSave` to run `clippy` for faster feedback.

How:
1. Create `.vscode` in the repository root if it doesn't exist.
2. Copy `.github/editor-setup/rust-analyzer-settings.json` to `.vscode/settings.json`.
3. In VS Code, open Command Palette (Ctrl+Shift+P) and run "Rust Analyzer: Restart Server" or "Developer: Reload Window".

Note: We intentionally keep these files outside `.vscode` to avoid committing user-specific editor settings. Each contributor can copy them locally if they prefer consistent behavior.
