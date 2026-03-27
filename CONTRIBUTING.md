# Contributing to SkillVault Desktop

Thanks for your interest in contributing! This guide will help you get set up.

## Development Setup

### Prerequisites

- macOS 10.15+
- Node.js 18+
- Rust 1.70+ (install via [rustup.rs](https://rustup.rs/))

### Getting Started

```bash
git clone https://github.com/boneio/skillvault-desktop.git
cd skillvault-desktop
npm install
npm run tauri dev
```

This starts the app in development mode with hot reload for the frontend.

## Code Style

- **No frontend frameworks.** The UI is vanilla TypeScript with direct DOM manipulation. No React, Vue, or Svelte.
- **Geist fonts.** All text uses Geist (sans-serif) and Geist Mono (monospace) to match the SkillVault web design.
- **Dark theme only.** Follow the existing CSS token system in `src/styles/`.
- **Rust formatting.** Run `cargo fmt` before committing Rust code.
- **TypeScript.** Follow the existing patterns — strict mode, no `any` types where avoidable.

## Running Tests

```bash
# Rust tests
cargo test --manifest-path src-tauri/Cargo.toml

# Type-check the frontend
npx tsc --noEmit
```

## Pull Request Process

1. **Fork** the repository and create a branch from `main`.
2. **Name your branch** descriptively: `feat/skill-search`, `fix/scanner-crash`, `docs/readme-update`.
3. **Keep PRs focused.** One feature or fix per PR.
4. **Run tests** and make sure they pass before submitting.
5. **Write a clear description** explaining what changed and why.
6. A maintainer will review your PR and may request changes.

## Reporting Issues

Open a [GitHub issue](https://github.com/boneio/skillvault-desktop/issues) with:

- Steps to reproduce
- Expected vs actual behavior
- macOS version and app version

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
