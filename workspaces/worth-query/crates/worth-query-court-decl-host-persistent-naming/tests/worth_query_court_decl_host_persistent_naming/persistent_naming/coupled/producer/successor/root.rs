//! `stir-root`, an owned route (`SourceScopeCascade`): only an owned route
//! derives a successor round of its chain. Its Initial creates and names
//! output `stir-root:<model>`; its
//! Preserve reads member `reference` (which the child writes) and, when
//! `courts` registered a drop for the edit's value, reads the dropped
//! member's and seam's names in its decision and drops both.
use super::super::readiness;
use super::*;

pub const PROFILE: &str = "stir-root";

pub fn output_key(model: &str) -> String {
    format!("stir-root:{model}")
}

pub struct Family;
impl WorthQueryProducerOutputFamily<Schema> for Family {
    type Source = query::model::Binding;
    type Entity = Member;
    const IDENTITY: &'static str = PROFILE;
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = &[
        WorthQueryProducerApplicability::new(PROFILE, WorthQueryProducerLifecyclePosture::Initial),
        WorthQueryProducerApplicability::new(PROFILE, WorthQueryProducerLifecyclePosture::Preserve),
    ];
    fn profile_kind(_: &query::Row) -> &'static str {
        PROFILE
    }
}
#[derive(Clone)]
pub struct Demand(pub String);
impl WorthQueryApplicationOutputDemand<Schema> for Demand {
    type OutputFamily = Family;
    fn source_intent(&self) -> query::model::Read {
        query::model::Read(self.0.clone())
    }
}

pub struct Feature;
pub struct InputPort;
pub struct OutputPort;
impl ApplicationFeature<Schema> for Feature {
    type Inputs = ApplicationFeatureInputList<InputPort, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "persistent-naming.stir-root.feature";
}
impl ApplicationInputPort<Schema, Feature> for InputPort {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "stir-root-source";
    const REQUIRED: bool = false;
}
impl ApplicationOutputPort<Schema, Feature> for OutputPort {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "stir-root-result";
}
pub fn feature() -> ApplicationFeatureSpec {
    ApplicationFeatureSpec::root::<Schema, Feature>()
        .provides::<OutputPort>()
        .conditional_operation::<initial::Operation>()
        .conditional_operation::<preserve::Operation>()
        .finish()
}

pub mod initial {
    use super::*;
    worth_query_operation!(pub StirRootInitial for Schema, input InputBinding; access {
        read_entities: [Model, Member];
        read_fields: [ModelKey::at("ModelKey"), ModelValue::at("ModelValue"), MemberKey::at("MemberKey"), MemberValue::at("MemberValue")];
        read_relations: [ModelMember];
        writes: [MemberKey::at("MemberKey"), MemberValue::at("MemberValue")];
        creates: [Member]; links: [ModelMember];
    });
    pub type Operation = StirRootInitial;
    pub struct Outputs;
    pub struct Primary;
    impl WorthQueryApplicationOutputRole for Primary {
        type Schema = Schema;
        type Contract = Outputs;
        type Entity = Member;
        type Action = WorthQueryCreateOutput;
        type Cardinality = WorthQueryExactlyOneOutput;
        const NAME: &'static str = "primary";
    }
    impl ApplicationMutationOutputContract<Schema> for Outputs {
        const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
            &[<Primary as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
        const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] = &[];
    }
    pub struct Decision(WorthQueryInvariantMutationTarget<Schema, Model>);
    producer_binding!(
        "persistent-naming.stir-root.initial",
        Initial,
        Primary,
        SourceScopeCascade
    );

    pub struct Handler;
    impl OperationHandler<Schema, Binding> for Handler {
        fn decide(
            &self,
            input: &Input,
            reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
        ) -> HandlerResult<Decision, edit::Denial> {
            let result = (|| {
                let source = reader.resolve_entity(model_key(), input.key.clone())?;
                reader.field(&source, ModelValue::at("ModelValue"))?;
                Ok::<_, HandlerExecutionDenial>(Decision(reader.mutation_target(&source)?))
            })();
            match result {
                Ok(decision) => HandlerResult::Completed(decision),
                Err(cause) => HandlerResult::ExecutionDenied(cause),
            }
        }

        fn build_candidate(
            &self,
            input: &Input,
            decision: Decision,
            writer: &mut CandidateWriter<'_, Schema, Binding>,
        ) -> HandlerResult<edit::Edited, edit::Denial> {
            let result = (|| {
                let source = writer.projected_entity(&decision.0)?;
                let key = output_key(&input.key);
                let member = writer.create_entity_in_context(
                    &source,
                    Member::reference(),
                    WorthQueryApplicationEntityKey::new(key.clone()).unwrap(),
                )?;
                writer.initialize_field(&member, MemberKey::at("MemberKey"), key.clone())?;
                writer.initialize_field(&member, MemberValue::at("MemberValue"), input.value)?;
                writer.link(
                    ModelMember::reference(),
                    format!("owns:{key}"),
                    &source,
                    &member,
                )?;
                let birth = outputs::birth(&key, "authored");
                writer.name_created_entity(&member, birth)?;
                writer.create_output::<Primary>(&member)?;
                Ok::<_, WorthQueryApplicationAttemptDenial>(())
            })();
            match result {
                Ok(()) => HandlerResult::Completed(edit::Edited),
                Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
            }
        }
    }
}

pub mod preserve {
    use super::*;
    worth_query_operation!(pub StirRootPreserve for Schema, input InputBinding; access {
        read_entities: [Model, Member];
        read_fields: [ModelKey::at("ModelKey"), ModelValue::at("ModelValue"), MemberKey::at("MemberKey"), MemberValue::at("MemberValue")];
        read_relations: [ModelMember, Seam];
        writes: [MemberKey::at("MemberKey"), MemberValue::at("MemberValue")];
        creates: [Member]; links: [ModelMember];
        deletes: [Member]; unlinks: [Seam, ModelMember];
    });
    pub type Operation = StirRootPreserve;
    pub struct Outputs;
    pub struct Primary;
    impl WorthQueryApplicationOutputRole for Primary {
        type Schema = Schema;
        type Contract = Outputs;
        type Entity = Member;
        type Action = WorthQueryPreserveOutput;
        type Cardinality = WorthQueryExactlyOneOutput;
        const NAME: &'static str = "primary";
    }
    impl ApplicationMutationOutputContract<Schema> for Outputs {
        const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
            &[<Primary as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
        const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] = &[];
    }
    pub struct Decision {
        primary: WorthQueryInvariantMutationTarget<Schema, Member>,
        drop: Option<super::super::courts::Drop>,
        /// The model to bear member `made` in, and the round's reference.
        made: Option<(WorthQueryInvariantMutationTarget<Schema, Model>, u64)>,
    }
    producer_binding!(
        "persistent-naming.stir-root.preserve",
        Preserve,
        Primary,
        SourceScopeCascade
    );

    pub struct Handler;
    impl OperationHandler<Schema, Binding> for Handler {
        fn decide(
            &self,
            input: &Input,
            reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
        ) -> HandlerResult<Decision, edit::Denial> {
            let result = (|| {
                let source = reader.resolve_entity(model_key(), input.key.clone())?;
                reader.field(&source, ModelValue::at("ModelValue"))?;
                let primary = reader.prior_output_preserved_or_created::<
                    Binding,
                    Primary,
                    super::initial::Binding,
                    super::initial::Primary,
                >()?;
                reader.field(&primary, MemberValue::at("MemberValue"))?;
                // The child writes this value: reading it reopens this
                // producer as a successor round once the child has run.
                let reference =
                    reader.resolve_entity(MemberKey::at("MemberKey"), "reference".to_owned())?;
                let seen = reader.field(&reference, MemberValue::at("MemberValue"))?;
                let seen = seen.unwrap_or_default();
                let drop = super::super::courts::decide(reader, input.value, seen)?;
                let made = match super::super::pending::root_decide(reader, input.value, seen) {
                    true => Some((reader.mutation_target(&source)?, seen)),
                    false => None,
                };
                Ok::<_, HandlerExecutionDenial>(Decision {
                    primary: reader.mutation_target(&primary)?,
                    drop,
                    made,
                })
            })();
            match result {
                Ok(decision) => HandlerResult::Completed(decision),
                Err(cause) => HandlerResult::ExecutionDenied(cause),
            }
        }

        fn build_candidate(
            &self,
            input: &Input,
            decision: Decision,
            writer: &mut CandidateWriter<'_, Schema, Binding>,
        ) -> HandlerResult<edit::Edited, edit::Denial> {
            let result = (|| {
                let primary = writer.projected_entity(&decision.primary)?;
                writer.write_field(&primary, MemberValue::at("MemberValue"), input.value)?;
                writer.preserve_output::<Primary>(&primary)?;
                if let Some(drop) = decision.drop {
                    super::super::courts::build(drop, writer)?;
                }
                if let Some((model, reference)) = decision.made {
                    super::super::pending::bear(writer, &model, reference)?;
                }
                Ok::<_, WorthQueryApplicationAttemptDenial>(())
            })();
            match result {
                Ok(()) => HandlerResult::Completed(edit::Edited),
                Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
            }
        }
    }
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    preserve::declare(initial::declare(schema))
}

pub fn contracts(
    contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    contracts.producer::<initial::Producer>()?;
    contracts.producer::<preserve::Producer>()?;
    contracts.conditional::<initial::Readiness>()?;
    contracts.conditional::<preserve::Readiness>()?;
    Ok(())
}

pub fn install(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    setup.handler::<initial::Binding, _>(initial::Handler)?;
    setup.handler::<preserve::Binding, _>(preserve::Handler)?;
    if !super::installing() {
        return Ok(());
    }
    setup.producer::<initial::Producer>(initial::Provider)?;
    setup.producer::<preserve::Producer>(preserve::Provider)?;
    setup.conditional::<initial::Readiness>(())?;
    setup.conditional::<preserve::Readiness>(())
}
