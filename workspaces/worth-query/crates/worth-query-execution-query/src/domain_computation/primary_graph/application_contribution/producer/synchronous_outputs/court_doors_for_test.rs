//! Sealed test doors over the wave's private orphan backstop and the
//! successor round's #277 conversion predicate.

/// The first derived task unreached from `roots` through `prerequisites`
/// (the orphan backstop's walk, uncharged).
#[doc(hidden)]
pub fn first_orphan_for_test(
    roots: &[usize],
    derived: &[bool],
    prerequisites: &[Vec<usize>],
) -> Option<usize> {
    super::wave::first_orphan(roots, derived, prerequisites, &mut |_| Ok::<_, std::convert::Infallible>(()))
        .unwrap_or_else(|never| match never {})
}

/// Whether a successor's failed prepare that stopped with `kind` is the
/// untyped failure the #277 conversion may type (`ProducerUnavailable` only).
#[doc(hidden)]
pub fn converts_to_prior_effect_for_test(
    kind: crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind,
) -> bool {
    super::super::execution::ProducerExecutionStop::ExecutionStopped(
        crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial::new(kind, "successor"),
    )
    .is_untyped_unavailable()
}

/// Whether a request-admission stop (a cancelled or timed-out request) is
/// typed by the #277 conversion: never.
#[doc(hidden)]
pub fn admission_converts_to_prior_effect_for_test() -> bool {
    super::super::execution::denial::request_admission_denied("successor", "cancelled")
        .is_untyped_unavailable()
}
