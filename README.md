Local dev command:
```sh
WEBKIT_DISABLE_DMABUF_RENDERER=1 cargo tauri dev
```

Android dev:
First ensure src-tauri/.cargo/config.toml has uncommented env, then
```sh
cargo tauri android dev
```

When building (on linux), it is likely you will need NO_STRIP:
```
NO_STRIP=true cargo tauri build
```
