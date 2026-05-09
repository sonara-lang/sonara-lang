.PHONY: build build-linux build-windows test clean

build: build-linux build-windows

build-linux:
	cargo build --release
	cp target/release/sonara bin/sonara

build-windows:
	cargo build --release --target x86_64-pc-windows-gnu
	cp target/x86_64-pc-windows-gnu/release/sonara.exe bin/sonara.exe

test:
	cargo test

clean:
	cargo clean
	rm -f bin/sonara bin/sonara.exe
	rm -rf .sonara
