build:
	docker run --rm -v ${PWD}:/src -it rustlang/rust:nightly bash -c "cd /src && cargo build -p wld-cli --release"
	mkdir -p bin
	cp target/release/wld-cli bin/wld-cli
build-windows:
	docker run --rm -v ${PWD}:/src -it rustlang/rust:nightly bash -c "cd /src && rustup target add x86_64-pc-windows-gnu && cargo build -p wld-cli --target x86_64-pc-windows-gnu --release"
	mkdir -p bin
	cp target/release/wld-cli bin/wld-cli.exe