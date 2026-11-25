//! Operations, handlers, and producers for Directory Home indexed-selection reopen court.
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
pub struct UpdateStatusInput {
    pub key: String,
    pub status: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Done;
worth_query_structured_value_binding!(pub InitialInputBinding for InitialInput { identity: "indexed-selection-reopen.init-in.v1" });
worth_query_structured_value_binding!(pub PreserveInputBinding for PreserveInput { identity: "indexed-selection-reopen.keep-in.v1" });
worth_query_structured_value_binding!(pub UpdateStatusInputBinding for UpdateStatusInput { identity: "indexed-selection-reopen.upd-in.v1" });
worth_query_structured_value_binding!(pub DoneBinding for Done { identity: "indexed-selection-reopen.done.v1" });
worth_query_operation!(pub InitialProduce for IndexedSelectionSchema, input InitialInputBinding);
worth_query_operation_creates!(InitialProduce => [Account, Derived]);
worth_query_operation!(pub PreserveProduce for IndexedSelectionSchema, input PreserveInputBinding);
worth_query_operation_reads!(PreserveProduce => [Derived]);
worth_query_operation!(pub UpdateStatus for IndexedSelectionSchema, input UpdateStatusInputBinding);

pub struct InitialOutputs;
pub struct BornDerived;
impl WorthQueryApplicationOutputRole for BornDerived {
    type Schema = IndexedSelectionSchema;
    type Contract = InitialOutputs;
    type Entity = Derived;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "derived";
}
impl ApplicationMutationOutputContract<IndexedSelectionSchema> for InitialOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[<BornDerived as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
}
pub struct PreserveOutputs;
pub struct KeptDerived;
impl WorthQueryApplicationOutputRole for KeptDerived {
    type Schema = IndexedSelectionSchema;
    type Contract = PreserveOutputs;
    type Entity = Derived;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "derived";
}
impl ApplicationMutationOutputContract<IndexedSelectionSchema> for PreserveOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[<KeptDerived as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
}

pub type MutScope = ApplicationMutationFieldScope<AccountKey>;
macro_rules! mut_binding {
    ($name:ident, $input:ty, $input_binding:ty, $op:ident, $decision:ty, $output:ty, $source:ty, $id:literal) => {
        pub struct $name;
        impl ApplicationMutationBinding<IndexedSelectionSchema> for $name {
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
            fn scope_field() -> AccountKey {
                AccountKey::at("AccountKey")
            }
            fn principal_binding() -> ApplicationPrincipalBindingRef<
                IndexedSelectionSchema,
                OperatorPrincipalBinding,
                ExternalMapping,
                Principal,
                u64,
                U64ApplicationValueBinding,
            > {
                OperatorPrincipalBinding::reference()
            }
        }
        impl ApplicationMutationIntent<IndexedSelectionSchema> for $input {
            type Binding = $name;
            fn input(&self) -> &Self {
                self
            }
            fn scope_binding(&self) -> MutScope {
                MutScope::new(AccountKey::at("AccountKey"), self.key.clone())
            }
        }
    };
}
mut_binding!(
    InitialBinding,
    InitialInput,
    InitialInputBinding,
    InitialProduce,
    (),
    InitialOutputs,
    ApplicationQueryMutationSource<AccountQuery>,
    "indexed-selection-reopen.initial.v1"
);
mut_binding!(
    PreserveBinding,
    PreserveInput,
    PreserveInputBinding,
    PreserveProduce,
    (
        usize,
        WorthQueryInvariantMutationTarget<IndexedSelectionSchema, Derived>
    ),
    PreserveOutputs,
    ApplicationQueryMutationSource<AccountQuery>,
    "indexed-selection-reopen.preserve.v1"
);
mut_binding!(UpdateStatusBinding, UpdateStatusInput, UpdateStatusInputBinding, UpdateStatus, WorthQueryInvariantMutationTarget<IndexedSelectionSchema, Account>, NoApplicationMutationOutputs, NoApplicationMutationSource, "indexed-selection-reopen.update-status.v1");
impl ApplicationMutationAdmissionBinding<IndexedSelectionSchema> for UpdateStatusBinding {
    type Admission = ApplicationConventionalAdmission;
}

pub struct Handler;
impl OperationHandler<IndexedSelectionSchema, InitialBinding> for Handler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[WorthQueryOperationDurableCapability::RelationalMutation])
    }
    fn decide(
        &self,
        _: &InitialInput,
        reader: &mut DecisionReader<'_, '_, '_, IndexedSelectionSchema, InitialBinding>,
    ) -> HandlerResult<(), Done> {
        // Observe ONLY the indexed equality absence — no direct field read on the
        // member we are about to create — so post-reopen invalidation proves the
        // rebased IndexedEntitySelection fact against the cold-rebuilt index.
        let prepared = match reader.prepare_entity_selection(AccountStatus::at("AccountStatus")) {
            Ok(p) => p,
            Err(e) => return HandlerResult::ExecutionDenied(e),
        };
        match reader.select_entities_prepared(&prepared, "pending-membership".to_owned(), 2) {
            Ok(matches) if matches.is_empty() => HandlerResult::Completed(()),
            Ok(_) => HandlerResult::DomainDenied(Done),
            Err(e) => HandlerResult::ExecutionDenied(e),
        }
    }
    fn build_candidate(
        &self,
        input: &InitialInput,
        (): (),
        writer: &mut CandidateWriter<'_, IndexedSelectionSchema, InitialBinding>,
    ) -> HandlerResult<Done, Done> {
        let res = (|| {
            // Rebase the precommit indexed absence of "pending-membership" onto this
            // newly created member in the same candidate.
            let member = writer.create_entity(
                Account::reference(),
                WorthQueryApplicationEntityKey::new("acct-member").unwrap(),
            )?;
            writer.initialize_field(
                &member,
                AccountKey::at("AccountKey"),
                "acct-member".to_owned(),
            )?;
            writer.initialize_field(
                &member,
                AccountStatus::at("AccountStatus"),
                "pending-membership".to_owned(),
            )?;
            let derived = writer.create_entity(
                Derived::reference(),
                WorthQueryApplicationEntityKey::new(format!("derived:{}", input.key)).unwrap(),
            )?;
            writer.initialize_field(&derived, DerivedKey::at("DerivedKey"), input.key.clone())?;
            writer.initialize_field(
                &derived,
                DerivedMarker::at("DerivedMarker"),
                "pending-membership".to_owned(),
            )?;
            writer.create_output::<BornDerived>(&derived)
        })();
        match res {
            Ok(()) => HandlerResult::Completed(Done),
            Err(e) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(e)),
        }
    }
}
impl OperationHandler<IndexedSelectionSchema, PreserveBinding> for Handler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[WorthQueryOperationDurableCapability::RelationalMutation])
    }
    fn decide(
        &self,
        _: &PreserveInput,
        reader: &mut DecisionReader<'_, '_, '_, IndexedSelectionSchema, PreserveBinding>,
    ) -> HandlerResult<
        (
            usize,
            WorthQueryInvariantMutationTarget<IndexedSelectionSchema, Derived>,
        ),
        Done,
    > {
        let prepared = match reader.prepare_entity_selection(AccountStatus::at("AccountStatus")) {
            Ok(p) => p,
            Err(e) => return HandlerResult::ExecutionDenied(e),
        };
        let count =
            match reader.select_entities_prepared(&prepared, "pending-membership".to_owned(), 2) {
                Ok(matches) => matches.len(),
                Err(e) => return HandlerResult::ExecutionDenied(e),
            };
        let prior = match reader
            .prior_output_preserved_or_created::<PreserveBinding, KeptDerived, InitialBinding, BornDerived>()
        {
            Ok(p) => p,
            Err(e) => return HandlerResult::ExecutionDenied(e),
        };
        if let Err(e) = reader.field(&prior, DerivedMarker::at("DerivedMarker")) {
            return HandlerResult::ExecutionDenied(e);
        }
        let target = match reader.mutation_target(&prior) {
            Ok(t) => t,
            Err(e) => return HandlerResult::ExecutionDenied(e),
        };
        HandlerResult::Completed((count, target))
    }
    fn build_candidate(
        &self,
        _: &PreserveInput,
        (count, target): (
            usize,
            WorthQueryInvariantMutationTarget<IndexedSelectionSchema, Derived>,
        ),
        writer: &mut CandidateWriter<'_, IndexedSelectionSchema, PreserveBinding>,
    ) -> HandlerResult<Done, Done> {
        let res = (|| {
            let derived = writer.projected_entity(&target)?;
            writer.write_field(
                &derived,
                DerivedMarker::at("DerivedMarker"),
                format!("count:{count}"),
            )?;
            writer.preserve_output::<KeptDerived>(&derived)
        })();
        match res {
            Ok(()) => HandlerResult::Completed(Done),
            Err(e) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(e)),
        }
    }
}
impl OperationHandler<IndexedSelectionSchema, UpdateStatusBinding> for Handler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[WorthQueryOperationDurableCapability::RelationalMutation])
    }
    fn decide(
        &self,
        input: &UpdateStatusInput,
        reader: &mut DecisionReader<'_, '_, '_, IndexedSelectionSchema, UpdateStatusBinding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<IndexedSelectionSchema, Account>, Done>
    {
        let account = match reader.resolve_entity(AccountKey::at("AccountKey"), input.key.clone()) {
            Ok(a) => a,
            Err(e) => return HandlerResult::ExecutionDenied(e),
        };
        if let Err(e) = reader.field(&account, AccountStatus::at("AccountStatus")) {
            return HandlerResult::ExecutionDenied(e);
        }
        match reader.mutation_target(&account) {
            Ok(t) => HandlerResult::Completed(t),
            Err(e) => HandlerResult::ExecutionDenied(e),
        }
    }
    fn build_candidate(
        &self,
        input: &UpdateStatusInput,
        target: WorthQueryInvariantMutationTarget<IndexedSelectionSchema, Account>,
        writer: &mut CandidateWriter<'_, IndexedSelectionSchema, UpdateStatusBinding>,
    ) -> HandlerResult<Done, Done> {
        let res = (|| {
            let account = writer.projected_entity(&target)?;
            writer.write_field(
                &account,
                AccountStatus::at("AccountStatus"),
                input.status.clone(),
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
    WorthQueryProducerApplicability::new("indexed", WorthQueryProducerLifecyclePosture::Initial);
pub const KEEP_APP: WorthQueryProducerApplicability =
    WorthQueryProducerApplicability::new("indexed", WorthQueryProducerLifecyclePosture::Preserve);
impl WorthQueryProducerOutputFamily<IndexedSelectionSchema> for Family {
    type Source = AccountReadBinding;
    type Entity = Derived;
    const IDENTITY: &'static str = "indexed-selection-reopen.family.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = &[INIT_APP, KEEP_APP];
    fn profile_kind(_: &KeyRow) -> &'static str {
        "indexed"
    }
}
#[derive(Clone)]
pub struct Demand(pub String);
impl WorthQueryApplicationOutputDemand<IndexedSelectionSchema> for Demand {
    type OutputFamily = Family;
    fn source_intent(&self) -> ReadAccount {
        ReadAccount {
            key: self.0.clone(),
        }
    }
}
pub struct InitialProducer;
pub struct PreserveProducer;
pub struct Provider;
impl WorthQueryApplicationProducerBinding<IndexedSelectionSchema> for InitialProducer {
    type Operation = InitialBinding;
    type OutputFamily = Family;
    type Provider = Provider;
    type OutputRole = BornDerived;
    const IDENTITY: &'static str = "indexed-selection-reopen.initial-producer.v1";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = &[INIT_APP];
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
    const REUSE_POLICY: &'static str = "exact-source";
}
impl WorthQueryApplicationProducerProvider<IndexedSelectionSchema, InitialProducer> for Provider {
    type DenialValue = WorthQueryUndeclaredProducerDenial;
    const SEMANTIC_IDENTITY: &'static str = "indexed-selection-reopen.initial-provider.v1";
    fn operation_input(&self, row: &KeyRow) -> InitialInput {
        InitialInput { key: row.0.clone() }
    }
    fn idempotency_key(&self, _: &KeyRow, identity: &[u8; 32]) -> u64 {
        u64::from_le_bytes(identity[..8].try_into().unwrap())
    }
}
impl WorthQueryApplicationProducerBinding<IndexedSelectionSchema> for PreserveProducer {
    type Operation = PreserveBinding;
    type OutputFamily = Family;
    type Provider = Provider;
    type OutputRole = KeptDerived;
    const IDENTITY: &'static str = "indexed-selection-reopen.preserve-producer.v1";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = &[KEEP_APP];
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
    const REUSE_POLICY: &'static str = "exact-source";
}
impl WorthQueryApplicationProducerProvider<IndexedSelectionSchema, PreserveProducer> for Provider {
    type DenialValue = WorthQueryUndeclaredProducerDenial;
    const SEMANTIC_IDENTITY: &'static str = "indexed-selection-reopen.preserve-provider.v1";
    fn operation_input(&self, row: &KeyRow) -> PreserveInput {
        PreserveInput { key: row.0.clone() }
    }
    fn idempotency_key(&self, _: &KeyRow, identity: &[u8; 32]) -> u64 {
        u64::from_le_bytes(identity[..8].try_into().unwrap())
    }
}
