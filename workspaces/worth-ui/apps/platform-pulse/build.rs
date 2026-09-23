const WINDOWS_MSVC_STACK_RESERVE_BYTES: u64 = 8 * 1024 * 1024;

/// The build whose native courtroom executes against a real observer: Windows,
/// or the Linux X11 certification profile (`worth_ui_windowing = "x11"` with
/// `worth_ui_adapter = "software"`, the only qualified profile that presents
/// under Xvfb). Every other Linux build runs the product with no executing
/// observer lane. `tests/executable_world.rs` compares the emitted alias with
/// the same derivation spelled in `cfg!`, so this function and the source
/// cannot disagree silently.
fn certified_executable() -> bool {
    match std::env::var("CARGO_CFG_TARGET_OS").as_deref() {
        Ok("windows") => true,
        Ok("linux") => {
            std::env::var("CARGO_CFG_WORTH_UI_WINDOWING").as_deref() == Ok("x11")
                && std::env::var("CARGO_CFG_WORTH_UI_ADAPTER").as_deref() == Ok("software")
        }
        _ => false,
    }
}

/// The build whose product process runs on the host at all: the launcher's
/// desktop lease and process containment exist only here. Compile-only
/// targets (milestone-3.10.3 §D5) compile every scenario and launch nothing.
/// Paired with its own `cfg!` cross-check in `tests/executable_world.rs`.
fn product_executable() -> bool {
    matches!(
        std::env::var("CARGO_CFG_TARGET_OS").as_deref(),
        Ok("windows") | Ok("linux")
    )
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-check-cfg=cfg(worth_ui_certified_executable)");
    println!("cargo:rustc-check-cfg=cfg(worth_ui_product_executable)");
    if certified_executable() {
        println!("cargo:rustc-cfg=worth_ui_certified_executable");
    }
    if product_executable() {
        println!("cargo:rustc-cfg=worth_ui_product_executable");
    }
    println!(
        "cargo:rustc-env=WORTH_UI_PLATFORM_PULSE_STACK_RESERVE_BYTES={WINDOWS_MSVC_STACK_RESERVE_BYTES}"
    );
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
    {
        println!(
            "cargo:rustc-link-arg-bin=worth-ui-platform-pulse=/STACK:{WINDOWS_MSVC_STACK_RESERVE_BYTES}"
        );
    }
}
