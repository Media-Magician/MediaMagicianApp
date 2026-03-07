## When publishing, go through these steps:

### PKGBUILD
- update version numbers
- build tauri project (webkitgtk)
- upload to github release with regular naming (binary as MediaMagician-x86_64, PKGBUILD as PKGBUILD)

- build tauri project (cef)
- upload to github release with cef naming (binary as MediaMagician_CEF-x86_64, PKGBUILD as PKGBUILD_CEF)

### Flatpak
- update version numbers
- build tauri project (webkitgtk)
- copy to buildplace and build with output name MediaMagician_version.flatpak
- upload to github release with regular naming (MediaMagician_version.flatpak)

- build tauri project (cef)
- copy to buildplace and build with output name MediaMagician_CEF_version.flatpak
- upload to github release with cef naming (MediaMagician_CEF_version.flatpak)

### Windows
- update version numbers
- build tauri project (regular)
- upload msi to github release with naming MediaMagician-x86_64_version.msi

### Android
- update version numbers
- build tauri project (regular)
- upload apk to github release with naming MediaMagician_version.apk
