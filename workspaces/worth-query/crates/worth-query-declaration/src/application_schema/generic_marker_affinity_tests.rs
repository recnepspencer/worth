use super::{
    ApplicationAspectMarkerIdentity, ApplicationEffectMarkerIdentity,
    ApplicationEntityMarkerIdentity, ApplicationExternalEffectBinding,
    ApplicationExternalEffectProtocol, ApplicationOperationMarkerIdentity,
    ApplicationRetainedEffectBinding, ApplicationSchema, ApplicationStructuredValueBinding,
    ApplicationValueValidationDenial,
};
use crate::application_query::ApplicationQueryMarkerIdentity;
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};

trait TopologySchemaBinding {}
trait WorkSchemaBinding {}
trait ModularSchemaBinding: ApplicationSchema {}

crate::worth_query_entity!(Body for Schema: TopologySchemaBinding);
crate::worth_query_aspect!(
    BodyTopology for Schema: TopologySchemaBinding, Body;
    identity = AspectIdentity(81),
    revision = AspectContractRevision(3),
);

crate::worth_query_entity!(Component for Schema: ModularSchemaBinding);
crate::worth_query_application_contribution! {
    contribution ComponentContribution for Schema: ModularSchemaBinding {
        identity: "worth.query.test.component-contribution.v1",
        members: |schema| { schema.entity(Component::reference::<Schema>()) }
    }
}
crate::worth_query_application! {
    ComposedSchema {
        owner: "worth.query.test.composed",
        version: (1, 0),
        contributions: [ComponentContribution],
    }
}
impl ModularSchemaBinding for ComposedSchema {}

struct HouseSchema;
impl TopologySchemaBinding for HouseSchema {}
impl WorkSchemaBinding for HouseSchema {}

struct BankSchema;
impl TopologySchemaBinding for BankSchema {}
impl WorkSchemaBinding for BankSchema {}

struct Input;
struct QueryOutput;
struct Payload(Vec<u8>);
struct Scope;

macro_rules! structured_binding {
    ($binding:ident, $value:ty, $identity:literal) => {
        struct $binding;

        impl ApplicationStructuredValueBinding for $binding {
            type Value = $value;
            const IDENTITY_NAME: &'static str = $identity;

            fn validate(
                _: &Self::Value,
            ) -> std::result::Result<(), ApplicationValueValidationDenial> {
                Ok(())
            }
        }
    };
}

structured_binding!(InputBinding, Input, "worth.query.test.work-input.v1");
structured_binding!(
    ResultBinding,
    QueryOutput,
    "worth.query.test.work-result.v1"
);
structured_binding!(PayloadBinding, Payload, "worth.query.test.work-payload.v1");

impl ApplicationRetainedEffectBinding for PayloadBinding {
    fn retained_bytes(value: &Self::Value) -> u64 {
        value.0.capacity() as u64
    }
}

impl ApplicationExternalEffectBinding for PayloadBinding {
    const PROTOCOL: ApplicationExternalEffectProtocol = ApplicationExternalEffectProtocol::new(
        BoundaryProtocolIdentity::new("worth.query.test.work-effect"),
        BoundaryProtocolVersion::new(1),
    );
    const MAX_EXTERNAL_BYTES: u64 = 64;

    fn external_effect_bytes(value: &Self::Value) -> Vec<u8> {
        value.0.clone()
    }
}

crate::worth_query_application_query!(
    WorkQuery for Schema: WorkSchemaBinding,
    identity "worth.query.test.work-query.v1",
    parameters InputBinding,
    result ResultBinding,
    scope Scope => "Scope",
    name "work"
);
crate::worth_query_operation!(
    WorkOperation for Schema: WorkSchemaBinding,
    input InputBinding
);
crate::worth_query_effect!(
    WorkEffect for Schema: WorkSchemaBinding,
    payload PayloadBinding
);

fn assert_topology_marker_affinity<Schema>()
where
    Schema: TopologySchemaBinding,
    Body: ApplicationEntityMarkerIdentity<Schema>,
    BodyTopology: ApplicationAspectMarkerIdentity<Schema, Body>,
{
}

#[test]
fn entry_owned_binding_admits_the_same_markers_into_independent_root_schemas() {
    assert_topology_marker_affinity::<HouseSchema>();
    assert_topology_marker_affinity::<BankSchema>();

    let house_body = Body::reference::<HouseSchema>();
    let bank_body = Body::reference::<BankSchema>();
    let house_topology = BodyTopology::reference::<HouseSchema>();
    let bank_topology = BodyTopology::reference::<BankSchema>();

    assert_eq!(house_body.name(), "Body");
    assert_eq!(bank_body.name(), "Body");
    assert_eq!(house_topology.name(), "BodyTopology");
    assert_eq!(bank_topology.name(), "BodyTopology");
}

#[test]
fn root_owned_application_composes_an_entry_owned_generic_contribution() {
    let declaration = ComposedSchema::declaration().unwrap();

    assert_eq!(declaration.erased().members().len(), 1);
    assert_eq!(declaration.contributions().len(), 1);
    assert_eq!(
        ComponentContribution::reference::<ComposedSchema>()
            .identity()
            .as_str(),
        "worth.query.test.component-contribution.v1"
    );
}

#[test]
fn structured_work_markers_share_explicit_schema_affinity_without_value_owned_query_traits() {
    fn assert_work_affinity<Schema>()
    where
        Schema: WorkSchemaBinding,
        WorkQuery: ApplicationQueryMarkerIdentity<Schema>,
        WorkOperation: ApplicationOperationMarkerIdentity<Schema>,
        WorkEffect: ApplicationEffectMarkerIdentity<Schema>,
    {
    }

    assert_work_affinity::<HouseSchema>();
    assert_work_affinity::<BankSchema>();

    let house_query = WorkQuery::reference::<HouseSchema>();
    let bank_operation = WorkOperation::reference::<BankSchema>();
    let house_effect = WorkEffect::reference::<HouseSchema>();

    assert_eq!(
        house_query.parameter_type().as_str(),
        InputBinding::IDENTITY_NAME
    );
    assert_eq!(
        house_query.result_type().as_str(),
        ResultBinding::IDENTITY_NAME
    );
    assert_eq!(
        bank_operation.input_identity().as_str(),
        InputBinding::IDENTITY_NAME
    );
    assert_eq!(
        house_effect.payload_identity().as_str(),
        PayloadBinding::IDENTITY_NAME
    );
    assert_eq!(PayloadBinding::retained_bytes(&Payload(vec![1, 2, 3])), 3);
    assert_eq!(
        PayloadBinding::external_effect_bytes(&Payload(vec![4, 5])),
        vec![4, 5]
    );
}
