//! Pure selected-program inspection denial enum.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQuerySelectedProgramInspectionDenial {
    ReadExecution(
        Result<
            crate::domain_computation::WorthQueryProviderSessionDenialKind,
            crate::domain_computation::WorthQueryProviderSessionControlStopKind,
        >,
    ),
    ProgramSupportUnavailable,
    ProgramActivationUnavailable,
    ProgramActivationUnreadable,
    ProgramActivationUnrostered,
}
