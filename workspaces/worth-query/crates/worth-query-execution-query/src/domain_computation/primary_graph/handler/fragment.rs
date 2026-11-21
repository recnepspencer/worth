//! One installed fragment check opens the current Query phase's view.

use super::super::fragment_view::{FragmentDenial, Reading, View, Writing};
use super::{CandidateWriter, DecisionReader};
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding,
    application_schema::{fragment::Fragment, ApplicationSchema},
};
use worth_query_execution_lineage::facade::ports::FragmentWritingViewInternalPort as _;

impl<S: ApplicationSchema, B: ApplicationMutationBinding<S>> DecisionReader<'_, '_, '_, S, B> {
    pub fn fragment<F: Fragment>(
        &mut self,
        marker: F,
    ) -> Result<View<'_, F, Reading<'_>>, FragmentDenial> {
        self.reader.fragment(marker)
    }
}
impl<S: ApplicationSchema, B: ApplicationMutationBinding<S>> CandidateWriter<'_, S, B> {
    pub fn fragment<F: Fragment>(
        &mut self,
        marker: F,
    ) -> Result<View<'_, F, Writing<'_>>, FragmentDenial> {
        let definition = self
            .value_bindings
            .fragment(marker)
            .ok_or(FragmentDenial::NotInstalled { fragment: F::NAME })?;
        Ok(View::from_writing(definition, self.candidate))
    }
}
