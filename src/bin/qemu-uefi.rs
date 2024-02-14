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
    qemu.arg(format!("format=raw,file={}", env!("UEFI_IMAGE")));
    qemu.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());

    qemu.arg("-serial").arg("stdio");

    match env::consts::OS {
        "windows" => {
            qemu.arg("-accel").arg("whpx,kernel-irqchip=off");
        }, "linux" => {
            qemu.arg("-accel").arg("kvm");
        }, "macos" => {
            qemu.arg("-accel").arg("hvf");
        }, _ => {}
    }

    qemu.arg("-device").arg(format!("VGA,{}", env::var("VGA_OPTIONS").unwrap()));
    qemu.arg("-cpu").arg("qemu64");

    let exit_status = qemu.status().unwrap();
    process::exit(exit_status.code().unwrap_or(-1));
}