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
    type Decision = ();
    type Denial = PlanarReplacementDenial;
    type DenialBinding = VertexReplacementDenialBinding;
    type Output = VertexReplacementOutputs;
    type ScopeBinding = PlanarMutationScope<Schema>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;

    const IDENTITY: &'static str = "worth.query.certification.vertex-replacement.v1";
    const HANDLER_IDENTITY: &'static str =
        "worth.query.certification.vertex-replacement-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.certification.vertex-replacement-command.v1";
    const CANDIDATES: ApplicationCandidateRequirements = replacement_requirements();

    fn idempotency_key_identity(key: &u64) -> [u8; 32] {
        super::identity::command_identity(*key)
    }
    fn input_identity(input: &VertexReplacement) -> [u8; 32] {
        super::identity::input_identity(input)
    }

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
    fn scope_binding(&self) -> PlanarMutationScope<Schema> {
        PlanarMutationScope::new(BodyKey::reference(), self.scope_key.clone())
    }
}

pub(super) const fn replacement_requirements() -> ApplicationCandidateRequirements {
    ApplicationCandidateRequirements::fixed_shape(
        ApplicationCandidateCardinalityCeiling::fixed(1, 1, 2, 2, 4, 0),
        ApplicationCandidateResourceCeiling::bounded(8192, 4096),
    )
}

pub struct VertexReplacementOutputs;
impl<Schema: TopologySchemaBinding> ApplicationMutationOutputContract<Schema>
    for VertexReplacementOutputs
{
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[
        ApplicationMutationOutputRoleDescriptor::for_entity::<Schema, Body>(
            "anchor",
            ApplicationMutationOutputPosture::Preserve,
        ),
        ApplicationMutationOutputRoleDescriptor::for_entity::<Schema, Body>(
            "replacement",
            ApplicationMutationOutputPosture::Create,
        ),
        ApplicationMutationOutputRoleDescriptor::for_entity::<Schema, Body>(
            "retired",
            ApplicationMutationOutputPosture::Retire,
        ),
    ];
}
