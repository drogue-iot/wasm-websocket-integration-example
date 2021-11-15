# wasm-websocket-integration-example

## Prerequisites

* Cargo
* Npm

## Build dependencies

```
cargo install trunk
rustup target add wasm32-unknown-unknown
```

## Running

```
npm install
trunk serve --port 8000
```

## Updating github pages

```
trunk build -d docs --public-url /wasm-websocket-integration-example
git add docs
git commit -m 'Update GH pages'
git push
```

