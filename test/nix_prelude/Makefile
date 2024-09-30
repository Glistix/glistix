.PHONY: test
test:
	@echo test/nix_prelude
	@cp ../../compiler-core/templates/prelude.nix prelude.nix
	@nix-instantiate --show-trace --eval main.nix
	@rm prelude.nix
