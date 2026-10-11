#![allow(non_snake_case)]

use inquire::Select;
use std::process::Command;

fn pretty_cmd(cmd: &Command) -> String {
    format!(
        "{} {:?}",
        cmd.get_envs()
            .map(|(key, val)| format!("{:?}={:?}", key, val))
            .fold(String::new(), |a, b| a + &b),
        cmd
    )
}

fn main() {
    let mut platformArg = None;
    let mut modeArg = None;
    for arg in std::env::args().skip(1) {
        if let Some(value) = arg.strip_prefix("platform=") {
            platformArg = Some(value.to_owned());
        } else if let Some(value) = arg.strip_prefix("mode=") {
            modeArg = Some(value.to_owned());
        } else {
            eprintln!("unknown argument: {arg}");
            std::process::exit(2);
        }
    }

    let platform = platformArg.unwrap_or_else(|| {
        Select::new("Platform:", vec!["Linux", "Android", "Docker", "Windows"])
            .prompt()
            .unwrap()
            .to_owned()
    });
    if !["Linux", "Android", "Docker", "Windows"].contains(&platform.as_str()) {
        eprintln!("invalid platform: {platform}");
        std::process::exit(2);
    }

    let mode = modeArg.unwrap_or_else(|| {
        Select::new("Mode:", vec!["dev", "build"])
            .prompt()
            .unwrap()
            .to_owned()
    });
    if !["dev", "build"].contains(&mode.as_str()) {
        eprintln!("invalid mode: {mode}");
        std::process::exit(2);
    }

    let (config, mut args): (&str, Vec<&str>) = match platform.as_str() {
        "Linux" => (".cargo/arch.toml", vec!["tauri"]),
        "Docker" => (".cargo/docker.toml", vec!["tauri"]),
        "Android" => (".cargo/android.toml", vec!["tauri", "android"]),
        "Windows" => (".cargo/windows.toml", vec!["tauri"]),
        _ => unreachable!(),
    };
    args.push(mode.as_str());
    if platform == "Android" && mode == "build" {
        // args.push("--apk");
        // args.push("--target");
        args.extend(["--apk", "--target", "aarch64"]);
    }

    std::fs::create_dir_all("src-tauri/.cargo").ok();
    std::fs::copy(config, "src-tauri/.cargo/config.toml").expect("failed to copy config");

    let mut cmd = Command::new("cargo");
    cmd.args(&args);
    cmd.env("LD_LIBRARY_PATH", "/home/paige/.cache/tauri-cef/cef_linux_x86_64/146.0.76");
        
    println!("Executing: {}", pretty_cmd(&cmd));

    let status = cmd.status()
        .expect("failed to run cargo tauri");

    std::process::exit(status.code().unwrap_or(1));
}
