.phony: build
build: clean erlang nodejs deno

.phony: clean
clean:
	rm -rf build

.phony: erlang
erlang:
	@echo test/language on Erlang
	cargo run --quiet -- test --target erlang

.phony: nodejs
nodejs:
	@echo test/language on JavaScript with Node
	cargo run --quiet -- test --target javascript --runtime nodejs

.phony: deno
deno:
	@echo test/language on JavaScript with Deno
	cargo run --quiet -- test --target javascript --runtime deno

.phony: bun
bun:
	@echo test/language on JavaScript with Bun
	cargo run --quiet -- test --target javascript --runtime bun

.phony: nix
nix:
	@echo test/language on Nix
	cargo run --quiet -- test --target nix
