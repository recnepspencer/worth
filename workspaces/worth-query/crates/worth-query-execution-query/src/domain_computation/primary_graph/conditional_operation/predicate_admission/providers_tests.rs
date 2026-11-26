//! Query's conditional output versions: a fresh occurrence always advances
//! past the node's baseline, an exact attempt is echoed, and a host provider
//! always projects from the true attempt.
use super::{computed_output, QueryConditionalComputeContext as Context};
use worth_signal::facade::Aspect;

fn version(context: Context, before: Option<u64>) -> Result<u64, String> {
    computed_output(&context, before, |_| None)
        .map(|result| result.aspect_version.get(Aspect::new(0)))
}

#[test]
fn a_fresh_occurrence_advances_past_whatever_moved_its_baseline() {
    // Deliveries bumped the node to 2 and the wake's attempt is also 2: the
    // occurrence must still differ from its baseline, not echo the attempt.
    assert_eq!(version(Context::fresh(2), Some(2)), Ok(3));
    // A baseline far above any attempt (an earlier output plus bumps).
    assert_eq!(
        version(Context::fresh(3), Some((1 << 63) + 5)),
        Ok((1 << 63) + 6)
    );
    // The last version refuses instead of wrapping onto an earlier one.
    assert!(version(Context::fresh(1), Some(u64::MAX)).is_err());
}

#[test]
fn an_exact_attempt_is_compared_as_supplied() {
    assert_eq!(version(Context::exact(7), Some(7)), Ok(7));
    assert_eq!(version(Context::exact(7), None), Ok(7));
}

#[test]
fn a_host_provider_projects_from_the_true_attempt() {
    let mut seen = None;
    let result = computed_output(&Context::fresh(4), Some(40), |attempt| {
        seen = Some(attempt);
        Some(Ok(9))
    })
    .unwrap();
    assert_eq!(seen, Some(4));
    assert_eq!(result.aspect_version.get(Aspect::new(0)), 9);
}
