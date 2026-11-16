//! Issued source custody for authored stages; validation stays in mutation admission.
use super::*;
use crate::application_entry::mutation::request::WorthQueryMutationExpectedSource;
use worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation;
use worth_query_execution::facade::primary_graph::{
    WorthQueryObservedResultSet, WorthQueryObservedSource,
};

mod sealed {
    pub trait Source {}
    impl Source for () {}
    impl<Q> Source for super::WorthQueryObservedSource<Q> {}
    impl<Q> Source for super::WorthQueryObservedResultSet<Q> {}
}

/// A stage accepts only an issued row/result set, or a declared no-source intent.
#[doc(hidden)]
pub trait EditStageSource<S: ApplicationSchema, I: ApplicationMutationIntent<S>>:
    sealed::Source + Send + Sync + 'static
{
    #[doc(hidden)]
    fn retain(
        &self,
        target: &mut crate::application_entry::mutation::staged::WorthQueryStagedMutation<S, I>,
    );
}
impl<S: ApplicationSchema, I: ApplicationMutationIntent<S>> EditStageSource<S, I> for ()
where
    I::Binding: ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
{
    fn retain(
        &self,
        _: &mut crate::application_entry::mutation::staged::WorthQueryStagedMutation<S, I>,
    ) {
    }
}
impl<S: ApplicationSchema, I: ApplicationMutationIntent<S>, Q: 'static> EditStageSource<S, I>
    for WorthQueryObservedSource<Q>
where
    <I::Binding as ApplicationMutationBinding<S>>::SourceExpectation:
        ApplicationMutationSourceExpectation<S, Query = Q>,
{
    fn retain(
        &self,
        target: &mut crate::application_entry::mutation::staged::WorthQueryStagedMutation<S, I>,
    ) {
        target.source = Some(WorthQueryMutationExpectedSource::Row(self.clone()));
    }
}
impl<S: ApplicationSchema, I: ApplicationMutationIntent<S>, Q: 'static> EditStageSource<S, I>
    for WorthQueryObservedResultSet<Q>
where
    <I::Binding as ApplicationMutationBinding<S>>::SourceExpectation:
        ApplicationMutationSourceExpectation<S, Query = Q>,
{
    fn retain(
        &self,
        target: &mut crate::application_entry::mutation::staged::WorthQueryStagedMutation<S, I>,
    ) {
        target.source = Some(WorthQueryMutationExpectedSource::ResultSet(self.clone()));
    }
}
