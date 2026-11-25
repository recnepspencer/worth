//! Minimal generated-output operation, handler, and producer bindings.
use super::*;
use worth_query_decl::facade::application_operation::*;
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::{
    worth_query_operation, worth_query_operation_creates, worth_query_operation_reads,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::application_contribution::*;
use worth_query_host::facade::primary_graph::*;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Generate {
    pub payload: i64,
}
#[derive(Debug)]
pub struct Generated;
#[derive(Debug)]
pub enum Denial {}
pub struct Decision {
    root: WorthQueryInvariantMutationTarget<Catalog, Principal>,
}
worth_query_structured_value_binding!(pub GenerateInputBinding for Generate { identity: "worth.query.early-open-recovery.generate-input.v1" });
worth_query_structured_value_binding!(pub GenerateResultBinding for Generated { identity: "worth.query.early-open-recovery.generate-result.v1" });
worth_query_structured_value_binding!(pub GenerateDenialBinding for Denial { identity: "worth.query.early-open-recovery.generate-denial.v1" });
worth_query_operation!(pub GenerateArtifact for Catalog, input GenerateInputBinding);
worth_query_operation_reads!(GenerateArtifact => [Principal]);
worth_query_operation_creates!(GenerateArtifact => [Artifact]);

pub struct Outputs;
pub struct ArtifactOutput;
impl WorthQueryApplicationOutputRole for ArtifactOutput {
    type Schema = Catalog;
    type Contract = Outputs;
    type Entity = Artifact;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "artifact";
}
impl ApplicationMutationOutputContract<Catalog> for Outputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[<ArtifactOutput as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
}

pub struct GenerateBinding;
impl ApplicationMutationAdmissionBinding<Catalog> for GenerateBinding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<Catalog> for GenerateBinding {
    type Input = Generate;
    type InputBinding = GenerateInputBinding;
    type Result = Generated;
    type ResultBinding = GenerateResultBinding;
    type IdempotencyKey = u64;
    type Operation = GenerateArtifact;
    type Decision = Decision;
    type Denial = Denial;
    type DenialBinding = GenerateDenialBinding;
    type Output = Outputs;
    type ScopeBinding = ApplicationMutationPrincipalScope<PrincipalValue>;
    type PrincipalBinding = PrincipalBinding;
    type Mapping = Mapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<SourceQuery>;
    const IDENTITY: &'static str = "worth.query.early-open-recovery.generate-binding.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.early-open-recovery.generate-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.early-open-recovery.generate-idempotency.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = false;
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> PrincipalValue {
        PrincipalValue::at("Value")
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Catalog,
        PrincipalBinding,
        Mapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        principal_binding()
    }
}

pub struct GenerateHandler;
impl ApplicationMutationIntent<Catalog> for Generate {
    type Binding = GenerateBinding;
    fn input(&self) -> &Self {
        self
    }
    fn scope_binding(
        &self,
    ) -> <GenerateBinding as ApplicationMutationBinding<Catalog>>::ScopeBinding {
        ApplicationMutationPrincipalScope::new(PrincipalValue::at("Value"))
    }
}
impl OperationHandler<Catalog, GenerateBinding> for GenerateHandler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[WorthQueryOperationDurableCapability::RelationalMutation])
    }
    fn decide(
        &self,
        _input: &Generate,
        reader: &mut DecisionReader<'_, '_, '_, Catalog, GenerateBinding>,
    ) -> HandlerResult<Decision, Denial> {
        HandlerResult::from_result((|| {
            let root = reader.resolve_entity(PrincipalValue::at("Value"), 1u64)?;
            Ok(Decision {
                root: reader.mutation_target(&root)?,
            })
        })())
    }
    fn build_candidate(
        &self,
        input: &Generate,
        decision: Decision,
        writer: &mut CandidateWriter<'_, Catalog, GenerateBinding>,
    ) -> HandlerResult<Generated, Denial> {
        let result = (|| {
            let root = writer
                .projected_entity(&decision.root)
                .map_err(HandlerExecutionDenial::new)?;
            let artifact = writer
                .create_entity_in_context(
                    &root,
                    Artifact::reference(),
                    WorthQueryApplicationEntityKey::new("generated-artifact").unwrap(),
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&artifact, ArtifactPayload::at("Payload"), input.payload)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .create_output::<ArtifactOutput>(&artifact)
                .map_err(HandlerExecutionDenial::new)?;
            Ok(Generated)
        })();
        match result {
            Ok(result) => HandlerResult::Completed(result),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}

pub struct OutputFamily;
const INITIAL: WorthQueryProducerApplicability = WorthQueryProducerApplicability::new(
    "artifact",
    WorthQueryProducerLifecyclePosture::Initial,
);
impl WorthQueryProducerOutputFamily<Catalog> for OutputFamily {
    type Source = SourceBinding;
    type Entity = Artifact;
    const IDENTITY: &'static str = "worth.query.early-open-recovery.family.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = &[INITIAL];
    fn profile_kind(_: &SourceRow) -> &'static str {
        "artifact"
    }
}
pub struct OutputProvider;
pub struct OutputProducer;
impl WorthQueryApplicationProducerProvider<Catalog, OutputProducer> for OutputProvider {
    type DenialValue = WorthQueryUndeclaredProducerDenial;
    const SEMANTIC_IDENTITY: &'static str = "worth.query.early-open-recovery.provider.v1";
    fn operation_input(&self, source: &SourceRow) -> Generate {
        Generate {
            payload: (source.value * 42) as i64,
        }
    }
    fn idempotency_key(&self, _: &SourceRow, source_identity: &[u8; 32]) -> u64 {
        u64::from_le_bytes(source_identity[..8].try_into().unwrap())
    }
}
impl WorthQueryApplicationProducerBinding<Catalog> for OutputProducer {
    type Operation = GenerateBinding;
    type OutputFamily = OutputFamily;
    type Provider = OutputProvider;
    type OutputRole = ArtifactOutput;
    const IDENTITY: &'static str = "worth.query.early-open-recovery.producer.v1";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = &[INITIAL];
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
    const REUSE_POLICY: &'static str = "exact-source";
    const INPUT_REUSE: Option<WorthQueryProducerInputReuseContract> =
        Some(WorthQueryProducerInputReuseContract::canonical_bitwise(
            WorthQueryDecisionContextDependencies::NONE,
        ));
}
