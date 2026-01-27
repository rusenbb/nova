# Contributing to Nova

Thank you for your interest in contributing to Nova! This document provides guidelines and instructions for contributing.

## Development Setup

### Prerequisites

- **Rust** (latest stable) - [Install via rustup](https://rustup.rs/)
- **Node.js** (v18+) - For TypeScript SDK development
- **Xcode** (macOS only) - For Swift frontend development

### Platform-Specific Dependencies

#### Linux (Ubuntu/Debian)
```bash
sudo apt install libxdo-dev libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libgtk-3-dev
```

#### Linux (Fedora)
```bash
sudo dnf install libxdo-devel libxcb-devel gtk3-devel
```

#### Linux (Arch)
```bash
sudo pacman -S libxdo libxcb gtk3
```

#### macOS
```bash
# Install Xcode command line tools
xcode-select --install
```

### Building

#### Library (all platforms)
```bash
cargo build
cargo test
```

#### GTK Frontend (Linux)
```bash
cargo build --features gtk-ui
cargo run --features gtk-ui
```

#### macOS Frontend
```bash
# Open in Xcode
open frontends/macos/Nova.xcodeproj

# Or build from command line
cd frontends/macos
xcodebuild -scheme Nova -configuration Debug
```

#### TypeScript SDK
```bash
cd packages/nova-sdk
npm install
npm run build
```

## Project Structure

```
nova/
├── src/                    # Rust library and GTK binary
│   ├── lib.rs              # Library entry point
│   ├── main.rs             # GTK binary entry point
│   ├── core/               # Core search engine
│   ├── extensions/         # Extension runtime (Deno)
│   ├── ffi.rs              # FFI exports for native frontends
│   └── platform/           # Platform abstraction layer
├── frontends/
│   └── macos/              # Swift/AppKit frontend
├── packages/
│   └── nova-sdk/           # TypeScript SDK for extensions
└── sample-extensions/      # Example extensions
```

## Development Workflow

### Running Tests
```bash
# All tests
cargo test

# Specific module
cargo test --lib extensions

# With output
cargo test -- --nocapture
```

### Linting
```bash
# Format code
cargo fmt

# Run clippy
cargo clippy

# GTK-specific clippy
cargo clippy --features gtk-ui
```

### Benchmarks
```bash
cargo bench --bench performance
```

## Making Changes

### Code Style

- Follow Rust conventions and use `cargo fmt`
- Use `thiserror` for error types
- Avoid `.unwrap()` in production code paths
- Prefer `&str` over `String` in function parameters when possible
- Document public APIs with doc comments

### Commit Messages

Write clear, descriptive commit messages:
- Use present tense ("Add feature" not "Added feature")
- Reference issues when applicable ("Fix #123")
- Keep the first line under 72 characters

### Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Make your changes
4. Ensure tests pass (`cargo test`)
5. Ensure linting passes (`cargo fmt && cargo clippy`)
6. Push and open a PR

### PR Guidelines

- Keep PRs focused on a single change
- Update documentation if needed
- Add tests for new functionality
- Ensure CI passes before requesting review

## Extension Development

See [Extension Development Guide](docs/extension-development.md) for details on building Nova extensions.

### Quick Start
```bash
# Create a new extension
cargo run --example nova_cli -- create extension my-extension

# Development mode with hot reload
cargo run --example nova_cli -- dev my-extension

# Build for distribution
cargo run --example nova_cli -- build my-extension
```

## Getting Help

- Open an issue for bugs or feature requests
- Check existing issues before creating new ones
- Provide reproduction steps for bugs

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
