//! Four authored cabinet parts with independent installed field scopes.
use super::super::program as base_program;
use super::*;
#[derive(Clone, Debug, Eq, PartialEq, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct StageResult {
    pub old_value: u64,
    pub value: u64,
    pub ordinal: usize,
    pub key: String,
}
worth_query_structured_value_binding!(pub StageResultBinding for StageResult { identity: "owned-outputs.edit-envelope.stage-result.v1" });
macro_rules! part {
    ($module:ident, $operation:ident, $identity:literal, $entity:ident, $key:ident, $key_name:literal, $value:ident, $value_name:literal $(, reads $other_key:ident at $other_key_name:literal, $other_value:ident at $other_value_name:literal, lookup $lookup:literal)?) => { pub mod $module {
        use super::*;
        #[derive(Clone, Debug, WorthValue)]
        #[worth_value(crate = "worth_query_decl::facade::application_schema")]
        pub struct Intent { pub key: String, pub value: u64, pub key_derived_target: bool }
        worth_query_structured_value_binding!(pub Input for Intent { identity: $identity });
        worth_query_operation!(pub $operation for Schema, input Input; access { read_fields: [$key::at($key_name), $value::at($value_name) $(, $other_key::at($other_key_name), $other_value::at($other_value_name))?]; writes: [$value::at($value_name)]; });
        fn scope(input: &Intent) -> String { input.key.clone() }
        worth_query_mutation_binding!(pub Binding for Intent, schema Schema,
            identity $identity, input Input, operation $operation, result StageResultBinding,
            idempotency String, identity $identity, decision Decision, denial DenialBinding,
            handler identity $identity, program required, outputs NoApplicationMutationOutputs,
            principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
            scope $key, field $key::at($key_name), value scope,
        );
        pub struct Decision { target: WorthQueryInvariantMutationTarget<Schema, $entity>, result: StageResult }
        pub struct Handler;
        impl OperationHandler<Schema, Binding> for Handler {
            fn decide(&self, input: &Intent, reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>) -> HandlerResult<Decision, Denial> {
                if input.value > 1000 { return HandlerResult::DomainDenied(Denial::ValueOutOfRange); }
                let result = (|| {
                    let ordinal = reader.stage_ordinal().expect("envelope supplies the real stage ordinal");
                    let key = reader.idempotency_key().clone();
                    let lookup = if input.key_derived_target { key.clone() } else { input.key.clone() };
                    let entity = reader.resolve_entity($key::at($key_name), lookup)?;
                    let value = input.value;
                    $(let dependency = reader.resolve_entity($other_key::at($other_key_name), $lookup.to_owned())?;
                    let value = value + reader.field(&dependency, $other_value::at($other_value_name))?.expect("declared dependency exists");)?
                    let old_value = reader.field(&entity, $value::at($value_name))?.expect("the genuine fixture seeds every declared part field");
                    Ok::<_, HandlerExecutionDenial>(Decision { target: reader.mutation_target(&entity)?, result: StageResult { old_value, value, ordinal, key } })
                })();
                match result { Ok(value) => HandlerResult::Completed(value), Err(cause) => HandlerResult::ExecutionDenied(cause) }
            }
            fn build_candidate(&self, _input: &Intent, decision: Decision, writer: &mut CandidateWriter<'_, Schema, Binding>) -> HandlerResult<StageResult, Denial> {
                match writer.projected_entity(&decision.target).and_then(|entity| writer.write_field(&entity, $value::at($value_name), decision.result.value)) {
                    Ok(()) => HandlerResult::Completed(decision.result), Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
                }
            }
        }
        pub fn declare(schema: ApplicationSchemaDeclarationBuilder<Schema>) -> ApplicationSchemaDeclarationBuilder<Schema> {
            schema.declared_operation::<$operation>($operation::reference().definition().no_external_effect().no_aftermath().finish()).application_mutation_binding::<Binding>()
        }
    }};
}
part!(
    model,
    PatchEnvelopeModel,
    "owned-outputs.edit-envelope.model.v1",
    Model,
    ModelKey,
    "ModelKey",
    ModelValue,
    "ModelValue"
);
part!(
    member,
    PatchEnvelopeMember,
    "owned-outputs.edit-envelope.member.v1",
    Member,
    MemberKey,
    "MemberKey",
    MemberValue,
    "MemberValue"
);
part!(
    cap,
    PatchEnvelopeCap,
    "owned-outputs.edit-envelope.cap.v1",
    Cap,
    CapKey,
    "CapKey",
    CapValue,
    "CapValue"
);
part!(
    calibration,
    PatchEnvelopeCalibration,
    "owned-outputs.edit-envelope.calibration.v1",
    Member,
    MemberKey,
    "MemberKey",
    CalibrationReady,
    "Ready"
);
part!(dependent, PatchEnvelopeDependent, "owned-outputs.edit-envelope.dependent.v1", Member, MemberKey, "MemberKey", MemberValue, "MemberValue", reads ModelKey at "ModelKey", ModelValue at "ModelValue", lookup "a");
part!(cyclic, PatchEnvelopeCyclic, "owned-outputs.edit-envelope.cyclic.v1", Model, ModelKey, "ModelKey", ModelValue, "ModelValue", reads FirstKey at "FirstKey", FirstValue at "FirstValue", lookup "first:a");
part!(after_producer, PatchEnvelopeAfterProducer, "owned-outputs.edit-envelope.after-producer.v1", Cap, CapKey, "CapKey", CapValue, "CapValue", reads FirstKey at "FirstKey", FirstValue at "FirstValue", lookup "first:a");
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    super::interleaved::declare(after_producer::declare(cyclic::declare(
        dependent::declare(calibration::declare(cap::declare(member::declare(
            model::declare(schema),
        )))),
    )))
}
pub fn install(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    super::interleaved::install(setup)?;
    setup.handler::<model::Binding, _>(model::Handler)?;
    setup.handler::<member::Binding, _>(member::Handler)?;
    setup.handler::<cap::Binding, _>(cap::Handler)?;
    setup.handler::<calibration::Binding, _>(calibration::Handler)?;
    setup.handler::<dependent::Binding, _>(dependent::Handler)?;
    setup.handler::<cyclic::Binding, _>(cyclic::Handler)?;
    setup.handler::<after_producer::Binding, _>(after_producer::Handler)
}
macro_rules! root {
    ($module:ident, $connection:ident, $tree:ident, $family:ident, $port:ident) => {
        pub struct $connection;
        impl ApplicationConnectionIdentity for $connection { const IDENTITY: &'static str = concat!("owned-outputs.edit-envelope.root.", stringify!($module)); }
        impl ApplicationOccurrenceConnectionBinding<Schema, base_program::Entry, base_program::Derived> for $connection {}
        pub type $tree = ApplicationOutputGraph<ApplicationConnectionRef<Schema, base_program::Entry, base_program::EntryOutput, base_program::Derived, base_program::$port, $connection>, ApplicationOutputLeaf>;
        impl WorthQueryApplicationRequiredOutputConnection<Schema> for $connection {
            type Source = $module::Binding; type Demand = $family::Demand;
            const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
            fn demand_from_source(_: &$module::Intent) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> { Ok($family::Demand("a".into())) }
        }
        impl WorthQueryApplicationRequiredOutputSource<Schema, $connection> for $module::Binding {
            fn demand_from_source(input: &$module::Intent) -> Result<$family::Demand, WorthQueryRequiredOutputConnectionDenial> { <$connection as WorthQueryApplicationRequiredOutputConnection<Schema>>::demand_from_source(input) }
        }
    };
}
root!(model, ModelRoot, ModelTree, first, FirstInput);
root!(member, MemberRoot, MemberTree, second, SecondInput);
root!(cap, CapRoot, CapTree, first, UpstreamInput);
root!(
    calibration,
    CalibrationRoot,
    CalibrationTree,
    second,
    DownstreamInput
);
root!(dependent, DependentRoot, DependentTree, second, ChildInput);
root!(cyclic, CyclicRoot, CyclicTree, first, RestrictedInput);
root!(
    after_producer,
    AfterProducerRoot,
    AfterProducerTree,
    first,
    FromFirstInput
);
pub struct RestoreRoot;
impl ApplicationConnectionIdentity for RestoreRoot {
    const IDENTITY: &'static str = "owned-outputs.edit-envelope.restore-root";
}
impl ApplicationOccurrenceConnectionBinding<Schema, base_program::Entry, base_program::Derived>
    for RestoreRoot
{
}
pub type RestoreTree = ApplicationOutputGraph<
    ApplicationConnectionRef<
        Schema,
        base_program::Entry,
        base_program::EntryOutput,
        base_program::Derived,
        base_program::RestrictedOwnedInput,
        RestoreRoot,
    >,
    ApplicationOutputLeaf,
>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for RestoreRoot {
    type Source = authored_member::restore::Binding;
    type Demand = first::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        _: &authored_member::restore::Intent,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(first::Demand("a".into()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, RestoreRoot>
    for authored_member::restore::Binding
{
    fn demand_from_source(
        input: &authored_member::restore::Intent,
    ) -> Result<first::Demand, WorthQueryRequiredOutputConnectionDenial> {
        <RestoreRoot as WorthQueryApplicationRequiredOutputConnection<Schema>>::demand_from_source(
            input,
        )
    }
}
pub struct Delivery<R>(std::marker::PhantomData<fn() -> R>);
impl<B, R> ApplicationActionDelivery<Schema, B> for Delivery<R>
where
    B: ApplicationMutationBinding<Schema>,
    R: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
{
    type Mode = ApplicationFixedDeliveryMode<R, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.edit-envelope.delivery";
}
impl<B, R> ApplicationFixedActionDelivery<Schema, B> for Delivery<R>
where
    B: ApplicationMutationBinding<Schema>,
    R: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
{
    type Root = R;
    type Projection = ApplicationNoProjection;
}
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(
        ModelTree,
        (
            MemberTree,
            (
                CapTree,
                (
                    CalibrationTree,
                    (
                        DependentTree,
                        (CyclicTree, (RestoreTree, AfterProducerTree)),
                    ),
                ),
            ),
        ),
    )>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.edit-envelope.program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = base_program::Program::feature_specs();
        specs[0] = base_program::entry_spec_with_envelope_deliveries::<
            Completion,
            Completion,
            Delivery<ModelTree>,
            Delivery<MemberTree>,
            Delivery<CapTree>,
            Delivery<CalibrationTree>,
            Delivery<DependentTree>,
            Delivery<CyclicTree>,
            Delivery<AfterProducerTree>,
            Delivery<RestoreTree>,
        >();
        specs
    }
}
pub fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(
            Cap::reference(),
            WorthQueryApplicationEntityKey::new("envelope-cap").unwrap(),
        )
        .field(CapKey::at("CapKey"), "envelope-cap".to_owned())
        .field(CapValue::at("CapValue"), 7),
    )?;
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        CapNext::reference(),
        "envelope-cap-loop",
        WorthQueryApplicationEntityKey::new("envelope-cap").unwrap(),
        WorthQueryApplicationEntityKey::new("envelope-cap").unwrap(),
    ))?;
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(
            Member::reference(),
            WorthQueryApplicationEntityKey::new("envelope-calibration").unwrap(),
        )
        .field(
            MemberKey::at("MemberKey"),
            "envelope-calibration".to_owned(),
        )
        .field(MemberValue::at("MemberValue"), 8)
        .field(CalibrationReady::at("Ready"), 9),
    )
}
