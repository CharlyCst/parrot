.PHONY: parrot
parrot:
	cargo build --release

.PHONY: test
test:
	cargo build
	parrot -p test run

