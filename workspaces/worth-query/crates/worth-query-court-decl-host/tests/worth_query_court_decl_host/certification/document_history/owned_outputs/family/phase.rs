//! Optional predecessor observations and lifecycle bindings stay at their fixture owner.
#[macro_use]
mod members;
macro_rules! producer_capture_contract {
    () => {
        None
    };
}
macro_rules! producer_program_required {
    () => {
        false
    };
}
macro_rules! producer_input_reuse_contract {
    () => {
        None
    };
}
macro_rules! producer_capture_handler {
    () => {};
}
macro_rules! producer_operation {
    (FirstInitial, $entity:ident, $key:ident, $value:ident, $of:ident) => { member_input_operation!(FirstInitial, $entity, $key, $value, $of); };
    (FirstPreserve, $entity:ident, $key:ident, $value:ident, $of:ident) => { member_input_operation!(FirstPreserve, $entity, $key, $value, $of); };
    (ConvergentPreserve, $entity:ident, $key:ident, $value:ident, $of:ident) => {
        worth_query_operation!(pub ConvergentPreserve for Schema, input InputBinding; access {
            read_entities: [SourceEntity, $entity, Member]; read_fields: [source_key(), source_value(), $key::at(stringify!($key)), $value::at(stringify!($value)), MemberKey::at("MemberKey"), CalibrationReady::at("Ready")];
            read_relations: [$of]; writes: [$key::at(stringify!($key)), $value::at(stringify!($value)), CalibrationReady::at("Ready")]; creates: [$entity]; links: [$of];
        });
    };
    (OrdinaryInputInitial, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@member_read OrdinaryInputInitial, $entity, $key, $value, $of); };
    (OrdinaryInputPreserve, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@member_read OrdinaryInputPreserve, $entity, $key, $value, $of); };
    (DecisionInputInitial, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@member_read DecisionInputInitial, $entity, $key, $value, $of); };
    (DecisionInputPreserve, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@member_read DecisionInputPreserve, $entity, $key, $value, $of); };
    (ParityInputInitial, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@member_read ParityInputInitial, $entity, $key, $value, $of); };
    (ParityInputPreserve, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@member_read ParityInputPreserve, $entity, $key, $value, $of); };
    (SaturatedInputInitial, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@member_read SaturatedInputInitial, $entity, $key, $value, $of); };
    (SaturatedInputPreserve, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@member_read SaturatedInputPreserve, $entity, $key, $value, $of); };
    (DecisionWriteInitial, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@member_write DecisionWriteInitial, $entity, $key, $value, $of); };
    (DecisionWritePreserve, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@member_write DecisionWritePreserve, $entity, $key, $value, $of); };
    (CrossReaderInitial, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@member_read CrossReaderInitial, $entity, $key, $value, $of); };
    (CrossReaderPreserve, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@member_read CrossReaderPreserve, $entity, $key, $value, $of); };
    (CrossConvergeInitial, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@member_write CrossConvergeInitial, $entity, $key, $value, $of); };
    (CrossConvergePreserve, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@member_write CrossConvergePreserve, $entity, $key, $value, $of); };
    (CrossOscillateInitial, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@member_write CrossOscillateInitial, $entity, $key, $value, $of); };
    (CrossOscillatePreserve, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@member_write CrossOscillatePreserve, $entity, $key, $value, $of); };
    (SelfWriteInitial, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@self_write SelfWriteInitial, $entity, $key, $value, $of); };
    (SelfWritePreserve, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@self_write SelfWritePreserve, $entity, $key, $value, $of); };
    // The source is the member it writes: each record and field once.
    (@self_write $operation:ident, $entity:ident, $key:ident, $value:ident, $of:ident) => {
        worth_query_operation!(pub $operation for Schema, input InputBinding; access {
            read_entities: [Member, $entity]; read_fields: [MemberKey::at("MemberKey"), MemberValue::at("MemberValue"), $key::at(stringify!($key)), $value::at(stringify!($value))];
            read_relations: [$of]; writes: [$key::at(stringify!($key)), $value::at(stringify!($value)), MemberValue::at("MemberValue")]; creates: [$entity]; links: [$of];
        });
    };
    (GuardedCurrentInitial, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@member_read GuardedCurrentInitial, $entity, $key, $value, $of); };
    (GuardedCurrentPreserve, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@member_read GuardedCurrentPreserve, $entity, $key, $value, $of); };
    (SaturatedConsumerInitial, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@member_read SaturatedConsumerInitial, $entity, $key, $value, $of); };
    (SaturatedConsumerPreserve, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@member_read SaturatedConsumerPreserve, $entity, $key, $value, $of); };
    (CappedReaderInitial, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@member_read CappedReaderInitial, $entity, $key, $value, $of); };
    (CappedReaderPreserve, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@member_read CappedReaderPreserve, $entity, $key, $value, $of); };
    (CappedConsumerInitial, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@member_read CappedConsumerInitial, $entity, $key, $value, $of); };
    (CappedConsumerPreserve, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@member_read CappedConsumerPreserve, $entity, $key, $value, $of); };
    (CycleParentInitial, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@member_read CycleParentInitial, $entity, $key, $value, $of); };
    (CycleParentPreserve, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@member_read CycleParentPreserve, $entity, $key, $value, $of); };
    (CycleConsumerInitial, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@member_read CycleConsumerInitial, $entity, $key, $value, $of); };
    (CycleConsumerPreserve, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@member_read CycleConsumerPreserve, $entity, $key, $value, $of); };
    (GuardedInputInitial, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@member_read GuardedInputInitial, $entity, $key, $value, $of); };
    (GuardedInputPreserve, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@member_read GuardedInputPreserve, $entity, $key, $value, $of); };
    (GuardedMemberInitial, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@member_read GuardedMemberInitial, $entity, $key, $value, $of); };
    (GuardedMemberPreserve, $entity:ident, $key:ident, $value:ident, $of:ident) => { producer_operation!(@member_read GuardedMemberPreserve, $entity, $key, $value, $of); };
    (@member_write $operation:ident, $entity:ident, $key:ident, $value:ident, $of:ident) => {
        worth_query_operation!(pub $operation for Schema, input InputBinding; access {
            read_entities: [SourceEntity, $entity, Member]; read_fields: [source_key(), source_value(), $key::at(stringify!($key)), $value::at(stringify!($value)), MemberKey::at("MemberKey"), MemberValue::at("MemberValue")];
            read_relations: [$of]; writes: [$key::at(stringify!($key)), $value::at(stringify!($value)), MemberValue::at("MemberValue")]; creates: [$entity]; links: [$of];
        });
    };
    (@member_read $operation:ident, $entity:ident, $key:ident, $value:ident, $of:ident) => {
        worth_query_operation!(pub $operation for Schema, input InputBinding; access {
            read_entities: [SourceEntity, $entity, Member]; read_fields: [source_key(), source_value(), $key::at(stringify!($key)), $value::at(stringify!($value)), MemberKey::at("MemberKey"), MemberValue::at("MemberValue")];
            read_relations: [$of]; writes: [$key::at(stringify!($key)), $value::at(stringify!($value))]; creates: [$entity]; links: [$of];
        });
    };
    ($operation:ident, $entity:ident, $key:ident, $value:ident, $of:ident) => {
            worth_query_operation!(pub $operation for Schema, input InputBinding; access {
                read_entities: [SourceEntity, $entity]; read_fields: [source_key(), source_value(), $key::at(stringify!($key)), $value::at(stringify!($value))];
                read_relations: [$of]; writes: [$key::at(stringify!($key)), $value::at(stringify!($value))]; creates: [$entity]; links: [$of];
            });
    };
}
macro_rules! producer_decision_dependency {
    (CycleParentInitial, $reader:ident, $source:ident, $input:ident) => { producer_decision_dependency!(@cycle_parent $reader) };
    (CycleConsumerInitial, $reader:ident, $source:ident, $input:ident) => {{
        let model = $reader.resolve_entity(ModelKey::at("ModelKey"), "a".to_owned())?;
        producer_decision_dependency!(@current_output $reader, model, super::super::cycle_parent::Family)
    }};
    (CycleParentPreserve, $reader:ident, $source:ident, $input:ident) => { producer_decision_dependency!(@cycle_parent $reader) };
    (CycleConsumerPreserve, $reader:ident, $source:ident, $input:ident) => {{
        let model = $reader.resolve_entity(ModelKey::at("ModelKey"), "a".to_owned())?;
        producer_decision_dependency!(@current_output $reader, model, super::super::cycle_parent::Family)
    }};
    // min(m, 5); once m > 5, also read consumer `cycle-consumer(c)`.
    (@cycle_parent $reader:ident) => {{
        let member = producer_decision_dependency!(@member_read $reader);
        if member > 5 {
            let model = $reader.resolve_entity(ModelKey::at("ModelKey"), "c".to_owned())?;
            producer_decision_dependency!(@current_output $reader, model, super::super::cycle_consumer::Family);
        }
        member.min(5)
    }};
    (GuardedCurrentInitial, $reader:ident, $source:ident, $input:ident) => { producer_decision_dependency!(@saturated_current $reader) };
    (GuardedCurrentPreserve, $reader:ident, $source:ident, $input:ident) => { producer_decision_dependency!(@saturated_current $reader) };
    // Whether `saturated-input(a)` publishes a current output for model `a`.
    (@saturated_current $reader:ident) => {{
        let model = $reader.resolve_entity(ModelKey::at("ModelKey"), "a".to_owned())?;
        match $reader.current_output::<super::super::saturated_input::Family, Model>(&model)? {
            WorthQueryCurrentOutputSelection::Unique(_) => 1,
            _ => 0,
        }
    }};
    (SaturatedConsumerInitial, $reader:ident, $source:ident, $input:ident) => {{
        let model = $reader.resolve_entity(ModelKey::at("ModelKey"), "a".to_owned())?;
        producer_decision_dependency!(@current_output $reader, model, super::super::saturated_input::Family)
    }};
    (CappedReaderInitial, $reader:ident, $source:ident, $input:ident) => { producer_decision_dependency!(@cross_member $reader, $input).min(1) };
    (CappedConsumerInitial, $reader:ident, $source:ident, $input:ident) => { producer_decision_dependency!(@current_output $reader, $source, super::super::capped_reader::Family) };
    (SaturatedConsumerPreserve, $reader:ident, $source:ident, $input:ident) => {{
        let model = $reader.resolve_entity(ModelKey::at("ModelKey"), "a".to_owned())?;
        producer_decision_dependency!(@current_output $reader, model, super::super::saturated_input::Family)
    }};
    (CappedReaderPreserve, $reader:ident, $source:ident, $input:ident) => { producer_decision_dependency!(@cross_member $reader, $input).min(1) };
    (CappedConsumerPreserve, $reader:ident, $source:ident, $input:ident) => { producer_decision_dependency!(@current_output $reader, $source, super::super::capped_reader::Family) };
    // The value of the output `$family` currently publishes for `$producer`.
    (@current_output $reader:ident, $producer:ident, $family:ty) => {{
        match $reader.current_output::<$family, Model>(&$producer)? {
            WorthQueryCurrentOutputSelection::Unique(output) => $reader
                .field(&output, FirstValue::at("FirstValue"))?
                .expect("current output has its value"),
            _ => panic!("the consumed output is current and unique"),
        }
    }};
    (SelfWriteInitial, $reader:ident, $source:ident, $input:ident) => {{
        let first = $reader.resolve_entity(FirstKey::at("FirstKey"), "first:a".to_owned())?;
        $reader.field(&first, FirstValue::at("FirstValue"))?.expect("derived first:a has its value")
    }};
    (SelfWritePreserve, $reader:ident, $source:ident, $input:ident) => {{
        let first = $reader.resolve_entity(FirstKey::at("FirstKey"), "first:a".to_owned())?;
        $reader.field(&first, FirstValue::at("FirstValue"))?.expect("derived first:a has its value")
    }};
    (CrossReaderInitial, $reader:ident, $source:ident, $input:ident) => { producer_decision_dependency!(@cross_member $reader, $input) };
    (CrossReaderPreserve, $reader:ident, $source:ident, $input:ident) => { producer_decision_dependency!(@cross_member $reader, $input) };
    (CrossConvergeInitial, $reader:ident, $source:ident, $input:ident) => { producer_decision_dependency!(@cross_supplier $reader, $input) };
    (CrossConvergePreserve, $reader:ident, $source:ident, $input:ident) => { producer_decision_dependency!(@cross_supplier $reader, $input) };
    (CrossOscillateInitial, $reader:ident, $source:ident, $input:ident) => { producer_decision_dependency!(@cross_supplier $reader, $input) };
    (CrossOscillatePreserve, $reader:ident, $source:ident, $input:ident) => { producer_decision_dependency!(@cross_supplier $reader, $input) };
    (@cross_member $reader:ident, $input:ident) => {{
        let member = $reader.resolve_entity(MemberKey::at("MemberKey"), format!("x:{}", $input.0.key))?;
        $reader.field(&member, MemberValue::at("MemberValue"))?.expect("seeded cross member has its value")
    }};
    (@cross_supplier $reader:ident, $input:ident) => {{
        let supplier = $reader.resolve_entity(FirstKey::at("FirstKey"), format!("cross:{}", $input.0.key))?;
        $reader.field(&supplier, FirstValue::at("FirstValue"))?.expect("derived cross reader has its value")
    }};
    // Every other family reads no demand key.
    ($operation:ident, $reader:ident, $source:ident, $input:ident) => { producer_decision_dependency!($operation, $reader, $source) };
    (ConvergentPreserve, $reader:ident, $source:ident) => {{
        let member = $reader.resolve_entity(MemberKey::at("MemberKey"), "m".to_owned())?;
        $reader.field(&member, CalibrationReady::at("Ready"))?.unwrap_or(0)
    }};
    (OrdinaryInputInitial, $reader:ident, $source:ident) => { producer_decision_dependency!(@member_read $reader) };
    (OrdinaryInputPreserve, $reader:ident, $source:ident) => { producer_decision_dependency!(@member_read $reader) };
    (DecisionInputInitial, $reader:ident, $source:ident) => { producer_decision_dependency!(@member_read $reader) };
    (DecisionInputPreserve, $reader:ident, $source:ident) => { producer_decision_dependency!(@member_read $reader) };
    (ParityInputInitial, $reader:ident, $source:ident) => { producer_decision_dependency!(@member_read $reader) };
    (ParityInputPreserve, $reader:ident, $source:ident) => { producer_decision_dependency!(@member_read $reader) };
    (SaturatedInputInitial, $reader:ident, $source:ident) => { producer_decision_dependency!(@member_read $reader).min(5) };
    (SaturatedInputPreserve, $reader:ident, $source:ident) => { producer_decision_dependency!(@member_read $reader).min(5) };
    (GuardedInputInitial, $reader:ident, $source:ident) => { producer_decision_dependency!(@model_b $reader) };
    (GuardedInputPreserve, $reader:ident, $source:ident) => { producer_decision_dependency!(@model_b $reader) };
    (GuardedMemberInitial, $reader:ident, $source:ident) => { producer_decision_dependency!(@member_read $reader) };
    (GuardedFirstInitial, $reader:ident, $source:ident) => { producer_decision_dependency!(@first_b_presence $reader) };
    (GuardedFirstPreserve, $reader:ident, $source:ident) => { producer_decision_dependency!(@first_b_presence $reader) };
    // Whether output `first:b` exists: an optional indexed lookup.
    (@first_b_presence $reader:ident) => {
        u64::from($reader.resolve_optional_entity(FirstKey::at("FirstKey"), "first:b".to_owned())?.is_some())
    };
    (GuardedPresenceInitial, $reader:ident, $source:ident) => { producer_decision_dependency!(@saturated_presence $reader) };
    (GuardedPresencePreserve, $reader:ident, $source:ident) => { producer_decision_dependency!(@saturated_presence $reader) };
    // Whether output `saturated:a` exists: an optional indexed lookup, a source fact.
    (@saturated_presence $reader:ident) => {
        u64::from($reader.resolve_optional_entity(FirstKey::at("FirstKey"), "saturated:a".to_owned())?.is_some())
    };
    (GuardedMemberPreserve, $reader:ident, $source:ident) => { producer_decision_dependency!(@member_read $reader) };
    (@model_b $reader:ident) => {{
        let b = $reader.resolve_entity(ModelKey::at("ModelKey"), "b".to_owned())?;
        $reader.field(&b, ModelValue::at("ModelValue"))?.expect("model b has its value")
    }};
    (@member_read $reader:ident) => {{
        let member = $reader.resolve_entity(MemberKey::at("MemberKey"), "m".to_owned())?;
        $reader.field(&member, MemberValue::at("MemberValue"))?.expect("seeded decision member has its required value")
    }};
    ($operation:ident, $reader:ident, $source:ident) => { 0 };
}
// Shared fixture hooks default to no predecessor observation.
macro_rules! original_contract {
    (PredecessorInitial) => {
        Some(super::super::super::predecessor::original::CONTRACT)
    };
    (PredecessorPreserve) => {
        Some(super::super::super::predecessor::original::CONTRACT)
    };
    ($operation:ident) => {
        None
    };
}
macro_rules! predecessor_contract {
    (PredecessorInitial) => {
        Some(super::super::super::predecessor::CONTRACT)
    };
    (PredecessorPreserve) => {
        Some(super::super::super::predecessor::CONTRACT)
    };
    ($other:ident) => {
        None
    };
}
macro_rules! predecessor_hook {
    (PredecessorInitial, $reader:ident, $source:ident) => {
        super::super::super::predecessor::observe($reader, &$source)
    };
    (PredecessorPreserve, $reader:ident, $source:ident) => {
        super::super::super::predecessor::observe($reader, &$source)
    };
    ($other:ident, $reader:ident, $source:ident) => {
        ()
    };
}
// Producer lifecycle bindings and handler shape shared by the fixture families.
macro_rules! publication_link {
    (none, $writer:ident, $relation:expr, $key:expr, $source:expr, $output:expr) => {
        Ok::<(), WorthQueryApplicationAttemptDenial>(())
    };

    (source, $writer:ident, $relation:expr, $key:expr, $source:expr, $output:expr) => {
        $writer.link($relation, $key, $source, $output)
    };
    (output, $writer:ident, $relation:expr, $key:expr, $source:expr, $output:expr) => {
        $writer.link($relation, $key, $output, $source)
    };
}
macro_rules! phase {
    ($module:ident, $ownership:ident, $direction:ident, $source_query:ident, $operation:ident, $preserve:expr, $posture:ident, $action:ty, $writer:ident, $identity:literal,
     $entity:ident, $facts:ident, $key:ident, $value:ident, $of:ident) => {
        pub mod $module {
            use super::*;
            #[derive(Clone, serde::Serialize)] pub struct Input(pub query::Row);
            worth_query_structured_value_binding!(pub InputBinding for Input { identity: $identity });
            producer_operation!($operation, $entity, $key, $value, $of);
            pub type Operation = $operation;
            pub struct Outputs;
            pub struct Output;
            impl WorthQueryApplicationOutputRole for Output {
                type Schema = Schema; type Contract = Outputs; type Entity = $entity; type Action = $action;
                type Cardinality = WorthQueryExactlyOneOutput; const NAME: &'static str = "derived";
            }
            ordinary_carrier_contract!($operation);
            impl ApplicationMutationOutputContract<Schema> for Outputs {
                const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = producer_role_descriptors!($operation);
                const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] = &[];
            }
            pub struct Decision { source: WorthQueryInvariantMutationTarget<Schema, SourceEntity>, output: Option<WorthQueryInvariantMutationTarget<Schema, $entity>>, member_value: u64, member_target: Option<WorthQueryInvariantMutationTarget<Schema, Member>>, input_member: member_input_type!($operation) }
            pub struct Binding;
            impl ApplicationMutationBinding<Schema> for Binding {
                type Input = Input; type InputBinding = InputBinding; type Result = (); type ResultBinding = ResultBinding;
                type IdempotencyKey = u64; type Operation = Operation; type Decision = Decision;
                type Denial = Denial; type DenialBinding = DenialBinding; type Output = Outputs; type ScopeBinding = Scope;
                type PrincipalBinding = Actor; type Mapping = Mapping; type Principal = Principal;
                type PrincipalIdentity = u64; type PrincipalIdentityBinding = U64ApplicationValueBinding;
                type SourceExpectation = ApplicationQueryMutationSource<query::$source_query::Query>;
                const IDENTITY: &'static str = $identity; const HANDLER_IDENTITY: &'static str = $identity; const IDEMPOTENCY_IDENTITY: &'static str = $identity;
                const REQUIRES_APPLICATION_PROGRAM: bool = producer_program_required!();
                fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements { ApplicationWorkflowExecutionRequirements::not_applicable() }
                fn scope_field() -> SourceKey { source_key() }
                fn principal_binding() -> ApplicationPrincipalBindingRef<Schema, Actor, Mapping, Principal, u64, U64ApplicationValueBinding> { Actor::reference() }
            }
            impl ApplicationMutationIntent<Schema> for Input {
                type Binding = Binding;
                fn input(&self) -> &Self { self }
                fn scope_binding(&self) -> Scope { Scope::new(source_key(), self.0.key.clone()) }
            }
            pub struct Producer;
            pub struct Provider;
            impl WorthQueryApplicationProducerBinding<Schema> for Producer {
                type Operation = Binding; type OutputFamily = Family; type Provider = Provider; type OutputRole = Output;
                const IDENTITY: &'static str = $identity;
                const APPLICABILITY: &'static [WorthQueryProducerApplicability] = &[WorthQueryProducerApplicability::new(PROFILE, WorthQueryProducerLifecyclePosture::$posture)];
                const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
                const OWNERSHIP: ProducerPublicationOwnership = ProducerPublicationOwnership::$ownership;
                const PREDECESSOR_READS: Option<WorthQueryProducerObservationReadContract<Schema>> = predecessor_contract!($operation);
                const ORIGINAL_READS: Option<WorthQueryProducerObservationReadContract<Schema>> = original_contract!($operation);
                const REUSE_POLICY: &'static str = "exact-source";
                const INPUT_REUSE: Option<WorthQueryProducerInputReuseContract> = producer_input_reuse_contract!();
                fn input_capture() -> Option<WorthQueryProducerInputCaptureContract<Schema>> { producer_capture_contract!() }
            }
            impl WorthQueryApplicationProducerProvider<Schema, Producer> for Provider {
                type DenialValue = WorthQueryUndeclaredProducerDenial;
                const SEMANTIC_IDENTITY: &'static str = $identity;
                fn operation_input(&self, row: &query::Row) -> Input { Input(row.clone()) }
                fn idempotency_key(&self, _: &query::Row, identity: &[u8; 32]) -> u64 { u64::from_le_bytes(identity[..8].try_into().unwrap()) }
                fn domain_denial_reason(&self, denial: &Denial) -> Option<&'static str> { Some(match denial {
                    Denial::InitialHasPriorOutput => "InitialHasPriorOutput", Denial::PreserveMissingOutput => "PreserveMissingOutput", Denial::DerivedValueOutOfRange => "DerivedValueOutOfRange",
                }) }
            }
            use worth_query_host::facade::domain;
            #[derive(Clone, Copy, Debug, Eq, PartialEq)] pub struct ReadyDomain;
            #[derive(Clone, Copy, Debug, Eq, PartialEq)] pub struct ReadyOperation;
            #[derive(Clone, Copy, Debug, Eq, PartialEq)] pub struct ReadyFamily;
            worth_query_host::facade::worth_query_conditional_node!(pub ReadyNode in ReadyDomain, ReadyOperation, ReadyFamily => operation $identity);
            pub struct Readiness;
            type Contract = WorthQueryOutputReadinessContract<ReadyDomain, ReadyOperation, ReadyFamily>;
            fn readiness_contract(declaration: &ErasedApplicationSchemaDeclaration) -> Result<Contract, WorthQueryOutputReadinessContractDenial> {
                WorthQueryOutputReadinessContractBuilder::for_field(declaration, $value::at(stringify!($value)),
                    domain::WorthQueryDomainOperationIdentity::new($identity, 1), $identity,
                    readiness_query(stringify!($entity), stringify!($facts), stringify!($value)),
                    domain::WorthQueryOperationProjectionRole::new("derived").unwrap(),
                    domain::WorthQueryExecutionStrategyName::new($identity).unwrap(), $identity)?
                    .field_dependency(declaration, $value::at(stringify!($value)), WorthQueryReadinessFieldUse::DeliveryAndEligibility)?.build_typed()
            }
            fn conditional_binding(contract: &Contract) -> domain::WorthQueryApplicationConditionalOperationBinding<Schema, Operation, Input, ReadyDomain, ReadyOperation, ReadyFamily> {
                domain::WorthQueryApplicationConditionalOperationBinding::declare(Operation::reference(), contract.definition().reference())
            }
            impl WorthQueryApplicationConditionalBinding<Schema> for Readiness {
                type Configuration = (); type Installed = (); type Operation = Operation;
                const IDENTITY: &'static str = $identity;
                const REQUIRED_PRODUCERS: &'static [&'static str] = &[$identity];
                fn package_contract(declaration: &ErasedApplicationSchemaDeclaration) -> Result<WorthQueryApplicationConditionalPackageContract, WorthQueryOutputReadinessContractDenial> {
                    let contract = readiness_contract(declaration)?;
                    let binding = conditional_binding(&contract);
                    Ok(contract.into_package_contract(binding.portable().clone()))
                }
                fn install(_: (), _: &WorthQueryApplicationConditionalProducerAccess<'_, Schema>, installation: &mut WorthQueryConditionalApplicationRuntimeInstallation<Schema>) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
                    let contract = readiness_contract(Schema::declaration().unwrap().erased()).unwrap();
                    let binding = conditional_binding(&contract);
                    let operation = installation.installed_schema().installed_operation(Operation::reference()).unwrap();
                    let node = installation.installed_packages().bind_conditional_application_operation(operation, &binding).unwrap().bind_node(ReadyNode::reference()).unwrap();
                    let ordinal = contract.delivery_ordinal(installation.installed_schema(), node.declaration(), $value::at(stringify!($value))).unwrap();
                    installation.bind_output_readiness::<Producer, _, _, _, _, _, _>(node, ordinal)
                }
            }
            pub struct Handler { pub derive: fn(u64) -> u64 }
            impl OperationHandler<Schema, Binding> for Handler {
                fn decide(&self, input: &Input, reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>) -> HandlerResult<Decision, Denial> {
                    let result = (|| {
                        let source = reader.resolve_entity(source_key(), input.0.key.clone())?;
                        reader.field(&source, source_value())?;
                        predecessor_hook!($operation, reader, source);
                        let member_value = producer_decision_dependency!($operation, reader, source, input);
                        if let Some(denial) = producer_decision_refusal!($operation, member_value) { return Ok(Err(denial)); }
                        let member_target = producer_member_target!($operation, reader, input);
                        let prior = if $preserve { Some(reader.prior_output_preserved_or_created::<preserve::Binding, preserve::Output, initial::Binding, initial::Output>()?) }
                            else { reader.resolve_optional_entity($key::at(stringify!($key)), output_key(&input.0))? };
                        if !$preserve && prior.is_some() { return Ok(Err(Denial::InitialHasPriorOutput)); }
                        if $preserve && prior.is_none() { return Ok(Err(Denial::PreserveMissingOutput)); }
                        let input_member = capture_member_input!($operation, reader, source, prior);
                        let output = match prior {
                            Some(prior) => { reader.field(&prior, $value::at(stringify!($value)))?; Some(reader.mutation_target(&prior)?) }
                            None => None,
                        };
                        Ok(Ok(Decision { source: reader.mutation_target(&source)?, output, member_value, member_target, input_member }))
                    })();
                    match result { Ok(Ok(decision)) => HandlerResult::Completed(decision), Ok(Err(denial)) => HandlerResult::DomainDenied(denial), Err(error) => HandlerResult::ExecutionDenied(error) }
                }
                fn build_candidate(&self, input: &Input, decision: Decision, writer: &mut CandidateWriter<'_, Schema, Binding>) -> HandlerResult<(), Denial> {
                    let value = (self.derive)(input.0.value) + decision.member_value;
                    if value == u64::MAX { return HandlerResult::DomainDenied(Denial::DerivedValueOutOfRange); }
                    let result = (|| {
                        let source = writer.projected_entity(&decision.source)?;
                        producer_member_effect!($operation, writer, decision, input);
                        let output = match &decision.output {
                            Some(prior) => { let output = writer.projected_entity(prior)?; writer.write_field(&output, $value::at(stringify!($value)), value)?; output }
                            None => {
                                let key = output_key(&input.0);
                                let output = writer.create_entity_in_context(&source, $entity::reference(), WorthQueryApplicationEntityKey::new(key.clone()).unwrap())?;
                                writer.initialize_field(&output, $key::at(stringify!($key)), key.clone())?;
                                writer.initialize_field(&output, $value::at(stringify!($value)), value)?;
                                publication_link!($direction, writer, $of::reference(), format!("of:{key}"), &source, &output)?;
                                output
                            }
                        };
                        writer.$writer::<Output>(&output)?;
                        publish_member_input!($operation, writer, decision, output, input);
                        ordinary_carrier_output!($operation,writer,source);
                        Ok::<_, WorthQueryApplicationAttemptDenial>(())
                    })();
                    match result { Ok(()) => HandlerResult::Completed(()), Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)) }
                }
            }
            pub fn declare(schema: ApplicationSchemaDeclarationBuilder<Schema>) -> ApplicationSchemaDeclarationBuilder<Schema> {
                schema.declared_operation::<Operation>(Operation::reference().definition().no_external_effect().no_aftermath().finish()).application_mutation_binding::<Binding>()
            }
            producer_capture_handler!();
        }
    };
}
