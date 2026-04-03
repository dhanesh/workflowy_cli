.PHONY: build test install update-api-docs clean

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

clean:
	cargo clean
