/// Why an application attempt did not register. The request's cancellation
/// or elapsed deadline is its own cause; every other refusal rejects the
/// provider's plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum ApplicationAttemptRegistrationStop {
    Rejected(&'static str),
}

impl From<&'static str> for ApplicationAttemptRegistrationStop {
    fn from(reason: &'static str) -> Self {
        Self::Rejected(reason)
    }
}
