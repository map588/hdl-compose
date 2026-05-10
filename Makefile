.PHONY: build release run gui check test fmt clippy clean app help

BIN := hdl-compose
APP_BUNDLE := /Applications/hdl-compose.app

help:
	@echo "Targets:"
	@echo "  build    - cargo build (debug)"
	@echo "  release  - cargo build --release"
	@echo "  run      - cargo run -- gui (debug)"
	@echo "  gui      - alias for run"
	@echo "  check    - cargo check"
	@echo "  test     - cargo test"
	@echo "  fmt      - cargo fmt"
	@echo "  clippy   - cargo clippy --all-targets -- -D warnings"
	@echo "  app-release - cargo build --release && cp target/release/$(BIN) $(APP_BUNDLE)/Contents/MacOS/$(BIN)"
	@echo "  app-debug - cargo build && cp target/debug/$(BIN) $(APP_BUNDLE)/Contents/MacOS/$(BIN)"
	@echo "  clean    - cargo clean"
	@echo ""
	@echo "Requires Qt 6 (homebrew: brew install qt). qmake found: $$(command -v qmake6 || command -v qmake)"

build:
	cargo build

release:
	cargo build --release

run gui:
	cargo run -- gui

check:
	cargo check

test:
	cargo test

fmt:
	cargo fmt

clippy:
	cargo clippy --all-targets -- -D warnings

clean:
	cargo clean

app-release: app
	cp target/release/$(BIN) $(APP_BUNDLE)/Contents/MacOS/$(BIN)
	@pkill -x $(BIN) 2>/dev/null || true
	@echo "refreshed $(APP_BUNDLE)"

app-debug: app
	cp target/debug/$(BIN) $(APP_BUNDLE)/Contents/MacOS/$(BIN)
	@pkill -x $(BIN) 2>/dev/null || true
	@echo "refreshed $(APP_BUNDLE)"

# Refresh /Applications/hdl-compose.app wrapper used for computer-use GUI testing.
# The .app must already exist (created once manually with Info.plist).
app: build
	@if [ ! -d "$(APP_BUNDLE)" ]; then \
		echo "error: $(APP_BUNDLE) does not exist."; \
		echo "Create the bundle once with Info.plist, then re-run make app."; \
		exit 1; \
	fi
