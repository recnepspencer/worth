use worth_runtime_bridge::facade::BridgeDiagnosticsFacade;

fn main() {}

fn writeback_diagnostics_lookup_requires_typed_identity(diagnostics: &BridgeDiagnosticsFacade) {
    let _ = diagnostics.writeback_replay_record_for_identity(sealed_authority_placeholder::<&str>());
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
