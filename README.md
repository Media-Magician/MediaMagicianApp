Local dev command:
```sh
WEBKIT_DISABLE_DMABUF_RENDERER=1 cargo tauri dev
```

## Android
Android dev:
First ensure src-tauri/.cargo/config.toml has uncommented env, then
```sh
cargo tauri android dev
```
## Linux
When building (on linux), it is likely you will need NO_STRIP:
```
NO_STRIP=true cargo tauri build
```

## Windows
When building (on windows), you might need to set a clang var (batch example):
```
set LIBCLANG_PATH=C:/msys64/mingw64/bin
```
This needs setup by installing msys64 and using the following before tauri building:
```
pacman -S mingw-w64-x86_64-clang mingw-w64-x86_64-llvm
```
