use super::super::output::ApplicationMutationOutputPosture;

mod sealed {
    pub trait Sealed {}
}

/// Output action: the role names a record the mutation keeps, neither
/// creating nor retiring it.
pub struct WorthQueryPreserveOutput;
/// Output action: the role names a record the mutation creates.
pub struct WorthQueryCreateOutput;
/// Output action: the role names a record the mutation retires.
pub struct WorthQueryRetireOutput;

/// The action an output role performs on its record: implemented only by
/// [`WorthQueryPreserveOutput`], [`WorthQueryCreateOutput`] and
/// [`WorthQueryRetireOutput`], and sealed against other implementations.
pub trait WorthQueryApplicationOutputAction: sealed::Sealed + 'static {
    const POSTURE: ApplicationMutationOutputPosture;
}

impl sealed::Sealed for WorthQueryPreserveOutput {}
impl WorthQueryApplicationOutputAction for WorthQueryPreserveOutput {
    const POSTURE: ApplicationMutationOutputPosture = ApplicationMutationOutputPosture::Preserve;
}

impl sealed::Sealed for WorthQueryCreateOutput {}
impl WorthQueryApplicationOutputAction for WorthQueryCreateOutput {
    const POSTURE: ApplicationMutationOutputPosture = ApplicationMutationOutputPosture::Create;
}

impl sealed::Sealed for WorthQueryRetireOutput {}
impl WorthQueryApplicationOutputAction for WorthQueryRetireOutput {
    const POSTURE: ApplicationMutationOutputPosture = ApplicationMutationOutputPosture::Retire;
}
