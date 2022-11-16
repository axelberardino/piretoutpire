build:
	cargo build --release
	ln -sf ./target/release/pire2pire pire2pire
	ln -sf ./target/release/pire2pire demo/pire2pire

demo: build
	./demo/demo.sh

.PHONY: demo
