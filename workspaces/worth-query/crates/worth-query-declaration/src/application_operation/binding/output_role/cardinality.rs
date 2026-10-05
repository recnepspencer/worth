use super::super::output::ApplicationMutationOutputRoleCardinality;

mod sealed {
    pub trait Sealed {}
}

/// Cardinality: every completed mutation binds the role exactly once, and
/// reading it yields the bound value itself.
pub struct WorthQueryExactlyOneOutput;
/// Cardinality: a completed mutation binds the role at most once, and
/// reading it yields `Option`, so an absent role is a value, never a denial.
pub struct WorthQueryAtMostOneOutput;

/// How many times a completed mutation binds a fixed output role, and the
/// shape a read of it takes: implemented only by
/// [`WorthQueryExactlyOneOutput`] and [`WorthQueryAtMostOneOutput`].
pub trait WorthQueryApplicationOutputCardinality: sealed::Sealed + 'static {
    const CARDINALITY: ApplicationMutationOutputRoleCardinality;

    /// What a read of a role with this cardinality yields for a bound `T`.
    type Read<T>;

    /// Shape a read: `bound` is the role's binding if the commit has one.
    /// `None` means the read is denied as a missing role.
    fn read<T>(bound: Option<T>) -> Option<Self::Read<T>>;
}

impl sealed::Sealed for WorthQueryExactlyOneOutput {}
impl WorthQueryApplicationOutputCardinality for WorthQueryExactlyOneOutput {
    const CARDINALITY: ApplicationMutationOutputRoleCardinality =
        ApplicationMutationOutputRoleCardinality::ExactlyOne;
    type Read<T> = T;

    fn read<T>(bound: Option<T>) -> Option<T> {
        bound
    }
}

impl sealed::Sealed for WorthQueryAtMostOneOutput {}
impl WorthQueryApplicationOutputCardinality for WorthQueryAtMostOneOutput {
    const CARDINALITY: ApplicationMutationOutputRoleCardinality =
        ApplicationMutationOutputRoleCardinality::AtMostOne;
    type Read<T> = Option<T>;

    fn read<T>(bound: Option<T>) -> Option<Option<T>> {
        Some(bound)
    }
}
