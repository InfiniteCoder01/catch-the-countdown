debug:
	cargo build --release --target=wasm32-unknown-emscripten
	mkdir -p ./html
	cp ./target/wasm32-unknown-emscripten/release/deps/*.data ./html/
	cp ./target/wasm32-unknown-emscripten/release/*.wasm ./html/
	cp ./target/wasm32-unknown-emscripten/release/*.d ./html/
	cp ./target/wasm32-unknown-emscripten/release/*.js ./html/
