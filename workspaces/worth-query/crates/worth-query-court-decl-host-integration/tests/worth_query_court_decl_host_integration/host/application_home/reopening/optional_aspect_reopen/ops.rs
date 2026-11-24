//! Operations, handlers, and producers for Directory Home optional-aspect reopen courts.
use super::schema::*;
use worth_query_decl::facade::{
    application_operation::*, application_schema::*, worth_query_operation_creates,
    worth_query_operation_reads,
};
use worth_query_host::facade::{application_contribution::*, primary_graph::*, *};

#[derive(Clone, Debug, serde::Serialize)]
pub struct InitialInput {
    pub key: String,
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct PreserveInput {
    pub key: String,
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct MaterializeInput {
    pub key: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Done;
worth_query_structured_value_binding!(pub InitialInputBinding for InitialInput { identity: "optional-aspect-reopen.init-in.v1" });
worth_query_structured_value_binding!(pub PreserveInputBinding for PreserveInput { identity: "optional-aspect-reopen.keep-in.v1" });
worth_query_structured_value_binding!(pub MaterializeInputBinding for MaterializeInput { identity: "optional-aspect-reopen.mat-in.v1" });
worth_query_structured_value_binding!(pub DoneBinding for Done { identity: "optional-aspect-reopen.done.v1" });
worth_query_operation!(pub InitialProduce for OptionalAspectSchema, input InitialInputBinding);
worth_query_operation_creates!(InitialProduce => [Derived]);
worth_query_operation!(pub PreserveProduce for OptionalAspectSchema, input PreserveInputBinding);
worth_query_operation_reads!(PreserveProduce => [Derived]);
worth_query_operation!(pub MaterializeOther for OptionalAspectSchema, input MaterializeInputBinding);

pub struct InitialOutputs;
pub struct BornDerived;
impl WorthQueryApplicationOutputRole for BornDerived {
    type Schema = OptionalAspectSchema;
    type Contract = InitialOutputs;
    type Entity = Derived;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "derived";
}
impl ApplicationMutationOutputContract<OptionalAspectSchema> for InitialOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[<BornDerived as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
}
pub struct PreserveOutputs;
pub struct KeptDerived;
impl WorthQueryApplicationOutputRole for KeptDerived {
    type Schema = OptionalAspectSchema;
    type Contract = PreserveOutputs;
    type Entity = Derived;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "derived";
}
impl ApplicationMutationOutputContract<OptionalAspectSchema> for PreserveOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[<KeptDerived as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
}

pub type MutScope = ApplicationMutationFieldScope<AnchorKey>;
macro_rules! mut_binding {
    ($name:ident, $input:ty, $input_binding:ty, $op:ident, $decision:ty, $output:ty, $source:ty, $id:literal) => {
        pub struct $name;
        impl ApplicationMutationBinding<OptionalAspectSchema> for $name {
            type Input = $input;
            type InputBinding = $input_binding;
            type Result = Done;
            type ResultBinding = DoneBinding;
            type IdempotencyKey = u64;
            type Operation = $op;
            type Decision = $decision;
            type Denial = Done;
            type DenialBinding = DoneBinding;
            type Output = $output;
            type ScopeBinding = MutScope;
            type PrincipalBinding = OperatorPrincipalBinding;
            type Mapping = ExternalMapping;
            type Principal = Principal;
            type PrincipalIdentity = u64;
            type PrincipalIdentityBinding = U64ApplicationValueBinding;
            type SourceExpectation = $source;
            const IDENTITY: &'static str = $id;
            const HANDLER_IDENTITY: &'static str = $id;
            const IDEMPOTENCY_IDENTITY: &'static str = $id;
            fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
                ApplicationWorkflowExecutionRequirements::not_applicable()
            }
            fn scope_field() -> AnchorKey {
                AnchorKey::at("AnchorKey")
            }
            fn principal_binding() -> ApplicationPrincipalBindingRef<
                OptionalAspectSchema,
                OperatorPrincipalBinding,
                ExternalMapping,
                Principal,
                u64,
                U64ApplicationValueBinding,
            > {
                OperatorPrincipalBinding::reference()
            }
        }
        impl ApplicationMutationIntent<OptionalAspectSchema> for $input {
            type Binding = $name;
            fn input(&self) -> &Self {
                self
            }
            fn scope_binding(&self) -> MutScope {
                MutScope::new(AnchorKey::at("AnchorKey"), self.key.clone())
            }
        }
    };
}
mut_binding!(
    InitialBinding,
    InitialInput,
    InitialInputBinding,
    InitialProduce,
    String,
    InitialOutputs,
    ApplicationQueryMutationSource<AnchorQuery>,
    "optional-aspect-reopen.initial.v1"
);
mut_binding!(
    PreserveBinding,
    PreserveInput,
    PreserveInputBinding,
    PreserveProduce,
    (
        String,
        WorthQueryInvariantMutationTarget<OptionalAspectSchema, Derived>
    ),
    PreserveOutputs,
    ApplicationQueryMutationSource<AnchorQuery>,
    "optional-aspect-reopen.preserve.v1"
);
mut_binding!(MaterializeBinding, MaterializeInput, MaterializeInputBinding, MaterializeOther, WorthQueryInvariantMutationTarget<OptionalAspectSchema, Anchor>, NoApplicationMutationOutputs, NoApplicationMutationSource, "optional-aspect-reopen.materialize.v1");
impl ApplicationMutationAdmissionBinding<OptionalAspectSchema> for MaterializeBinding {
    type Admission = ApplicationConventionalAdmission;
}

pub struct Handler;
impl OperationHandler<OptionalAspectSchema, InitialBinding> for Handler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[WorthQueryOperationDurableCapability::RelationalMutation])
    }
    fn decide(
        &self,
        input: &InitialInput,
        reader: &mut DecisionReader<'_, '_, '_, OptionalAspectSchema, InitialBinding>,
    ) -> HandlerResult<String, Done> {
        let anchor = match reader.resolve_entity(AnchorKey::at("AnchorKey"), input.key.clone()) {
            Ok(anchor) => anchor,
            Err(e) => return HandlerResult::ExecutionDenied(e),
        };
        let selection = match reader.field(&anchor, SourceSelection::at("SourceSelection")) {
            Ok(value) => value,
            Err(e) => return HandlerResult::ExecutionDenied(e),
        };
        HandlerResult::Completed(selection.unwrap_or_else(|| "absent".to_owned()))
    }
    fn build_candidate(
        &self,
        input: &InitialInput,
        tag: String,
        writer: &mut CandidateWriter<'_, OptionalAspectSchema, InitialBinding>,
    ) -> HandlerResult<Done, Done> {
        let res = (|| {
            let derived = writer.create_entity(
                Derived::reference(),
                WorthQueryApplicationEntityKey::new(format!("derived:{}", input.key)).unwrap(),
            )?;
            writer.initialize_field(&derived, DerivedKey::at("DerivedKey"), input.key.clone())?;
            writer.initialize_field(&derived, DerivedTag::at("DerivedTag"), tag)?;
            writer.create_output::<BornDerived>(&derived)
        })();
        match res {
            Ok(()) => HandlerResult::Completed(Done),
            Err(e) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(e)),
        }
    }
}
impl OperationHandler<OptionalAspectSchema, PreserveBinding> for Handler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[WorthQueryOperationDurableCapability::RelationalMutation])
    }
    fn decide(
        &self,
        input: &PreserveInput,
        reader: &mut DecisionReader<'_, '_, '_, OptionalAspectSchema, PreserveBinding>,
    ) -> HandlerResult<
        (
            String,
            WorthQueryInvariantMutationTarget<OptionalAspectSchema, Derived>,
        ),
        Done,
    > {
        let anchor = match reader.resolve_entity(AnchorKey::at("AnchorKey"), input.key.clone()) {
            Ok(anchor) => anchor,
            Err(e) => return HandlerResult::ExecutionDenied(e),
        };
        let selection = match reader.field(&anchor, SourceSelection::at("SourceSelection")) {
            Ok(value) => value,
            Err(e) => return HandlerResult::ExecutionDenied(e),
        };
        let prior = match reader
            .prior_output_preserved_or_created::<PreserveBinding, KeptDerived, InitialBinding, BornDerived>()
        {
            Ok(prior) => prior,
            Err(e) => return HandlerResult::ExecutionDenied(e),
        };
        if let Err(e) = reader.field(&prior, DerivedTag::at("DerivedTag")) {
            return HandlerResult::ExecutionDenied(e);
        }
        let target = match reader.mutation_target(&prior) {
            Ok(t) => t,
            Err(e) => return HandlerResult::ExecutionDenied(e),
        };
        HandlerResult::Completed((selection.unwrap_or_else(|| "absent".to_owned()), target))
    }
    fn build_candidate(
        &self,
        _: &PreserveInput,
        (tag, target): (
            String,
            WorthQueryInvariantMutationTarget<OptionalAspectSchema, Derived>,
        ),
        writer: &mut CandidateWriter<'_, OptionalAspectSchema, PreserveBinding>,
    ) -> HandlerResult<Done, Done> {
        let res = (|| {
            let derived = writer.projected_entity(&target)?;
            writer.write_field(&derived, DerivedTag::at("DerivedTag"), tag)?;
            writer.preserve_output::<KeptDerived>(&derived)
        })();
        match res {
            Ok(()) => HandlerResult::Completed(Done),
            Err(e) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(e)),
        }
    }
}
impl OperationHandler<OptionalAspectSchema, MaterializeBinding> for Handler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[WorthQueryOperationDurableCapability::RelationalMutation])
    }
    fn decide(
        &self,
        input: &MaterializeInput,
        reader: &mut DecisionReader<'_, '_, '_, OptionalAspectSchema, MaterializeBinding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<OptionalAspectSchema, Anchor>, Done> {
        let anchor = match reader.resolve_entity(AnchorKey::at("AnchorKey"), input.key.clone()) {
            Ok(anchor) => anchor,
            Err(e) => return HandlerResult::ExecutionDenied(e),
        };
        if let Err(e) = reader.field(&anchor, SourceContent::at("SourceContent")) {
            return HandlerResult::ExecutionDenied(e);
        }
        match reader.mutation_target(&anchor) {
            Ok(target) => HandlerResult::Completed(target),
            Err(e) => HandlerResult::ExecutionDenied(e),
        }
    }
    fn build_candidate(
        &self,
        _: &MaterializeInput,
        target: WorthQueryInvariantMutationTarget<OptionalAspectSchema, Anchor>,
        writer: &mut CandidateWriter<'_, OptionalAspectSchema, MaterializeBinding>,
    ) -> HandlerResult<Done, Done> {
        let res = (|| {
            let anchor = writer.projected_entity(&target)?;
            // Write ONLY SourceContent (the OTHER field of OptionalSource), leaving
            // SourceSelection unset so invalidation must come from whole-aspect presence.
            writer.write_field(
                &anchor,
                SourceContent::at("SourceContent"),
                "complete-required-content".to_owned(),
            )
        })();
        match res {
            Ok(()) => HandlerResult::Completed(Done),
            Err(e) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(e)),
        }
    }
}

pub struct Family;
pub const INIT_APP: WorthQueryProducerApplicability =
    WorthQueryProducerApplicability::new("optional", WorthQueryProducerLifecyclePosture::Initial);
pub const KEEP_APP: WorthQueryProducerApplicability =
    WorthQueryProducerApplicability::new("optional", WorthQueryProducerLifecyclePosture::Preserve);
impl WorthQueryProducerOutputFamily<OptionalAspectSchema> for Family {
    type Source = AnchorReadBinding;
    type Entity = Derived;
    const IDENTITY: &'static str = "optional-aspect-reopen.family.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = &[INIT_APP, KEEP_APP];
    fn profile_kind(_: &KeyRow) -> &'static str {
        "optional"
    }
}
#[derive(Clone)]
pub struct Demand(pub String);
impl WorthQueryApplicationOutputDemand<OptionalAspectSchema> for Demand {
    type OutputFamily = Family;
    fn source_intent(&self) -> ReadAnchor {
        ReadAnchor {
            key: self.0.clone(),
        }
    }
}
pub struct InitialProducer;
pub struct PreserveProducer;
pub struct Provider;
impl WorthQueryApplicationProducerBinding<OptionalAspectSchema> for InitialProducer {
    type Operation = InitialBinding;
    type OutputFamily = Family;
    type Provider = Provider;
    type OutputRole = BornDerived;
    const IDENTITY: &'static str = "optional-aspect-reopen.initial-producer.v1";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = &[INIT_APP];
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
    const REUSE_POLICY: &'static str = "exact-source";
}
impl WorthQueryApplicationProducerProvider<OptionalAspectSchema, InitialProducer> for Provider {
    type DenialValue = WorthQueryUndeclaredProducerDenial;
    const SEMANTIC_IDENTITY: &'static str = "optional-aspect-reopen.initial-provider.v1";
    fn operation_input(&self, row: &KeyRow) -> InitialInput {
        InitialInput { key: row.0.clone() }
    }
    fn idempotency_key(&self, _: &KeyRow, identity: &[u8; 32]) -> u64 {
        u64::from_le_bytes(identity[..8].try_into().unwrap())
    }
}
impl WorthQueryApplicationProducerBinding<OptionalAspectSchema> for PreserveProducer {
    type Operation = PreserveBinding;
    type OutputFamily = Family;
    type Provider = Provider;
    type OutputRole = KeptDerived;
    const IDENTITY: &'static str = "optional-aspect-reopen.preserve-producer.v1";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = &[KEEP_APP];
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
    const REUSE_POLICY: &'static str = "exact-source";
}
impl WorthQueryApplicationProducerProvider<OptionalAspectSchema, PreserveProducer> for Provider {
    type DenialValue = WorthQueryUndeclaredProducerDenial;
    const SEMANTIC_IDENTITY: &'static str = "optional-aspect-reopen.preserve-provider.v1";
    fn operation_input(&self, row: &KeyRow) -> PreserveInput {
        PreserveInput { key: row.0.clone() }
    }
    fn idempotency_key(&self, _: &KeyRow, identity: &[u8; 32]) -> u64 {
        u64::from_le_bytes(identity[..8].try_into().unwrap())
    }
}
