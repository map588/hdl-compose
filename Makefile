.PHONY: build release run gui check test test-ui fmt clippy clean app app-release app-debug app-bundle help

BIN := hdlc
APP_BUNDLE := /Applications/hdl-compose.app
LSREGISTER := /System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister

help:
	@echo "Targets:"
	@echo "  build    - cargo build (debug)"
	@echo "  release  - cargo build --release"
	@echo "  run      - cargo run -- gui (debug)"
	@echo "  gui      - alias for run"
	@echo "  check    - cargo check"
	@echo "  test     - cargo test, with the tests of the GUI (offscreen: no window shows)"
	@echo "  test-ui  - tests of the application on a headless screen (needs headless-computer-use)"
	@echo "  fmt      - cargo fmt"
	@echo "  clippy   - cargo clippy --all-targets -- -D warnings"
	@echo "  app-release - release build installed as $(APP_BUNDLE) (macOS)"
	@echo "  app-debug - debug build installed as $(APP_BUNDLE) (macOS)"
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

# The feature gui-tests adds the tests of the C++ side. They run the window
# with the Qt offscreen platform, so nothing shows on the screen.
test:
	cargo test --features gui-tests

# The tests of the application as a user sees it. They start the real hdlc
# program on a headless screen, so nothing shows on the screen. They need
# the headless-computer-use project, see ui-tests/Cargo.toml.
test-ui:
	cargo test --manifest-path ui-tests/Cargo.toml

fmt:
	cargo fmt

clippy:
	cargo clippy --all-targets -- -D warnings

clean:
	cargo clean

app-release: release app-bundle
	cp target/release/$(BIN) $(APP_BUNDLE)/Contents/MacOS/$(BIN)
	@pkill -x $(BIN) 2>/dev/null || true
	@echo "refreshed $(APP_BUNDLE)"

app-debug: build app-bundle
	cp target/debug/$(BIN) $(APP_BUNDLE)/Contents/MacOS/$(BIN)
	@pkill -x $(BIN) 2>/dev/null || true
	@echo "refreshed $(APP_BUNDLE)"

# Install /Applications/hdl-compose.app (macOS). The bundle makes the app
# launchable from Finder and registers it as the handler for .hdlc files.
app: app-debug

# Create the bundle skeleton when it is missing, install the current
# Info.plist, and tell Launch Services to read it again.
app-bundle:
	mkdir -p $(APP_BUNDLE)/Contents/MacOS
	cp packaging/macos/Info.plist $(APP_BUNDLE)/Contents/Info.plist
	$(LSREGISTER) -f $(APP_BUNDLE)
