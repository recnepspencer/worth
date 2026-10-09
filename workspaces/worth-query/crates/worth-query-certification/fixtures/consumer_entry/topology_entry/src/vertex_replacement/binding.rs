use super::*;
use crate::{
    ConsumerPrincipalBinding, ExternalPrincipalMapping, PlanarMutationScope, PlanarPosition,
    Principal,
};
use std::marker::PhantomData;
use worth_query_decl::facade::{application_operation::*, application_schema::*};

pub struct VertexReplacementBinding<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> ApplicationMutationBinding<Schema>
    for VertexReplacementBinding<Schema>
{
    type Input = VertexReplacement;
    type InputBinding = VertexReplacementInputBinding;
    type Result = PlanarVertexReplacementResult;
    type ResultBinding = VertexReplacementResultBinding;
    type IdempotencyKey = u64;
    type Operation = ReplacePlanarVertex;
    type Decision = super::handler::decision::ReplacementDecision<Schema>;
    type Denial = PlanarReplacementDenial;
    type DenialBinding = VertexReplacementDenialBinding;
    type Output = VertexReplacementOutputs;
    type ScopeBinding = PlanarMutationScope<Schema>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<crate::PlanarQuery>;

    const IDENTITY: &'static str = "worth.query.certification.vertex-replacement.v1";
    const HANDLER_IDENTITY: &'static str =
        "worth.query.certification.vertex-replacement-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.certification.vertex-replacement-command.v1";
    const CANDIDATES: ApplicationCandidateRequirements = replacement_requirements();

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

impl<Schema: TopologySchemaBinding> ApplicationMutationIntent<Schema> for VertexReplacement {
    type Binding = VertexReplacementBinding<Schema>;
    fn input(&self) -> &Self {
        self
    }
    fn scope_binding(&self) -> PlanarMutationScope<Schema> {
        PlanarMutationScope::new(BodyKey::reference(), self.scope_key.clone())
    }
}

pub(super) const fn replacement_requirements() -> ApplicationCandidateRequirements {
    ApplicationCandidateRequirements::fixed_shape(
        ApplicationCandidateCardinalityCeiling::fixed(1, 1, 2, 2, 4, 0),
        ApplicationCandidateResourceCeiling::representation_bytes(8192),
    )
}

pub struct VertexReplacementOutputs;

/// The vertex a replacement keeps as its anchor.
pub struct VertexReplacementAnchorOutput<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> WorthQueryApplicationOutputRole
    for VertexReplacementAnchorOutput<Schema>
{
    type Schema = Schema;
    type Contract = VertexReplacementOutputs;
    type Entity = Body;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "anchor";
}

/// The vertex a replacement creates.
pub struct VertexReplacementCreatedOutput<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> WorthQueryApplicationOutputRole
    for VertexReplacementCreatedOutput<Schema>
{
    type Schema = Schema;
    type Contract = VertexReplacementOutputs;
    type Entity = Body;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "replacement";
}

/// The vertex a replacement retires.
pub struct VertexReplacementRetiredOutput<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> WorthQueryApplicationOutputRole
    for VertexReplacementRetiredOutput<Schema>
{
    type Schema = Schema;
    type Contract = VertexReplacementOutputs;
    type Entity = Body;
    type Action = WorthQueryRetireOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "retired";
}

impl<Schema: TopologySchemaBinding> ApplicationMutationOutputContract<Schema>
    for VertexReplacementOutputs
{
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[
        <VertexReplacementAnchorOutput<Schema> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <VertexReplacementCreatedOutput<Schema> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <VertexReplacementRetiredOutput<Schema> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
    ];
}
