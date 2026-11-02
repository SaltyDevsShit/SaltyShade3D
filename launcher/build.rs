//! Gives the launcher (what the desktop icon opens) the SaltyShade 3D icon and file details.

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../overlay/assets/saltyshade.ico")
            .set("ProductName", "SaltyShade 3D")
            .set("FileDescription", "SaltyShade 3D")
            .set("LegalCopyright", "MIT OR Apache-2.0");
        res.compile().expect("embedding the launcher icon");
    }
    println!("cargo:rerun-if-changed=../overlay/assets/saltyshade.ico");
}
