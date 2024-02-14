use std::{
    env,
    process::{self, Command},
};

fn main() {
    let mut qemu = Command::new(
        format!("{}/tools/qemu/qemu-system-x86_64",
        env::var("CARGO_MANIFEST_DIR").unwrap())
    );

    qemu.arg("-drive");
    qemu.arg(format!("format=raw,file={}", env!("BIOS_IMAGE")));

    qemu.arg("-serial").arg("stdio");
    qemu.arg("-S");

    match env::consts::OS {
        "windows" => {
            qemu.arg("-accel").arg("whpx");
        }, "linux" => {
            qemu.arg("-accel").arg("kvm");
        }, "macos" => {
            qemu.arg("-accel").arg("hvf");
        }, _ => {}
    }

    qemu.arg("-device").arg(format!("VGA,{}", env::var("VGA_OPTIONS").unwrap()));

    let exit_status = qemu.status().unwrap();
    process::exit(exit_status.code().unwrap_or(-1));
}