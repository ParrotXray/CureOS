use std::path::PathBuf;

fn main() {
    // 從環境變數獲取內核路徑（運行時）
    let kernel_path = PathBuf::from(
        std::env::var("CARGO_BIN_FILE_CURE_KERNEL_cure-kernel")
            .expect("CARGO_BIN_FILE_CURE_KERNEL_cure-kernel not set")
    );

    println!("cargo:warning=Kernel path: {:?}", kernel_path);

    // 創建輸出目錄
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    // 創建 BIOS 磁盤映像
    let bios_path = out_dir.join("cure-bios.img");
    bootloader::BiosBoot::new(&kernel_path)
        .create_disk_image(&bios_path)
        .expect("Failed to create BIOS disk image");

    // 創建 UEFI 磁盤映像
    let uefi_path = out_dir.join("cure-uefi.img");
    bootloader::UefiBoot::new(&kernel_path)
        .create_disk_image(&uefi_path)
        .expect("Failed to create UEFI disk image");

    // 輸出路徑信息
    println!("cargo:warning=✓ BIOS image: {:?}", bios_path);
    println!("cargo:warning=✓ UEFI image: {:?}", uefi_path);

    // 讓 cargo 知道磁盤映像的位置
    println!("cargo:rustc-env=BIOS_IMAGE={}", bios_path.display());
    println!("cargo:rustc-env=UEFI_IMAGE={}", uefi_path.display());
}