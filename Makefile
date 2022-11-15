build:
	cargo build --release
	ln -s ./target/release/pire2pire pire2pire
	ln -s ./target/release/pire2pire demo/pire2pire

demo: build
	./demo/demo.sh

.PHONY: demo
