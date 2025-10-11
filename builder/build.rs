use std::path::PathBuf;

fn main() {
    let kernel_path = PathBuf::from(
        std::env::var("CARGO_BIN_FILE_CURE_KERNEL_cure-kernel")
            .expect("CARGO_BIN_FILE_CURE_KERNEL_cure-kernel not set")
    );

    println!("cargo:warning=Kernel path: {:?}", kernel_path);

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let bios_path = out_dir.join("cure-bios.img");
    bootloader::BiosBoot::new(&kernel_path)
        .create_disk_image(&bios_path)
        .expect("Failed to create BIOS disk image");

    let uefi_path = out_dir.join("cure-uefi.img");
    bootloader::UefiBoot::new(&kernel_path)
        .create_disk_image(&uefi_path)
        .expect("Failed to create UEFI disk image");

    println!("cargo:warning=✓ BIOS image: {:?}", bios_path);
    println!("cargo:warning=✓ UEFI image: {:?}", uefi_path);

    println!("cargo:rustc-env=BIOS_IMAGE={}", bios_path.display());
    println!("cargo:rustc-env=UEFI_IMAGE={}", uefi_path.display());
}