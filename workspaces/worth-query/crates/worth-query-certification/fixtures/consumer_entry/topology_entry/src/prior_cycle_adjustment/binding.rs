use super::*;
use crate::{ConsumerPrincipalBinding, ExternalPrincipalMapping, PlanarMutationScope, Principal};
use std::marker::PhantomData;
use worth_query_decl::facade::{application_operation::*, application_schema::*};

pub struct PriorCycleAdjustmentBinding<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> ApplicationMutationBinding<Schema>
    for PriorCycleAdjustmentBinding<Schema>
{
    type Input = PriorCycleAdjustment;
    type InputBinding = PriorCycleAdjustmentInputBinding;
    type Result = PriorCycleAdjustmentResult;
    type ResultBinding = PriorCycleAdjustmentResultBinding;
    type IdempotencyKey = u64;
    type Operation = AdjustPriorCycle;
    type Decision = super::handler::decision::PriorCycleDecision<Schema>;
    type Denial = PriorCycleAdjustmentDenial;
    type DenialBinding = PriorCycleAdjustmentDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = PlanarMutationScope<Schema>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<crate::PlanarQuery>;

    const IDENTITY: &'static str = "worth.query.certification.prior-cycle-adjustment.v1";
    const HANDLER_IDENTITY: &'static str =
        "worth.query.certification.prior-cycle-adjustment-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.certification.prior-cycle-adjustment-command.v1";
    const CANDIDATES: ApplicationCandidateRequirements = requirements(16);

    fn idempotency_key_identity(key: &u64) -> [u8; 32] {
        super::identity::command_identity(*key)
    }

    fn input_identity(input: &PriorCycleAdjustment) -> [u8; 32] {
        super::identity::input_identity(input)
    }

    fn scope_field() -> ApplicationFieldRef<
        Schema,
        Body,
        crate::PlanarPosition,
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

impl<Schema: TopologySchemaBinding> ApplicationMutationIntent<Schema> for PriorCycleAdjustment {
    type Binding = PriorCycleAdjustmentBinding<Schema>;

    fn input(&self) -> &Self {
        self
    }

    fn scope_binding(&self) -> PlanarMutationScope<Schema> {
        PlanarMutationScope::new(BodyKey::reference(), self.scope_key.clone())
    }
}

pub(super) const fn requirements(writes: usize) -> ApplicationCandidateRequirements {
    ApplicationCandidateRequirements::fixed_shape(
        ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, writes, 0),
        ApplicationCandidateResourceCeiling::bounded(4096, 4096),
    )
}
