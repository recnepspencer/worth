use worth_query_consumer_values::{PlanarAdjustment, PlanarOperation};
use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;
use worth_query_host::facade::primary_graph::{
    WorthQueryGeneratedOutputSuspensionDenial, WorthQueryGeneratedOutputSuspensionFailure,
    WorthQueryObservedSource, WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_topology_entry::{InitialPlanarProducer, PlanarMutation, PlanarQuery};

use super::super::{length, mutate, Request};
use crate::ConsumerSchema;

pub(super) fn publish_unrelated(
    request: &Request<'_>,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
) {
    let successor = mutate(
        request,
        PlanarMutation {
            scope_key: "sibling-a".to_owned(),
            operation: PlanarOperation::Adjust(vec![PlanarAdjustment {
                body_key: "sibling-a".to_owned(),
                replacement_y: length(22),
            }]),
            validator_work: 4_096,
        },
        981,
    );
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = successor else {
        panic!("the unrelated successor must publish: {successor:?}")
    };
    assert_eq!(receipt.product_branch(), branch);
}

pub(super) fn require_stale_source_denial(
    application: &WorthQueryPrimaryGraphApplicationRuntime<ConsumerSchema>,
    scope: &worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
    source: WorthQueryObservedSource<PlanarQuery>,
) {
    match application
        .on_branch(branch)
        .select()
        .expect("the Query-issued output occurrence remains selectable")
        .suspend_current_generated_output::<InitialPlanarProducer<ConsumerSchema>>(scope, source)
    {
        Err(WorthQueryGeneratedOutputSuspensionFailure::Qualification(
            WorthQueryGeneratedOutputSuspensionDenial::SourceMismatch,
        )) => {}
        _ => panic!("a predecessor source proof must not suspend current output"),
    }
}

pub(super) fn failure_name(
    failure: &worth_query_host::facade::primary_graph::WorthQueryGeneratedOutputSuspensionFailure,
) -> &'static str {
    use worth_query_host::facade::primary_graph::WorthQueryGeneratedOutputSuspensionFailure::*;
    match failure {
        Qualification(_) => "qualification",
        ProductActivationUnavailable => "product-activation-unavailable",
        Preparation => "preparation",
        PublicationNoEffect => "publication-no-effect",
        ProductUnpublished(_) => "product-unpublished",
    }
}
