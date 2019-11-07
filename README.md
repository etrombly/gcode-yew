## About

This is a minimal example of the gcode-rs crate with web assembly.

![Screenshot](/screenshot.png?raw=true "Screenshot")

## Usage

### 1) Install `Rust`, `wasm-pack`, and "rollup"

Follow the instructions at https://www.rust-lang.org/tools/install and follow the `installation` link at [`wasm-pack`](https://github.com/rustwasm/wasm-pack).

### 2) Build

Enter `wasm-pack build --target web` from your project's root directory.

### 3) [temporary] Bundle

Enter `rollup ./main.js --format iife --file ./pkg/bundle.js` from your project's root directory.

### 4) [optional] Test Run

Run a webserver from your project's root directory, such as with `python3 -m http.server`, and load http://localhost:8000/ in a browser to run the app.

### 5) Deploy

Access your generated build artifacts, `bundle.js` and `yew_wasm_pack_minimal_bg.wasm`, in ./pkg from your project's root directory.
