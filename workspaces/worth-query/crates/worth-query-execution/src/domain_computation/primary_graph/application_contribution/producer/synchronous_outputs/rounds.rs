//! Actual producer starts are bounded within one synchronous edit attempt.
pub(in crate::domain_computation::primary_graph::application_contribution::producer) mod limits;
/// Default edit policy, shared by the entry controls and internal source entries.
pub const fn default_producer_round_bounds() -> (std::num::NonZeroUsize, std::num::NonZeroUsize) {
    (
        std::num::NonZeroUsize::new(64).unwrap(),
        std::num::NonZeroUsize::new(4096).unwrap(),
    )
}
