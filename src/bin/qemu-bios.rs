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

    let exit_status = qemu.status().unwrap();
    process::exit(exit_status.code().unwrap_or(-1));
}