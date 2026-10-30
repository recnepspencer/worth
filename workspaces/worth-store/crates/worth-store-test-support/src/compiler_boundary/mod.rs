mod declaration;
mod diagnostics;
mod prebuilt_environment;
mod result;
mod suite_reading;
#[cfg(test)]
mod tests;

pub use declaration::{
    ExpectedCompilerDenial, UiCompilerEnvironment, UiFixtureDeclaration, UiFixtureIdentity,
    UiProofSuiteDeclaration,
};
pub use diagnostics::CheckedCompilerDiagnostic;
pub use result::{UiFixtureResult, UiRunFailure, UiRunResult};

pub use suite_reading::{read_ui_fixture_suite, read_ui_proof_suite};
