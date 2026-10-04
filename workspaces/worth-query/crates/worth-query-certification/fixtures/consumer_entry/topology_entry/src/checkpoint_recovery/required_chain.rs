//! Real consumed-output edges over three independently observed native sources.

use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationOutputDemandProgress, WorthQueryOutputSettlementPosture,
};
#[cfg(feature = "test-query-execution-observer")]
use worth_query_host::facade::primary_graph::query_read_kernel_entries_on_this_thread_for_test as query_entries;

mod binding;
#[cfg(feature = "test-query-execution-observer")]
mod diamond;
mod performed_head_movement;
mod producer;
mod program;
mod readiness;
#[cfg(feature = "test-query-execution-observer")]
mod required_queue;
mod restored_currentness;
mod source_world;
mod stable_alias_currentness;
use binding::*;
use producer::*;

pub(super) fn output_feature_spec() -> ApplicationFeatureSpec {
    ApplicationFeatureSpec::root::<CheckpointSchema, PlanarOutputFeature>()
        .provides::<PlanarDerivedBodyOutput>()
        .conditional_operation::<MutatePlanar>()
        .conditional_operation::<PublishChain>()
        .finish()
}

pub(crate) fn declare<Schema: TopologySchemaBinding>(
    schema: worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<
        Schema,
    >,
) -> worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<Schema> {
    binding::declare(schema)
}

pub(crate) fn contracts<Schema: TopologySchemaBinding>(
    contracts: &mut worth_query_host::facade::application_contribution::WorthQueryApplicationContributionContracts<Schema>,
) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
    contracts.producer::<ChainProducer<Schema>>()?;
    contracts.conditional::<readiness::ChainReadiness<Schema>>()?;
    Ok(())
}

pub(crate) fn configure<Schema: TopologySchemaBinding>(
    setup: &mut worth_query_host::facade::application_contribution::WorthQueryApplicationContributionSetup<'_, Schema>,
) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
    setup.handler::<ChainBinding<Schema>, _>(ChainHandler)?;
    setup.producer::<ChainProducer<Schema>>(ChainProvider)?;
    setup.conditional::<readiness::ChainReadiness<Schema>>(())
}

#[test]
fn one_advance_discharge_follows_real_consumed_output_edges_after_upstream_stable_cutoff() {
    let _guard = checkpoint_recovery_test_guard();
    let profile =
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::standard();
    let application = support::install_program_with_seed::<program::ChainProgram>(
        None,
        profile,
        32,
        128 * 1_024 * 1_024,
        u64::try_from(profile.limits().source_currentness_work()).unwrap(),
        source_world::seed,
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut a = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .start_in_program::<program::ChainProgram, program::ChainRoot>(&application)
        .unwrap();
    let mut b = request
        .demand(ChainDemand("anchor-b".to_owned()))
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&application)
        .unwrap();
    let mut c = request
        .demand(ChainDemand("anchor-c".to_owned()))
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&application)
        .unwrap();
    let initial_a = (0..256)
        .find_map(|_| match a.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending => None,
            WorthQueryApplicationOutputDemandProgress::Settled(settled) => Some(settled),
        })
        .expect("the actual upstream initially settles");
    assert_eq!(initial_a.producer_contacts_in_this_demand(), 1);
    let initial_b = (0..256)
        .find_map(|_| match b.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending => None,
            WorthQueryApplicationOutputDemandProgress::Settled(settled) => Some(settled),
        })
        .expect("the actual middle consumer initially settles");
    assert_eq!(initial_b.producer_contacts_in_this_demand(), 1);
    let original = (0..256)
        .find_map(|_| match c.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending => None,
            WorthQueryApplicationOutputDemandProgress::Settled(settled) => Some(settled),
        })
        .expect("the actual A to B to C chain initially settles");
    assert_eq!(original.producer_contacts_in_this_demand(), 1);

    let before_input = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap();
    let selected = request
        .query(PlanarRead {
            body_key: "anchor-source-c".to_owned(),
        })
        .execute()
        .unwrap();
    request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-source-c".to_owned(),
            replacement_y: length(11),
        })
        .expect_source(selected.observed_sources()[0].clone())
        .idempotency(&0x9176_3001_u64)
        .execute_performed::<program::ChainProgram, program::ChainRoot>(&application)
        .unwrap();
    drop(selected);
    let changed = request
        .query(PlanarRead {
            body_key: "anchor-source-c".to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        changed.rows()[0].y,
        length(11),
        "the source-only dependency really changed"
    );
    let after_input = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        before_input.rows(),
        after_input.rows(),
        "the upstream producer's declared input stays equal"
    );
    drop((before_input, after_input, changed));
    let before = request.retain_read().unwrap();
    #[cfg(feature = "test-query-execution-observer")]
    let before_entries = query_entries();
    let settled = match c
        .advance(&request)
        .expect("one caller pumps the actual prerequisite chain")
    {
        WorthQueryApplicationOutputDemandProgress::Settled(settled) => settled,
        WorthQueryApplicationOutputDemandProgress::Pending => {
            panic!("the finite required chain needs another caller advance")
        }
    };
    let after = request.retain_read().unwrap();
    #[cfg(feature = "test-query-execution-observer")]
    assert!(
        query_entries() > before_entries,
        "the actual dirty upstream is re-evidenced inside this advance"
    );
    assert_eq!(before.selected_commit(), after.selected_commit());
    assert_eq!(settled.producer_contacts_in_this_demand(), 0);
    assert_eq!(
        settled.posture(),
        WorthQueryOutputSettlementPosture::Performed
    );
    assert_eq!(
        settled
            .output_correspondence()
            .entity(binding::anchor_role::<CheckpointSchema>())
            .unwrap()
            .entity_id(),
        original
            .output_correspondence()
            .entity(binding::anchor_role::<CheckpointSchema>())
            .unwrap()
            .entity_id()
    );
    // Keep all three ordinary interests live: their required membership is real.
    drop((a, b, c));
}
