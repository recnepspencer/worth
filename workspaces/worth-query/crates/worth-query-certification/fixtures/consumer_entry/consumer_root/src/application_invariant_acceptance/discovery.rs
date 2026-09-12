use crate::ConsumerSchema;
use worth_query_decl::facade::application_schema::ApplicationOperationProgramTarget;
use worth_query_host::facade::{
    application_discovery::WorthQueryApplicationCallablePosture,
    primary_graph::WorthQueryPrimaryGraphApplicationRuntime,
};

pub(super) fn verify(application: &WorthQueryPrimaryGraphApplicationRuntime<ConsumerSchema>) {
    // Discovery requires no principal and only describes what can be requested.
    let discovery = application.discovery();
    let mutation = discovery
        .mutations()
        .find(|entry| entry.declaration().operation() == "MutatePlanar")
        .expect("the contributed mutation has a descriptive contract");
    let definition = mutation.declaration();
    assert_eq!(
        definition.input_identity().as_str(),
        "worth.query.certification.planar-mutation-input.v1"
    );
    assert_eq!(
        definition.result_identity().as_str(),
        "worth.query.certification.planar-mutation-result.v1"
    );
    assert_eq!(
        definition.denial_identity().as_str(),
        "worth.query.certification.planar-mutation-denial.v1"
    );
    assert_eq!(
        (
            &*definition.scope().entity,
            &*definition.scope().aspect,
            &*definition.scope().field
        ),
        ("Body", "PlanarPosition", "BodyKey")
    );
    assert_eq!(
        mutation.availability(),
        WorthQueryApplicationCallablePosture::InstalledRequestBinding
    );
    assert!(mutation.effects().any(|effect| matches!(effect,
        ApplicationOperationProgramTarget::Create { entity } if entity == "Body")));
    assert!(mutation.effects().any(|effect| matches!(effect,
        ApplicationOperationProgramTarget::Link { relation, from, to }
            if relation == "PlanarSuccessor" && from == "Body" && to == "Body")));
    assert_eq!(definition.output_roles()[0].name, "anchor");

    let query = discovery
        .queries()
        .find(|entry| entry.definition().name() == "PlanarQuery")
        .expect("the contributed read retains its result and eligibility metadata");
    assert_eq!(
        query.availability(),
        WorthQueryApplicationCallablePosture::InstalledRequestBinding
    );
    assert_eq!(
        query.definition().result_type(),
        "worth.query.certification.planar-read-result.v1"
    );
    assert_eq!(query.definition().scope_entity(), "Body");
    assert!(query.definition().lanes().one_shot_enabled());
    assert!(!query.definition().lanes().live_enabled());
    let read = discovery
        .query_requests()
        .find(|binding| binding.query_name() == "PlanarQuery")
        .expect("the entry input is described separately from compiled query parameters");
    assert_eq!(
        read.input_identity().as_str(),
        "worth.query.certification.planar-read-input.v1"
    );

    for name in ["PositionX", "PositionY", "Length"] {
        let field = discovery
            .fields()
            .find(|field| field.entity == "Body" && field.field == name)
            .expect("the geometry field has declared dimensions");
        assert_eq!(
            field.value_type,
            "worth.query.certification.positive-length.v1"
        );
        assert_eq!(field.unit, Some("Metre"));
        assert_eq!(field.frame, Some("worth.frames.model-local.v1"));
    }
}
