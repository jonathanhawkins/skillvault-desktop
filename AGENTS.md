# Repository Guidelines

## Project Structure & Module Organization
`src/` contains the Vite frontend written in vanilla TypeScript. Keep route-level screens in `src/views/`, shared UI in `src/components/`, app state and API wrappers in `src/lib/`, and CSS tokens/layout rules in `src/styles/`. `src-tauri/` is the Rust backend for Tauri: IPC commands live in `src-tauri/src/commands.rs`, API code in `src-tauri/src/api/`, installers in `src-tauri/src/installer/`, and local skill/plugin scanners in `src-tauri/src/scanner/`. Static app icons live under `src-tauri/icons/`; planning docs live in `docs/`.

## Build, Test, and Development Commands
Run `npm install` once to install the frontend and Tauri CLI dependencies. Use `npm run dev` for the frontend only, or `npm run tauri dev` for the full desktop app with hot reload. Build the web bundle with `npm run build`. Preview the built frontend with `npm run preview`. Run backend tests with `cargo test --manifest-path src-tauri/Cargo.toml`, and type-check the frontend with `npx tsc --noEmit`.

## Coding Style & Naming Conventions
Frontend code uses ES modules, 2-space indentation, `camelCase` for functions, and kebab-case filenames such as `skill-detail.ts`. This project does not use a frontend framework; extend the existing direct DOM-rendering patterns instead of introducing React or similar tools. Reuse the tokenized dark-theme styles in `src/styles/`. Rust code follows standard `rustfmt` formatting, `snake_case` for functions/modules, and small focused modules under `src-tauri/src/`.

## Testing Guidelines
Rust tests are colocated as `tests.rs` files or `#[cfg(test)]` modules, for example under `src-tauri/src/scanner/` and `src-tauri/src/api/`. Add regression tests when changing scanner logic, package validation, or API behavior. Before opening a PR, run `cargo test --manifest-path src-tauri/Cargo.toml` and `npx tsc --noEmit`.

## Commit & Pull Request Guidelines
Recent history uses short, imperative commit subjects such as `Handle 409 Conflict when package already exists` and `Add 9 validate_name tests for publish/install name security`. Follow that pattern: one clear change per commit, no prefixes unless they add value. PRs should explain what changed, why it changed, and how it was verified. Link the related issue when applicable, and include screenshots or short recordings for UI changes in `src/`.

## Security & Configuration Tips
Do not hardcode API tokens or local machine paths. Auth is stored through the app’s local config flow; keep secrets out of source, fixtures, and screenshots. When editing install, publish, or scanner code, preserve the existing path-validation and name-validation checks in `src-tauri/src/commands.rs`.
