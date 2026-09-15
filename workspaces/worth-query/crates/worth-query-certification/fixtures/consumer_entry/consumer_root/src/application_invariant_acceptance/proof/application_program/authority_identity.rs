use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationRequestExt, WorthQueryPerformedMutationExecutionDenial,
    },
    declaration::application_program::ApplicationProgramInventoryIdentity,
};
use worth_query_topology_entry::{PlanarRead, PlanarSourceAdjustment};

use super::super::super::{authentication, installation, seed::length};
use crate::{ConsumerProgramInventory, ConsumerProgramRoot, ConsumerSchema};

struct ForgedInventory;

impl ApplicationProgramInventoryIdentity for ForgedInventory {
    const IDENTITY: &'static str =
        <ConsumerProgramInventory as ApplicationProgramInventoryIdentity>::IDENTITY;
}

pub(super) fn forged_inventory_is_denied_before_publication(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let world = installation::install(foreign);
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .expect("the program application authenticates its principal");
    let request = world.application.request(&principal, &scope);

    let inventory_denial = match prepared(&request, &10_014)
        .execute_performed::<crate::ConsumerProgram, ForgedInventory, ConsumerProgramRoot>(
            &world.application,
        ) {
        Err(denial) => denial,
        Ok(_) => panic!("a same-name foreign inventory marker must be rejected"),
    };
    assert!(matches!(
        inventory_denial,
        WorthQueryPerformedMutationExecutionDenial::Preparation(
            worth_query_host::facade::application_entry::WorthQueryRequiredOutputPreparationDenial::MissingInventory
        )
    ));

    assert_eq!(
        current_y(&request),
        length(1),
        "the denial precedes publication"
    );
}

pub(super) fn unavailable_inventory_is_denied_before_publication() {
    let world = installation::install_unavailable();
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .expect("the unavailable program authenticates its principal");
    let request = world.application.request(&principal, &scope);

    let denial = match prepared(&request, &10_015).execute_performed::<
        crate::UnavailableConsumerProgram,
        ConsumerProgramInventory,
        ConsumerProgramRoot,
    >(&world.application)
    {
        Err(denial) => denial,
        Ok(_) => panic!("an unavailable inventory must be denied before publication"),
    };
    assert!(
        matches!(
            denial,
            WorthQueryPerformedMutationExecutionDenial::Preparation(
                worth_query_host::facade::application_entry::WorthQueryRequiredOutputPreparationDenial::Unavailable { ref feature }
            ) if feature == "worth.query.certification.planar-final-output-feature.v1"
        ),
        "unexpected unavailable denial: {denial:?}"
    );
    assert_eq!(current_y(&request), length(1));
}

fn prepared<'a>(
    request: &'a super::super::Request<'a>,
    command: &'a u64,
) -> worth_query_host::facade::application_entry::WorthQueryApplicationMutationRequestWithIdempotency<
    'a,
    'a,
    'a,
    'a,
    ConsumerSchema,
    PlanarSourceAdjustment,
    worth_query_host::facade::application_entry::WorthQueryMutationSourcePrepared,
>{
    request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(
            request
                .query(PlanarRead {
                    body_key: "anchor-a".to_owned(),
                })
                .execute()
                .unwrap()
                .observed_sources()[0]
                .clone(),
        )
        .idempotency(command)
}

fn current_y(request: &super::super::Request<'_>) -> worth_query_consumer_values::PositiveLength {
    request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap()
        .rows()[0]
        .y
}
