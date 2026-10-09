use super::*;
use std::marker::PhantomData;
use worth_query_consumer_values::{PlanarAdjustmentResult, PlanarMutationDenial, PlanarOperation};
use worth_query_decl::facade::{
    application_operation::*, application_schema::*, worth_query_operation,
    worth_query_operation_creates, worth_query_operation_links, worth_query_operation_reads,
    worth_query_operation_unlinks, worth_query_operation_writes,
    worth_query_structured_value_binding,
};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct PlanarMutation {
    pub scope_key: String,
    pub operation: PlanarOperation,
}
worth_query_structured_value_binding!(pub PlanarMutationInputBinding for PlanarMutation { identity: "worth.query.certification.planar-mutation-input.v2" });
worth_query_structured_value_binding!(pub PlanarMutationResultBinding for PlanarAdjustmentResult { identity: "worth.query.certification.planar-mutation-result.v1" });
worth_query_structured_value_binding!(pub PlanarMutationDenialBinding for PlanarMutationDenial { identity: "worth.query.certification.planar-mutation-denial.v1" });
worth_query_operation!(pub MutatePlanar for Schema: TopologySchemaBinding, input PlanarMutationInputBinding);
worth_query_operation_reads!(MutatePlanar => [Body, BodyKey, PositionX, PositionY, Length, PlanarSuccessor]);
worth_query_operation_writes!(MutatePlanar => [BodyKey, PositionX, PositionY, Length]);
worth_query_operation_creates!(MutatePlanar => [Body]);
worth_query_operation_links!(MutatePlanar => [PlanarSuccessor]);
worth_query_operation_unlinks!(MutatePlanar => [PlanarSuccessor]);

pub struct PlanarMutationBinding<Schema>(PhantomData<fn() -> Schema>);
pub type PlanarMutationScope<Schema> = ApplicationMutationFieldScope<
    Schema,
    Body,
    PlanarPosition,
    BodyKey,
    String,
    ReadOnly,
    NoApplicationUnit,
>;

impl<Schema: TopologySchemaBinding> ApplicationMutationBinding<Schema>
    for PlanarMutationBinding<Schema>
{
    type Input = PlanarMutation;
    type InputBinding = PlanarMutationInputBinding;
    type Result = PlanarAdjustmentResult;
    type ResultBinding = PlanarMutationResultBinding;
    type IdempotencyKey = u64;
    type Operation = MutatePlanar;
    type Decision = ();
    type Denial = PlanarMutationDenial;
    type DenialBinding = PlanarMutationDenialBinding;
    type Output = PlanarOutputs;
    type ScopeBinding = PlanarMutationScope<Schema>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<PlanarQuery>;
    const IDENTITY: &'static str = "worth.query.certification.planar-mutation.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.certification.planar-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.certification.planar-command.v1";
    const CANDIDATES: ApplicationCandidateRequirements = requirements(16, 16, 16, 64, 8192);
    fn scope_field() -> ApplicationFieldRef<
        Schema,
        Body,
        PlanarPosition,
        BodyKey,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        BodyKey::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        ConsumerPrincipalBinding,
        ExternalPrincipalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ConsumerPrincipalBinding::reference()
    }
}
impl<Schema: TopologySchemaBinding> ApplicationMutationIntent<Schema> for PlanarMutation {
    type Binding = PlanarMutationBinding<Schema>;
    fn input(&self) -> &Self {
        self
    }
    fn scope_binding(&self) -> PlanarMutationScope<Schema> {
        PlanarMutationScope::new(BodyKey::reference(), self.scope_key.clone())
    }
}
pub const fn requirements(
    creates: usize,
    links: usize,
    unlinks: usize,
    writes: usize,
    bytes: usize,
) -> ApplicationCandidateRequirements {
    ApplicationCandidateRequirements::fixed_shape(
        ApplicationCandidateCardinalityCeiling::fixed(creates, 0, links, unlinks, writes, 0),
        ApplicationCandidateResourceCeiling::representation_bytes(bytes),
    )
}

/// The output contract of every planar mutation.
///
#[doc = include_str!("output_role_contract.md")]
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
