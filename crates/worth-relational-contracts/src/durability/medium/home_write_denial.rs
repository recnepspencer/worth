#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelationalHomeProgramWriteDenial {
    TransitionRequired,
    BodyMismatch,
    BodyCapacityOverflow,
    BodyEncoding,
    UnsupportedBody,
    TransitionAlreadyBound,
    TransitionNotRequested,
    MediumMoved,
    Admission(worth_proof::HomeProgramTransitionAdmissionDenial),
}
