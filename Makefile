.PHONY: build test install update-api-docs clean lint audit

build:
	cargo build --release

test:
	cargo test

install: build
	cp target/release/workflowy-cli /usr/local/bin/

# Satisfies: T4, O2 — Self-updating API reference via Jina Reader
update-api-docs:
	curl -sL 'https://r.jina.ai/https://beta.workflowy.com/api-reference/' -o workflowy_api.md
	@echo "Updated workflowy_api.md"

# Satisfies: O2 — local quality gates matching CI
lint:
	cargo fmt --check
	cargo clippy -- -D warnings

# Satisfies: RT-7, S1 — dependency vulnerability scan (advisory, not blocking)
audit:
	cargo audit

clean:
	cargo clean
