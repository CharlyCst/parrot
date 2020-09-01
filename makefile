.PHONY: test
test:
	cargo build
	parrot -p test run

