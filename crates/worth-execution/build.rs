//! The OS-thread capability of the selected standard-library target.
fn main() {
    println!("cargo::rustc-check-cfg=cfg(worth_execution_os_threads)");
    let target = std::env::var("TARGET").expect("Cargo supplies the target");
    let families =
        std::env::var("CARGO_CFG_TARGET_FAMILY").expect("Cargo supplies target families");
    // Ordinary wasm standard libraries have no OS-thread implementation.
    // WASI's explicitly threaded target does; atomics alone are not threads.
    let has_os_threads =
        !families.split(',').any(|family| family == "wasm") || target == "wasm32-wasip1-threads";
    if has_os_threads {
        println!("cargo::rustc-cfg=worth_execution_os_threads");
    }
}
