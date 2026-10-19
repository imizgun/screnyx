fn main() {
    check_xtensa_linker_available();
    // Must be the last linker script passed (esp-hal places memory.x/esp32s3.x
    // here too, via its own build script's OUT_DIR copy) — otherwise interrupt
    // vector symbols like WIFI_NMI/BT_MAC stay undefined at link time.
    println!("cargo:rustc-link-arg=-Tlinkall.x");
}

#[cfg(unix)]
fn check_xtensa_linker_available() {
    println!("cargo:rerun-if-env-changed=PATH");

    let target = std::env::var("TARGET").unwrap_or_default();
    let linker = target
        .strip_prefix("xtensa-")
        .and_then(|t| t.strip_suffix("-none-elf"))
        .map(|chip| format!("xtensa-{chip}-elf-gcc"))
        .unwrap_or_else(|| "xtensa-esp32-elf-gcc".to_string());

    if std::process::Command::new(&linker)
        .arg("--version")
        .output()
        .is_ok()
    {
        return;
    }

    let export_file = std::env::var("HOME")
        .map(|home| format!("{home}/export-esp.sh"))
        .unwrap_or_else(|_| "$HOME/export-esp.sh".to_string());

    panic!(
        "Xtensa linker `{linker}` was not found in PATH.\n\
         Source espup's environment export file first: `. {export_file}`"
    );
}

#[cfg(not(unix))]
fn check_xtensa_linker_available() {}
