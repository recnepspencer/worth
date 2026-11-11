use worth_runtime_bridge::facade::BridgeDiagnosticsFacade;

fn main() {}

fn writeback_diagnostics_lookup_requires_retention_reference(diagnostics: &BridgeDiagnosticsFacade) {
    let _ = diagnostics.writeback_replay_record_for_reference(sealed_authority_placeholder::<&str>());
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
