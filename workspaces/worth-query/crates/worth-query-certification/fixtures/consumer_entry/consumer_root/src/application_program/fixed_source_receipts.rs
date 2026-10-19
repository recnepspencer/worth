//! Two native Fixed sources share admission scope while their targets remain independent.
use super::*;
use worth_query_consumer_values::{PlanarAdjustment, PlanarOperation, PositiveLength};
use worth_query_decl::facade::application_program::*;
use worth_query_host::facade::{application_entry::*, primary_graph::*};
use worth_query_topology_entry::{PlanarEdit, PlanarMutation, PlanarOutputDemand};

pub(crate) struct SourceConnection;
impl ApplicationConnectionIdentity for SourceConnection {
    const IDENTITY: &'static str = "consumer.shared-scope-fixed-source.v1";
}
impl
    ApplicationOccurrenceConnectionBinding<ConsumerSchema, PlanarSourceFeature, PlanarOutputFeature>
    for SourceConnection
{
}

pub(crate) fn target(
    input: &PlanarMutation,
) -> Result<&str, WorthQueryRequiredOutputConnectionDenial> {
    let PlanarOperation::Adjust(adjustments) = &input.operation else {
        return Err(WorthQueryRequiredOutputConnectionDenial::new(
            "source is not an adjustment",
        ));
    };
    let [adjustment] = adjustments.as_slice() else {
        return Err(WorthQueryRequiredOutputConnectionDenial::new(
            "source must adjust exactly one target",
        ));
    };
    Ok(&adjustment.body_key)
}
impl WorthQueryApplicationRequiredOutputConnection<ConsumerSchema> for SourceConnection {
    type Source = PlanarEditBinding<ConsumerSchema>;
    type Demand = PlanarOutputDemand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        input: &PlanarMutation,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(PlanarOutputDemand::new(target(input)?))
    }
}
impl WorthQueryApplicationRequiredOutputSource<ConsumerSchema, SourceConnection>
    for PlanarEditBinding<ConsumerSchema>
{
    fn demand_from_source(
        input: &PlanarMutation,
    ) -> Result<PlanarOutputDemand, WorthQueryRequiredOutputConnectionDenial> {
        <SourceConnection as WorthQueryApplicationRequiredOutputConnection<ConsumerSchema>>::demand_from_source(input)
    }
}
pub(crate) type Root = ApplicationOutputGraph<
    ApplicationConnectionInstanceRef<
        ConsumerSchema,
        SecondaryPlanarRoot,
        PlanarSourceFeature,
        PlanarBodyOutput,
        SecondaryPlanarRoot,
        PlanarOutputFeature,
        PlanarBodyInput,
        SourceConnection,
    >,
    ApplicationOutputLeaf,
>;
pub(crate) struct Delivery;
impl ApplicationActionDelivery<ConsumerSchema, PlanarEditBinding<ConsumerSchema>> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Root, ApplicationNoProjection>;
    const IDENTITY: &'static str = "consumer.shared-scope-fixed-delivery.v1";
}
impl ApplicationFixedActionDelivery<ConsumerSchema, PlanarEditBinding<ConsumerSchema>>
    for Delivery
{
    type Root = Root;
    type Projection = ApplicationNoProjection;
}
pub(crate) type Action = ApplicationActionReference<
    ConsumerSchema,
    SecondaryPlanarRoot,
    PlanarSourceFeature,
    PlanarEditBinding<ConsumerSchema>,
    Delivery,
>;
pub(crate) struct Program;
impl ApplicationProgramDefinition<ConsumerSchema> for Program {
    type Contributions =
        <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Contributions;
    type Outputs = ApplicationProgramOutputs<(
        ConsumerProgramRoot,
        (
            Root,
            (
                ConsumerDiscoveredProgramRoot,
                (
                    ConsumerRequiredSharedRoot,
                    ConsumerInitialDiscoveredProgramRoot,
                ),
            ),
        ),
    )>;
    type Rules = <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("consumer.shared-scope-fixed");
    fn adoption_authorization<V: ApplicationProgramAdoptionAuthorizationVisitor<ConsumerSchema>>(
        visitor: V,
    ) -> V::Output {
        visitor.capability::<crate::program_deployment::ConsumerProgramDeploymentBinding>()
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = ConsumerProgram::feature_specs();
        let secondary =
            ApplicationFeatureSpec::at::<ConsumerSchema, SecondaryPlanarRoot, PlanarSourceFeature>(
            )
            .provides::<PlanarBodyOutput>()
            .action::<PlanarEditBinding<ConsumerSchema>, Delivery>()
            .correspondence::<correspondence::OptionalAdjustmentCorrespondence>()
            .external_input::<external_input::NeutralExternalProvider>()
            .finish_action()
            .finish();
        let mut positions = specs
            .iter()
            .enumerate()
            .filter(|(_, spec)| spec.feature() == secondary.feature())
            .map(|(index, _)| index);
        let index = positions
            .next()
            .expect("the actual secondary source occurrence exists");
        assert!(positions.next().is_none(), "one exact canonical occurrence");
        specs[index] = secondary;
        specs
    }
}
pub(crate) fn input(target: &str, value: PositiveLength) -> PlanarEdit {
    PlanarEdit(PlanarMutation {
        scope_key: "anchor-a".into(),
        operation: PlanarOperation::Adjust(vec![PlanarAdjustment {
            body_key: target.into(),
            replacement_y: value,
        }]),
    })
}
pub(crate) type Fixed = WorthQueryFixedOperation<ConsumerSchema, PlanarEdit, Program, Root, Action>;
pub(crate) fn prepare(
    runtime: &worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<ConsumerSchema, Program>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, ConsumerSchema>,
    input: PlanarEdit,
    key: u64,
) -> Fixed {
    let source = request
        .query(worth_query_topology_entry::PlanarRead {
            body_key: input.0.scope_key.clone(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let operation = request
        .mutate(input)
        .expect_source(source)
        .idempotency_owned(key)
        .unwrap_or_else(|(_, _, cause)| panic!("source capture: {cause:?}"))
        .retain_operation::<Program, Action>(runtime)
        .unwrap_or_else(|(_, cause)| panic!("fixed source construction: {cause:?}"));
    assert!(operation.prepared_source_receipt().is_none());
    match operation.advance(request) {
        WorthQueryOperationAdvance::Progress {
            operation,
            progress: WorthQueryOperationProgress::SourcePerformed,
        } => operation,
        _ => panic!("original native source must reach its preparation boundary"),
    }
}
pub(crate) fn finish(
    mut operation: Fixed,
    request: &WorthQueryApplicationRequest<'_, '_, '_, ConsumerSchema>,
) -> WorthQuerySettledFixedOperation<ConsumerSchema, PlanarEdit, Program, Root, Action> {
    loop {
        match operation.advance(request) {
            WorthQueryOperationAdvance::Progress {
                operation: next,
                progress,
            } => {
                if matches!(progress, WorthQueryOperationProgress::OutputsStarted) {
                    assert!(next.prepared_source_receipt().is_none());
                }
                operation = next;
            }
            WorthQueryOperationAdvance::Settled(settled) => return settled,
            WorthQueryOperationAdvance::Refused { denial, .. }
            | WorthQueryOperationAdvance::Terminal { denial, .. } => {
                panic!("shared-scope output progression: {denial:?}")
            }
        }
    }
}

pub(crate) fn assert_single_adjustment_shape() {
    let y = PositiveLength::new(22).unwrap();
    let mut invalid = input("sibling-a", y).0;
    invalid.operation = PlanarOperation::Adjust(vec![]);
    assert!(target(&invalid).is_err());
    let PlanarOperation::Adjust(mut two) = input("sibling-a", y).0.operation else {
        unreachable!()
    };
    two.extend(two.clone());
    invalid.operation = PlanarOperation::Adjust(two);
    assert!(target(&invalid).is_err());
}

pub(crate) fn assert_y(
    request: &WorthQueryApplicationRequest<'_, '_, '_, ConsumerSchema>,
    target: &str,
    value: PositiveLength,
) {
    assert_eq!(
        request
            .query(worth_query_topology_entry::PlanarRead {
                body_key: target.into()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y,
        value
    );
}

pub(crate) fn edit_target(
    runtime: &worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<ConsumerSchema, Program>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, ConsumerSchema>,
) -> Fixed {
    prepare(
        runtime,
        request,
        input("sibling-a", PositiveLength::new(23).unwrap()),
        11_203,
    )
}

pub(crate) fn deployment_grants() -> [crate::program_deployment::ConsumerDeploymentGrantSeed; 1] {
    [
        crate::program_deployment::ConsumerDeploymentGrantSeed::for_program::<Program>(
            "model-owner",
            "model-owner",
        ),
    ]
}

/// Receipt identities are descriptive; none of these checks admits a live row.
pub(crate) fn assert_receipt_descriptions<Product: std::fmt::Debug + Eq>(
    sources: &[(
        u64,
        WorthQueryOperationScopeEntityBinding,
        String,
        Product,
        u64,
    )],
    producers: &[(
        u64,
        WorthQueryOperationScopeEntityBinding,
        String,
        Product,
        u64,
    )],
) {
    assert_eq!(sources.len(), 2);
    assert_eq!(producers.len(), 2);
    assert_eq!(sources[0].0, sources[1].0);
    assert_eq!(sources[0].1, sources[1].1);
    assert_eq!(sources[0].2, sources[1].2);
    assert_eq!(sources[0].3, sources[1].3);
    assert_ne!(
        sources[0].4, sources[1].4,
        "authentic original publications remain distinct"
    );
    assert_ne!(
        sources[0], producers[0],
        "producer evidence cannot stand in for its source"
    );
    assert_ne!(sources[1], producers[1]);
    assert_ne!(sources[0].2, producers[0].2);
    assert_ne!(producers[0], producers[1]);
}

/// Projection checks preserve multiplicity within ONE publication window.
pub(crate) fn assert_known_window<Product: std::fmt::Debug + Ord>(
    actual: &mut [(
        u64,
        WorthQueryOperationScopeEntityBinding,
        String,
        Product,
        u64,
    )],
    sources: &[(
        u64,
        WorthQueryOperationScopeEntityBinding,
        String,
        Product,
        u64,
    )],
    producers: &[(
        u64,
        WorthQueryOperationScopeEntityBinding,
        String,
        Product,
        u64,
    )],
) {
    actual.sort();
    assert!(
        actual.windows(2).all(|pair| pair[0] != pair[1]),
        "ambiguous projected marking membership"
    );
    assert!(actual
        .iter()
        .all(|item| sources.contains(item) || producers.contains(item)));
}

pub(crate) fn assert_source_edit_membership<Product: std::fmt::Debug + Eq + Clone>(
    actual: &[(
        u64,
        WorthQueryOperationScopeEntityBinding,
        String,
        Product,
        u64,
    )],
    sources: &[(
        u64,
        WorthQueryOperationScopeEntityBinding,
        String,
        Product,
        u64,
    )],
    producers: &[(
        u64,
        WorthQueryOperationScopeEntityBinding,
        String,
        Product,
        u64,
    )],
) {
    // Adjust reads then replaces Y: matching original source ordinals start dirty.
    // Producers read Y but write Length. The later Y edit newly dirties producer0,
    // not either born-dirty source. This does not prove source registration exists.
    let expected = vec![producers[0].clone()];
    assert_eq!(
        actual, expected,
        "complete independent semantic multiset, including multiplicity"
    );
    assert!(
        !actual.contains(&sources[1]) && !actual.contains(&producers[1]),
        "same-scope sibling excluded"
    );
}
