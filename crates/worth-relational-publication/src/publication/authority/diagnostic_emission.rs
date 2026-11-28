use contracts::diagnostics::data::profiles::RelationalDiagnosticsProfile;
use worth_relational_contracts::internal_port as contracts;

#[derive(Clone, Copy)]
pub struct PreparationDiagnostics<'a> {
    pub profile: &'a RelationalDiagnosticsProfile,
    pub instrumentation:
        &'a worth_relational_branch::internal_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation,
}
