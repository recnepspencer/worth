//! A recorded inverse may demand every field and relation of a wide graph.

use worth_query_declaration::facade::application_aftermath::{
    DeclaredAftermathPostcondition, DeclaredCorrectionAuthority, DeclaredLoweringCorrespondenceRef,
    PortableApplicationAftermathContract, PortableCorrectionMechanism, PortablePreImageDemand,
    PortablePreImageLocus, PortablePreImageRelationLocus, PortableRecordedInverse,
};
use worth_query_declaration::facade::application_schema::ApplicationOperationDecisionReadTarget;

use super::super::PublishedAftermathPosture;
use super::{binding, digest, AftermathInstall};

/// Basis entries of one installed inverse over one kind's fields and links.
fn basis_entries(fields: u32, relations: u32) -> u32 {
    let loci: Vec<_> = (0..fields)
        .map(|index| {
            PortablePreImageLocus::from_untrusted_fields(
                "Member".to_owned(),
                "State".to_owned(),
                format!("field-{index}"),
            )
        })
        .collect();
    let relations: Vec<_> = (0..relations)
        .map(|index| {
            PortablePreImageRelationLocus::from_untrusted_fields(
                format!("Link{index}"),
                "Member".to_owned(),
                "Member".to_owned(),
            )
        })
        .collect();
    let reads = loci
        .iter()
        .map(|locus| ApplicationOperationDecisionReadTarget::Field {
            entity: locus.entity().to_owned(),
            aspect: locus.aspect().to_owned(),
            field: locus.field().to_owned(),
        })
        .chain(
            relations
                .iter()
                .map(|locus| ApplicationOperationDecisionReadTarget::Relation {
                    relation: locus.relation().to_owned(),
                    from: locus.from().to_owned(),
                    to: locus.to().to_owned(),
                }),
        )
        .collect();
    let portable = PortableApplicationAftermathContract::from_untrusted_fields(
        DeclaredCorrectionAuthority::RuntimeAlone,
        Some(PortableCorrectionMechanism::RecordedInverse(
            PortableRecordedInverse::from_untrusted_fields(
                "restore-graph".to_owned(),
                DeclaredLoweringCorrespondenceRef::new("graph-inverse"),
                DeclaredAftermathPostcondition::ExactPriorTruth,
                PortablePreImageDemand::from_untrusted_fields(loci, 1_048_576)
                    .with_untrusted_relations(relations),
            ),
        )),
        None,
    );
    let installed = AftermathInstall::new(binding(digest(21), digest(22), 1), "replace-graph")
        .install_portable(reads, portable)
        .expect("the demand installs under the installation ceiling");
    assert_eq!(
        installed.published_posture(),
        PublishedAftermathPosture::Reversible
    );
    installed.canonical().work().canonical_entries()
}

#[test]
fn an_inverse_demanding_a_wide_graph_installs_with_three_entries_for_each_locus() {
    let narrow = basis_entries(1, 0);
    let wide = basis_entries(40, 17);
    assert_eq!(wide - narrow, 3 * (40 + 17 - 1));
}
