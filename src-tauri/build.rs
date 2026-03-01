#![allow(non_snake_case)]

fn main() {
    #[cfg(target_os = "android")]
    {
        let ndkHome = std::env::var("ANDROID_NDK_HOME")
            .or_else(|_| std::env::var("ANDROID_NDK_ROOT"))
            .or_else(|_| std::env::var("NDK_HOME"))
            .unwrap_or_else(|_| {
                // Fallback: hardcode your NDK path here
                "/home/paige/Android/Sdk/ndk/29.0.14206865/".to_string()
            });

        println!(
            "cargo:rustc-env=ANDROID_HOME={}",
            "/home/paige/Android/Sdk/"
        );
        println!("cargo:rustc-env=ANDROID_NDK_HOME={}", ndkHome);
        println!("cargo:rustc-env=ANDROID_NDK_HOME={}", ndkHome);
        println!("cargo:rustc-env=NDK_HOME={}", ndkHome);

        let targetArch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();

        let ffmpegInclude = match targetArch.as_str() {
            "aarch64" => "assets/android/arm64-v8a/include",
            // "arm" => "assets/android/armeabi-v7a",
            // "x86" => "assets/android/x86",
            // "x86_64" => "assets/android/x86_64",
            _ => panic!("Unsupported architecture: {}", targetArch),
        };

        let ffmpegLib = match targetArch.as_str() {
            "aarch64" => "assets/android/arm64-v8a/lib",
            // "arm" => "assets/android/armeabi-v7a",
            // "x86" => "assets/android/x86",
            // "x86_64" => "assets/android/x86_64",
            _ => panic!("Unsupported architecture: {}", targetArch),
        };

        // println!("cargo:rustc-env=FFMPEG_PKG_CONFIG=0");
        std::env::set_var("FFMPEG_PKG_CONFIG", "0");
        println!("cargo:rustc-env=FFMPEG_INCLUDE_DIR={}", ffmpegInclude);
        println!("cargo:rustc-env=FFMPEG_LIB_DIR={}", ffmpegLib);

        println!("cargo:rustc-link-search=native={}", ffmpegLib);
        println!("cargo:rustc-link-lib=dylib=avcodec");
        println!("cargo:rustc-link-lib=dylib=avfilter");
        println!("cargo:rustc-link-lib=dylib=avformat");
        println!("cargo:rustc-link-lib=dylib=avutil");
        println!("cargo:rustc-link-lib=dylib=swscale");
        println!("cargo:rustc-link-lib=dylib=swresample");
    }

    // #[cfg(all(target_os = "linux", target_arch = "x86_64", not(target_os = "android")))]
    // {
    //     let ffmpegInclude = "assets/arch/x86_64/include";
    //     let ffmpegLib = "assets/arch/x86_64/lib";

    //     std::env::set_var("FFMPEG_PKG_CONFIG", "0");
    //     println!("cargo:rustc-env=FFMPEG_INCLUDE_DIR={}", ffmpegInclude);
    //     println!("cargo:rustc-env=FFMPEG_LIB_DIR={}", ffmpegLib);

    //     // Statically link ffmpeg libs
    //     println!("cargo:rustc-link-search=native={}", ffmpegLib);
    //     // println!("cargo:rustc-link-lib=dylib=avcodec");
    //     // println!("cargo:rustc-link-lib=dylib=avfilter");
    //     // println!("cargo:rustc-link-lib=dylib=avformat");
    //     // println!("cargo:rustc-link-lib=dylib=avutil");
    //     // println!("cargo:rustc-link-lib=dylib=swscale");
    //     // println!("cargo:rustc-link-lib=dylib=swresample");
    //     println!("cargo:rustc-link-lib=static=avcodec");
    //     println!("cargo:rustc-link-lib=static=avfilter");
    //     println!("cargo:rustc-link-lib=static=avformat");
    //     println!("cargo:rustc-link-lib=static=avutil");
    //     println!("cargo:rustc-link-lib=static=swscale");
    //     println!("cargo:rustc-link-lib=static=swresample");

    //     // Extra required libs
    //     println!("cargo:rustc-link-lib=dylib=X11");
    //     println!("cargo:rustc-link-lib=dylib=va");
    //     println!("cargo:rustc-link-lib=dylib=va-drm");
    //     println!("cargo:rustc-link-lib=dylib=va-x11");
    //     println!("cargo:rustc-link-lib=dylib=drm");
    //     println!("cargo:rustc-link-lib=dylib=vdpau");
    //     println!("cargo:rustc-link-lib=dylib=bz2");
    //     println!("cargo:rustc-link-lib=dylib=lzma");
    //     println!("cargo:rustc-link-lib=dylib=x264");
    // }

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        let ffmpegInclude = "C:/E/Programming/GitHub/MediaMagicianApp/src-tauri/assets/windows/x86_64/include";
        let ffmpegLib = "C:/E/Programming/GitHub/MediaMagicianApp/src-tauri/assets/windows/x86_64/lib";

        eprintln!("FFmpeg lib path: {}", ffmpegLib);
        eprintln!("Files in lib dir: {:?}", std::fs::read_dir(&ffmpegLib).map(|d| d.map(|e| e.unwrap().file_name()).collect::<Vec<_>>()));

        std::env::set_var("FFMPEG_PKG_CONFIG", "0");
        println!("cargo:rustc-env=FFMPEG_INCLUDE_DIR={}", ffmpegInclude);
        println!("cargo:rustc-env=FFMPEG_LIB_DIR={}", ffmpegLib);

        println!("cargo:rustc-link-search=native={}", ffmpegLib);
        println!("cargo:rustc-link-lib=dylib=avcodec");
        println!("cargo:rustc-link-lib=dylib=avfilter");
        println!("cargo:rustc-link-lib=dylib=avformat");
        println!("cargo:rustc-link-lib=dylib=avutil");
        println!("cargo:rustc-link-lib=dylib=swscale");
        println!("cargo:rustc-link-lib=dylib=swresample");
    }

    tauri_build::build();
}
