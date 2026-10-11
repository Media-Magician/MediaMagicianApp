## General instructions for anywhere
Since xtask system is now present, just be in the root of the project (where this file is located), and run `cargo xtask` in order to launch the project. The console will prompt you to ask about which system you want to target and whether you want dev/build mode.


Local dev command (on linux without CEF):
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

Before docker build clear everything:
```
rm -rf target
rm -rf src-tauri/target
rm -rf src-tauri/gen/android/app/.cxx
rm -rf src-tauri/gen/android/app/build
rm -rf ~/.gradle/caches
```

Building using docker (this needs to be run in the directory above this one, and `MediaMagicianApp` and `ffmpeg` directories should be present):
```
docker build -f docker-building/linux/Dockerfile -t mediamagician-builder .
```
after which we can run `extract.sh` (in this project root) to extract things we need from the container into the `docker-dist` directory. Then copy the MediaMagician (binary) to buildplace.
We can then run the following to build the flatpak based on our `com.mediamagician.app.yml` file:
```
flatpak-builder --force-clean --user --install-deps-from=flathub --repo=repo --install builddir com.mediamagician.app.yml
```
and finally to test/run the app:
```
flatpak run com.mediamagician.app
```
and to bundle into single file:
```
flatpak build-bundle repo MediaMagician.flatpak com.mediamagician.app
```

To install tauri-cli from local cef version, just use
```
cargo install --path ~/Documents/GitHub/tauri_branch_cef/crates/tauri-cli --locked
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

## Handy resources
Flatpak build guide: https://docs.flatpak.org/en/latest/first-build.html
