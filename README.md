# cxgraph

cxgraph is a complex function graphing tool built around WebGPU available
[on the web](https://cx.trimill.xyz/) or (slightly) for desktop.

## building (web)

install `wasm-pack` through your package manager or with `cargo install wasm-pack`.

```sh
cd cxgraph-web
wasm-pack build --no-typescript --no-pack --target web
```

## documentation
- [language](docs/language.md)
- [web interface](docs/web.md)
