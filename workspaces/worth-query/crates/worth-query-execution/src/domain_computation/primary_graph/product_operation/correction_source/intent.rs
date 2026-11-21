//! L6 construction helper for `WorthQueryCorrectionIntent` from a selected source.
use super::*;
use worth_query_declaration::facade::{
    application_aftermath::ApplicationInverseBinding,
    application_operation::ApplicationMutationBinding,
};

pub use worth_query_execution_query::facade::domain_computation::primary_graph::product_operation::correction_source::intent::WorthQueryCorrectionIntent;

pub(super) trait WorthQueryCorrectionIntentRuntimeExt: Sized {
    fn from_disclosed_source<Schema: ApplicationSchema, C: ApplicationCorrectionBinding<Schema>>(
        target: &WorthQuerySelectedProductOperation<'_, Schema>,
        source: &WorthQuerySelectedCorrectionSource<'_, Schema>,
        command_key: &WorthQueryCorrectionCommandKey,
        lineage: &crate::domain_computation::application_aftermath::WorthQueryCorrectionLineageBinding,
    ) -> Self;
}

impl WorthQueryCorrectionIntentRuntimeExt for WorthQueryCorrectionIntent {
    fn from_disclosed_source<Schema: ApplicationSchema, C: ApplicationCorrectionBinding<Schema>>(
        target: &WorthQuerySelectedProductOperation<'_, Schema>,
        source: &WorthQuerySelectedCorrectionSource<'_, Schema>,
        command_key: &WorthQueryCorrectionCommandKey,
        lineage: &crate::domain_computation::application_aftermath::WorthQueryCorrectionLineageBinding,
    ) -> Self {
        use crate::domain_computation::application_aftermath::WorthQueryAftermathCausalRole as Role;
        let occurrence = source.ancestry().source().identity();
        let branch = target.product().observation();
        let original_intent = source.occurrence().original_intent();
        let action = match lineage.role() {
            Role::RecordedReverse => ApplicationInverseBinding::<Schema, C>::IDENTITY,
            Role::RecordedReapply => C::Source::IDENTITY,
        };
        Self::derive_for_source(
            [C::IDENTITY, C::Source::IDENTITY, action],
            occurrence,
            branch,
            &original_intent,
            command_key,
            lineage,
        )
    }
}
