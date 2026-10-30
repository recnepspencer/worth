use worth_runtime_bridge::facade::RuntimeBridge;

fn main() {}

fn preview_replay_requires_typed_session_identity(runtime: &RuntimeBridge) {
    let _ = runtime.replay_preview_bundle(sealed_authority_placeholder::<&str>());
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
