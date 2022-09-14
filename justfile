set shell := ["nu.exe", "-c"]

mod:
    cargo build --profile mod-release -p wabi_mod_impl --target wasm32-unknown-unknown
    wasm-bindgen --no-typescript --target web --out-name impl --out-dir assets/mods/ target/wasm32-unknown-unknown/mod-release/wabi_mod_impl.wasm
    mv assets/mods/impl_bg.wasm assets/mods/impl.wasm
    cp -r assets web/

mod-debug:
    cargo build -p impl --target wasm32-unknown-unknown
    wasm-bindgen --no-typescript --target web --out-name impl --out-dir assets/mods/ target/wasm32-unknown-unknown/debug/impl.wasm
    mv assets/mods/impl_bg.wasm assets/mods/impl.wasm
    cp -r assets web/

build:
    cargo build -r --target wasm32-unknown-unknown
    wasm-bindgen --no-typescript --out-name wabi --out-dir web --target web target\wasm32-unknown-unknown\release\wabi.wasm
    echo "" | save web/wabi.js --append
    echo "export { getImports }" | save web/wabi.js --append