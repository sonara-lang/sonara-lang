.PHONY: build test clean

build:
	cargo build --release
	cp target/release/sonara bin/sonara

test:
	cargo test

clean:
	cargo clean
	rm -f bin/sonara
	rm -rf .sonara
