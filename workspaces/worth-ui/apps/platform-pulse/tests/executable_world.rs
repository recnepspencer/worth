//! Which posture this build holds (milestone-3.10.3 §D5) is decided by one
//! alias, `worth_ui_certified_executable`, emitted by `build.rs` for Windows
//! and for the Linux X11 certification profile. Outside that alias every
//! scenario module but `failure_teardown` compiles and none executes, so the
//! scenario types, their methods and the re-exports the courtroom consumes
//! have no consumer here. `failure_teardown` is the exception because its
//! artifact writer names types only the certified world constructs.
//! `cargo check` is green without this allowance; it exists for
//! `cargo clippy --all-targets -- -D warnings`, which turns the resulting
//! ~100 dead-code and unused-import warnings fatal. Removing it is a lint
//! decision, not a build break.
#![cfg_attr(not(worth_ui_certified_executable), allow(dead_code, unused_imports))]

#[path = "executable_world/adjudication/mod.rs"]
mod adjudication;
#[path = "executable_world/courtroom/mod.rs"]
mod courtroom;
#[path = "executable_world/external_observation/mod.rs"]
mod external_observation;
#[cfg(worth_ui_certified_executable)]
#[path = "executable_world/failure_teardown/mod.rs"]
mod failure_teardown;
#[path = "executable_world/installation/mod.rs"]
mod installation;
#[path = "executable_world/native_platform/mod.rs"]
mod native_platform;
#[path = "executable_world/product_process/mod.rs"]
mod product_process;
#[path = "executable_world/source_delta/mod.rs"]
mod source_delta;

/// `build.rs` derives the alias from the build script's view of the target
/// and the two Linux cfg axes; this is the same derivation spelled in source.
/// A lane that drops both flags keeps this green (both sides fall together),
/// which is why the lane also asserts its executed-test count.
#[test]
fn the_certified_executable_alias_agrees_with_the_source_level_derivation() {
    let derived = cfg!(target_os = "windows")
        || (cfg!(target_os = "linux")
            && cfg!(worth_ui_windowing = "x11")
            && cfg!(worth_ui_adapter = "software"));
    assert_eq!(cfg!(worth_ui_certified_executable), derived);
}

/// The launcher's desktop lease and process containment exist exactly where
/// the product process runs; `build.rs` names that once and this pins it.
#[test]
fn the_product_executable_alias_agrees_with_the_source_level_derivation() {
    let derived = cfg!(any(target_os = "windows", target_os = "linux"));
    assert_eq!(cfg!(worth_ui_product_executable), derived);
}

#[test]
fn the_platform_posture_is_the_one_this_build_declares() {
    let expected = if cfg!(worth_ui_certified_executable) {
        "certified_executable"
    } else if cfg!(worth_ui_product_executable) {
        "not_yet_certified_executable"
    } else {
        "compile_only"
    };
    assert_eq!(native_platform::current_platform_posture().name(), expected);
}
