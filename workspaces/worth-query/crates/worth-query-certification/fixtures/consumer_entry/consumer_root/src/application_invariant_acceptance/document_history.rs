//! Executable document commands use the same public owners as any renderer.
use super::{authentication, installation, seed::length};
use crate::{
    application_program::historical::{FixedAction, FixedProgram, Original},
    ConsumerSchema,
};
use std::num::NonZeroUsize;
use worth_query_decl::facade::{
    application_operation::ApplicationMutationIntent,
    application_program::{
        ApplicationAction, ApplicationFixedActionDelivery, ApplicationProgramAuthoring,
    },
};
use worth_query_host::facade::application_entry::*;
use worth_query_topology_entry::{
    PlanarDocumentHistory, PlanarHistoryBookmark, PlanarHistoryNavigation, PlanarHistoryWindow,
    PlanarInitialAdjustment, PlanarOutputDemand, PlanarOutputRead, PlanarRead,
};

mod authoring;
mod journey;
pub(crate) mod model;
mod navigation;
mod setup;
mod view;

type History = PlanarDocumentHistory<ConsumerSchema>;
type Runtime = installation::ConsumerRuntime<FixedProgram>;
type Request<'a, 'p, 's> = WorthQueryApplicationRequest<'a, 'p, 's, ConsumerSchema>;
type Document<'a, O = WorthQueryHistoryNoOutputs> =
    WorthQueryApplicationHistoryDocument<'a, ConsumerSchema, History, OrdinaryHistoryPayload, O>;
type Binding = <Original as ApplicationMutationIntent<ConsumerSchema>>::Binding;
type Delivery = <FixedAction as ApplicationAction<ConsumerSchema, Binding>>::Delivery;
type Root = <Delivery as ApplicationFixedActionDelivery<ConsumerSchema, Binding>>::Root;
type Outputs = WorthQueryHistoryWithOutputs<ConsumerSchema, FixedProgram, Root>;
type Configured<'a> = Document<'a, Outputs>;

pub(crate) fn run(
    installed: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let world = setup::install(installed);
    journey::run(&world.application);
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_executable_document_adapter_selects_both_futures_and_retains_real_output_custody() {
        super::run(&crate::installed_schema());
    }
}
