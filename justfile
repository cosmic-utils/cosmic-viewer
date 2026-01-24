# List available commands
default:
    @just --list

# Build debug version
build:
    cargo build

# Build release version
build-release:
    cargo build --release

# Run debug build
run-debug:
    cargo run

# Run release build (default)
run:
    cargo run --release

# Run with picture
run-image image-path:
    cargo run --release -- "{{image-path}}"

# Run with directory
run-dir dir-path:
    cargo run --release -- "{{dir-path}}"

# Check without building
check:
    cargo check

# Format code
fmt:
    cargo fmt

# Check formatting without changes
fmt-check:
    cargo fmt -- --check

# Run clippy
lint:
    cargo clippy -- -D warnings

# Run clippy with all features
lint-all:
    cargo clippy --all-features -- -D warnings

# Format and lint
tidy: fmt lint

# Clean build artifacts
clean:
    cargo clean

# Full rebuild
rebuild: clean build-release

# Install to ~/.cargo/bin
install: build-release
    sudo cp target/release/cupola /usr/local/bin
    sudo cp data/*.desktop /usr/share/applications
    sudo cp data/*.svg /usr/share/icons/hicolor/scalable/apps

# Uninstall
uninstall:
    sudo rm /usr/local/bin/cupola
    sudo rm /usr/share/applications/*Cupola.desktop
    sudo rm /usr/share/icons/hicolor/scalable/apps/*Cupola.svg

# Show binary size
size: build-release
    @ls -lh target/release/cupola | awk '{ print $5 }'

# Generate documentation and open it
docs:
    cargo doc --open

# Run all checks before commit
pre-commit: fmt-check lint check
    @echo "All checks passed!"
