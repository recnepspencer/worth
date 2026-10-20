//! The real planar mutation result, scope, and complete output contract.
use super::fixture_values::{PlanarAdjustmentResult, PlanarMutationDenial};
use super::*;
use std::marker::PhantomData;
use worth_query_decl::facade::{
    application_operation::{
        ApplicationMutationFieldScope, ApplicationMutationOutputContract,
        ApplicationMutationOutputPostureSet, ApplicationMutationOutputRoleDescriptor,
        ApplicationMutationOutputRoleFamilyDescriptor, WorthQueryApplicationDeclaredOutputRole,
        WorthQueryApplicationDeclaredOutputRoleFamily, WorthQueryApplicationOutputRole,
        WorthQueryApplicationOutputRoleFamily, WorthQueryExactlyOneOutput,
        WorthQueryPreserveOutput,
    },
    application_schema::{NoApplicationUnit, ReadOnly},
    worth_query_structured_value_binding,
};
worth_query_structured_value_binding!(pub PlanarMutationResultBinding for PlanarAdjustmentResult { identity: "worth.query.certification.planar-mutation-result.v1" });
worth_query_structured_value_binding!(pub PlanarMutationDenialBinding for PlanarMutationDenial { identity: "worth.query.certification.planar-mutation-denial.v1" });
pub type PlanarMutationScope<Schema> = ApplicationMutationFieldScope<
    Schema,
    Body,
    PlanarPosition,
    BodyKey,
    String,
    ReadOnly,
    NoApplicationUnit,
>;

/// The output contract of every planar mutation.
///
#[doc = include_str!("../output_role_contract.md")]
pub struct PlanarOutputs;

/// The body every planar mutation preserves as its anchor, for every binding
/// whose outputs are [`PlanarOutputs`].
pub struct PlanarAnchorOutput<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> WorthQueryApplicationOutputRole for PlanarAnchorOutput<Schema> {
    type Schema = Schema;
    type Contract = PlanarOutputs;
    type Entity = Body;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "anchor";
}

/// The bodies a planar mutation creates, one member per created vertex.
pub struct PlanarCreatedOutputs<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> WorthQueryApplicationOutputRoleFamily
    for PlanarCreatedOutputs<Schema>
{
    type Schema = Schema;
    type Contract = PlanarOutputs;
    type Entity = Body;
    const PREFIX: &'static str = "created.";
    const POSTURES: ApplicationMutationOutputPostureSet =
        ApplicationMutationOutputPostureSet::CREATE;
    const MINIMUM: usize = 0;
}

impl<Schema: TopologySchemaBinding> ApplicationMutationOutputContract<Schema> for PlanarOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[<PlanarAnchorOutput<Schema> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
    const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] = &[
        <PlanarCreatedOutputs<Schema> as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR,
    ];
}
