//! Finite installed policy for unfinished snapshot-navigation attempts.
use std::num::NonZeroUsize;

/// Each attached document admits this many distinct unfinished navigation
/// attempts. Same-key resolution shares the original slot. Actual selection or
/// completed disposal releases it; completed history and cleanup have their
/// existing, separate resource owners.
///
/// Set `ApplicationHistoryBinding::NAVIGATION_RESOURCES` in the history's
/// implementation to a finite policy such as:
/// ```
/// use std::num::NonZeroUsize;
/// use worth_query_declaration::facade::application_history::ApplicationHistoryNavigationResources;
/// const NAVIGATION_RESOURCES: ApplicationHistoryNavigationResources =
///     ApplicationHistoryNavigationResources {
///         maximum_unfinished_attempts: NonZeroUsize::new(4).unwrap(),
///     };
/// assert_eq!(NAVIGATION_RESOURCES.maximum_unfinished_attempts.get(), 4);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationHistoryNavigationResources {
    pub maximum_unfinished_attempts: NonZeroUsize,
}

impl ApplicationHistoryNavigationResources {
    pub const DEFAULT: Self = Self {
        maximum_unfinished_attempts: NonZeroUsize::new(8).unwrap(),
    };
}
