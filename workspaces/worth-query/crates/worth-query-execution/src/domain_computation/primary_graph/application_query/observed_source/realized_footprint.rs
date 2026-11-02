//! Descriptive qualification posture of this actual completed invocation.
use super::WorthQueryObservedSource;
impl<Query> WorthQueryObservedSource<Query> {
    /// Whether this invocation retained installed, verified realized influence.
    /// Descriptive only; this supplies no READ, currentness or publication grant.
    pub fn has_verified_realized_footprint(&self) -> bool {
        self.invocation_managed_source
            .as_ref()
            .is_some_and(|source| source.has_verified_realized_footprint())
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn invocation_source_authorization(&self) -> Result<Option<&std::sync::Arc<crate::domain_computation::primary_graph::application_attempt::SourceReadAuthorization>>, super::WorthQuerySourceExpectationDenial>{
        let declared = self.source_meaning.footprint().source_authorization();
        let actual = self.invocation_source_authorization.as_ref();
        if declared.map(|value| value.key()) != actual.map(|value| value.key())
            || self
                .invocation_managed_source
                .as_ref()
                .is_some_and(|managed| {
                    actual.is_none_or(|actual| {
                        !std::sync::Arc::ptr_eq(managed.authorization(), actual)
                    })
                })
        {
            return Err(super::WorthQuerySourceExpectationDenial::new(
                super::WorthQuerySourceExpectationDenialKind::IncompleteFootprint,
                self.query_identifier.as_str(),
            ));
        }
        Ok(actual)
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn invocation_managed_source(
        &self,
    ) -> Result<
        Option<
            &std::sync::Arc<
                crate::domain_computation::primary_graph::application_attempt::ManagedSourceRead,
            >,
        >,
        super::WorthQuerySourceExpectationDenial,
    > {
        if self
            .source_meaning
            .footprint()
            .maintained_source()
            .is_some()
            != self.invocation_managed_source.is_some()
        {
            return Err(super::WorthQuerySourceExpectationDenial::new(
                super::WorthQuerySourceExpectationDenialKind::IncompleteFootprint,
                self.query_identifier.as_str(),
            ));
        }
        Ok(self.invocation_managed_source.as_ref())
    }
}

impl<Query> super::WorthQueryObservedResultSet<Query> {
    /// Descriptive qualification posture, never a currentness or publication grant.
    pub fn has_verified_realized_footprint(&self) -> bool {
        self.source().has_verified_realized_footprint()
    }
}
