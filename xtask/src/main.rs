use inquire::Select;
use std::process::Command;

fn main() {
    let platform = Select::new("Platform:", vec!["Linux", "Android", "Docker", "Windows"])
        .prompt()
        .unwrap();

    let mode = Select::new("Mode:", vec!["dev", "build"])
        .prompt()
        .unwrap();

    let (config, mut args): (&str, Vec<&str>) = match platform {
        "Linux" => (".cargo/arch.toml", vec!["tauri"]),
        "Docker" => (".cargo/docker.toml", vec!["tauri"]),
        "Android" => (".cargo/android.toml", vec!["tauri", "android"]),
        "Windows" => (".cargo/windows.toml", vec!["tauri"]),
        _ => unreachable!(),
    };
    args.push(mode);

    std::fs::create_dir_all("src-tauri/.cargo").ok();
    std::fs::copy(config, "src-tauri/.cargo/config.toml").expect("failed to copy config");

    let status = Command::new("cargo")
        // .arg("--config")
        // .arg(config)
        .args(&args)
        .status()
        .expect("failed to run cargo tauri");

    std::process::exit(status.code().unwrap_or(1));
}
