use std::num::NonZeroUsize;

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationPerformedMutationOutcome, WorthQueryApplicationProgramOutputProgress,
    WorthQueryApplicationRequestExt, WorthQueryOutputDemandControls,
};
use worth_query_topology_entry::{PlanarOutputRead, PlanarRead, PlanarSourceAdjustment};

use super::super::super::{authentication, installation, seed::length};
use crate::{ConsumerProgram, ConsumerProgramRoot, ConsumerSchema};

pub(super) fn revised_parent_publication_is_the_dependent_basis(
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
    .expect("the program source owner authenticates");
    let request = world.application.request(&principal, &scope);
    let controls = WorthQueryOutputDemandControls::new(
        NonZeroUsize::new(4_096).unwrap(),
        NonZeroUsize::new(8_192).unwrap(),
    );

    for (replacement_y, idempotency) in [(2, 10_052), (3, 10_053)] {
        let source = request
            .query(PlanarRead {
                body_key: "anchor-a".to_owned(),
            })
            .execute()
            .expect("the revised source remains readable")
            .observed_sources()[0]
            .clone();
        let performed = request
            .mutate(PlanarSourceAdjustment {
                scope_key: "anchor-a".to_owned(),
                replacement_y: length(replacement_y),
            })
            .expect_source(source)
            .idempotency(&idempotency)
            .execute_performed::<ConsumerProgram, ConsumerProgramRoot>(&world.application)
            .expect("the source revision publishes");
        let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = performed else {
            panic!("the source revision is fresh")
        };
        let mut performed = performed
            .start_required_outputs(&request, controls)
            .expect("the typed output graph starts");
        let settled = loop {
            match performed
                .required_output_mut()
                .advance(&request)
                .expect("the typed output graph advances")
            {
                WorthQueryApplicationProgramOutputProgress::Pending => {}
                WorthQueryApplicationProgramOutputProgress::Settled(settled) => break settled,
            }
        };
        let retained = request.at(settled.observation());
        assert_eq!(
            retained
                .query(PlanarOutputRead {
                    body_key: "final:anchor-a".to_owned(),
                })
                .execute()
                .expect("the dependent output is readable")
                .rows()[0]
                .value,
            length(replacement_y + 2),
            "the dependent must consume the newly published parent output"
        );
        assert_eq!(
            retained
                .query(PlanarOutputRead {
                    body_key: "final:final:anchor-a".to_owned(),
                })
                .execute()
                .expect("the nested dependent output is readable")
                .rows()[0]
                .value,
            length(replacement_y + 4),
            "the nested dependent must consume the newly published child output"
        );
    }
}
