set shell := ["nu.exe", "-c"]

mod:
    cargo build -p dummy --target wasm32-unknown-unknown
    wasm-bindgen --no-typescript --target web --out-name dummy --out-dir assets/mods/ target/wasm32-unknown-unknown/debug/dummy.wasm
    mv assets/mods/dummy_bg.wasm assets/mods/dummy.wasm
    cp -r assets web/

build:
    cargo build -r --target wasm32-unknown-unknown
    wasm-bindgen --no-typescript --out-name wabi --out-dir web --target web target\wasm32-unknown-unknown\release\wabi.wasm
    echo "" | save web/wabi.js --append
    echo "export { getImports }" | save web/wabi.js --append