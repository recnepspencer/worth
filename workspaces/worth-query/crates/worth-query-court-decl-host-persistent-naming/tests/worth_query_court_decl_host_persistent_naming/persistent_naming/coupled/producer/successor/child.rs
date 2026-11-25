//! `stir-child`, discovered from the root's output: it creates (then keeps)
//! output `stir-child:<model>` and writes member `reference` to the edit's
//! value plus one, the value the root's Preserve read.
use super::super::readiness;
use super::*;

pub const PROFILE: &str = "stir-child";

pub fn output_key(model: &str) -> String {
    format!("stir-child:{model}")
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
    const IDENTITY: &'static str = "persistent-naming.stir-child.feature";
}
impl ApplicationInputPort<Schema, Feature> for InputPort {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "stir-child-source";
    const REQUIRED: bool = false;
}
impl ApplicationOutputPort<Schema, Feature> for OutputPort {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "stir-child-result";
}
pub fn feature() -> ApplicationFeatureSpec {
    ApplicationFeatureSpec::root::<Schema, Feature>()
        .provides::<OutputPort>()
        .conditional_operation::<initial::Operation>()
        .conditional_operation::<preserve::Operation>()
        .finish()
}

/// Both phases' decision: the source model, the prior output (Preserve),
/// and member `reference`.
pub struct Targets {
    source: WorthQueryInvariantMutationTarget<Schema, Model>,
    reference: WorthQueryInvariantMutationTarget<Schema, Member>,
}

macro_rules! child_phase {
    ($module:ident, $operation:ident, $identity:literal, $posture:ident, $action:ident) => {
        pub mod $module {
            use super::*;
            worth_query_operation!(pub $operation for Schema, input InputBinding; access {
                read_entities: [Model, Member];
                read_fields: [ModelKey::at("ModelKey"), ModelValue::at("ModelValue"), MemberKey::at("MemberKey"), MemberValue::at("MemberValue")];
                read_relations: [ModelMember];
                writes: [MemberKey::at("MemberKey"), MemberValue::at("MemberValue")];
                creates: [Member]; links: [ModelMember];
            });
            pub type Operation = $operation;
            pub struct Outputs;
            pub struct Primary;
            impl WorthQueryApplicationOutputRole for Primary {
                type Schema = Schema;
                type Contract = Outputs;
                type Entity = Member;
                type Action = $action;
                type Cardinality = WorthQueryExactlyOneOutput;
                const NAME: &'static str = "primary";
            }
            impl ApplicationMutationOutputContract<Schema> for Outputs {
                const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
                    &[<Primary as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
                const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] = &[];
            }
            pub type Decision = (Targets, Option<WorthQueryInvariantMutationTarget<Schema, Member>>);
            producer_binding!($identity, $posture, Primary);

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
                        let reference = reader
                            .resolve_entity(MemberKey::at("MemberKey"), "reference".to_owned())?;
                        reader.field(&reference, MemberValue::at("MemberValue"))?;
                        let prior = prior!(reader, $posture);
                        Ok::<_, HandlerExecutionDenial>((
                            Targets {
                                source: reader.mutation_target(&source)?,
                                reference: reader.mutation_target(&reference)?,
                            },
                            prior,
                        ))
                    })();
                    match result {
                        Ok(decision) => HandlerResult::Completed(decision),
                        Err(cause) => HandlerResult::ExecutionDenied(cause),
                    }
                }

                fn build_candidate(
                    &self,
                    input: &Input,
                    (targets, prior): Decision,
                    writer: &mut CandidateWriter<'_, Schema, Binding>,
                ) -> HandlerResult<edit::Edited, edit::Denial> {
                    let result = (|| {
                        let reference = writer.projected_entity(&targets.reference)?;
                        writer.write_field(&reference, MemberValue::at("MemberValue"), input.value + 1)?;
                        output!($posture, writer, targets, prior, input);
                        Ok::<_, WorthQueryApplicationAttemptDenial>(())
                    })();
                    match result {
                        Ok(()) => HandlerResult::Completed(edit::Edited),
                        Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
                    }
                }
            }
        }
    };
}
macro_rules! prior {
    ($reader:ident, Initial) => {
        None
    };
    ($reader:ident, Preserve) => {{
        let prior = $reader.prior_output_preserved_or_created::<
                        Binding,
                        Primary,
                        super::initial::Binding,
                        super::initial::Primary,
                    >()?;
        $reader.field(&prior, MemberValue::at("MemberValue"))?;
        Some($reader.mutation_target(&prior)?)
    }};
}
/// The Initial creates and names the output; the Preserve keeps it.
macro_rules! output {
    (Initial, $writer:ident, $targets:ident, $prior:ident, $input:ident) => {{
        let _: Option<WorthQueryInvariantMutationTarget<Schema, Member>> = $prior;
        let source = $writer.projected_entity(&$targets.source)?;
        let key = output_key(&$input.key);
        let output = $writer.create_entity_in_context(
            &source,
            Member::reference(),
            WorthQueryApplicationEntityKey::new(key.clone()).unwrap(),
        )?;
        $writer.initialize_field(&output, MemberKey::at("MemberKey"), key.clone())?;
        $writer.initialize_field(&output, MemberValue::at("MemberValue"), $input.value)?;
        $writer.link(
            ModelMember::reference(),
            format!("owns:{key}"),
            &source,
            &output,
        )?;
        $writer.name_created_entity(&output, outputs::birth(&key, "authored"))?;
        $writer.create_output::<Primary>(&output)?;
    }};
    (Preserve, $writer:ident, $targets:ident, $prior:ident, $input:ident) => {{
        let prior = $prior.expect("a Preserve keeps its prior output");
        let output = $writer.projected_entity(&prior)?;
        $writer.write_field(&output, MemberValue::at("MemberValue"), $input.value)?;
        $writer.preserve_output::<Primary>(&output)?;
    }};
}
child_phase!(
    initial,
    StirChildInitial,
    "persistent-naming.stir-child.initial",
    Initial,
    WorthQueryCreateOutput
);
child_phase!(
    preserve,
    StirChildPreserve,
    "persistent-naming.stir-child.preserve",
    Preserve,
    WorthQueryPreserveOutput
);

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
