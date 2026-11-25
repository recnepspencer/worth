/// Why Relational reported an installed custom invariant not applicable
/// instead of evaluating it against the candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CustomInvariantNonApplicability {
    /// The candidate's complete footprint was proven and touches no kind in
    /// the rule's declared closure: the rule holds vacuously.
    CandidateOutsideClosure,
    /// The footprint was not proven outside the closure; the rule's scope
    /// planner found no touch it applies to.
    NoApplicableTouch,
    /// Checked scope preparation was refused before the rule could run.
    PreparationRefused,
}
