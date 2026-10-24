//! Windows only: embed the app icon and version info (VERSIONINFO) into `yedit.exe`.
//!
//! On every other target this does nothing. A missing resource compiler is a warning, so a
//! cross-compile from macOS or Linux still links, unless `YEDIT_REQUIRE_WINRES=1` (set by the
//! release workflow) turns it into an error.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../assets/app-icon/yedit.ico");
    println!("cargo:rerun-if-env-changed=YEDIT_REQUIRE_WINRES");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    // Printing to PDF (winprint's `PdfCreateRenderer`) imports Windows.Data.Pdf.dll, which
    // stripped Windows images (NTLite, some N/LTSC builds) lack. A load-time import stops the exe
    // from starting at all ("windows.data.pdf.dll was not found", #5884), so load it on first use;
    // the print command checks for it first and reports an error instead.
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        println!("cargo:rustc-link-arg-bins=/DELAYLOAD:windows.data.pdf.dll");
        println!("cargo:rustc-link-arg-bins=delayimp.lib");
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon("../../assets/app-icon/yedit.ico")
        .set("ProductName", "YEDIT")
        .set("FileDescription", "YEDIT image editor")
        .set("CompanyName", "FESI Labs")
        .set("LegalCopyright", "Copyright (c) FESI Labs and the PhotoCraft authors. MIT OR Apache-2.0.")
        .set("OriginalFilename", "yedit.exe")
        .set("InternalName", "yedit");
    if let Err(e) = res.compile() {
        if std::env::var_os("YEDIT_REQUIRE_WINRES").is_some() {
            panic!("embedding Windows resources failed: {e}");
        }
        println!("cargo:warning=yedit.exe built without icon/version resources: {e}");
    }
}
