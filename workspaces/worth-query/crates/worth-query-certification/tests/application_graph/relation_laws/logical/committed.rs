//! Ordinary projection and tracked decisions consume the same actual cohort.
use super::super::installation::{install, World};
use super::*;
use std::collections::BTreeSet;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy as Allocation;

pub(super) fn assert_projection(world: &World<Logical>, expected: &operation::Checked) {
    let completed = world
        .application
        .runtime()
        .with_host_advancement(|phase| {
            world
                .projection
                .project(
                    &phase,
                    |reader| {
                        let face = reader.resolve_entity(NodeKey::reference(), 0).unwrap();
                        // compiled-example: begin
                        let root = reader.entity_member(&face).unwrap();
                        let cases = reader
                            .derived_member_relations_from(RootUses::reference(), &root)
                            .unwrap();
                        let negative = cases
                            .iter()
                            .find(|pair| pair.to().case() == Some("Negative"))
                            .unwrap()
                            .to();
                        let component = reader
                            .graph_component_of(
                                UseComponents::reference(),
                                UseSuccessor::reference(),
                                negative,
                            )
                            .unwrap();
                        // compiled-example: end
                        assert_eq!(cases.len(), 2);
                        assert!(negative.as_entity().is_none());
                        assert_eq!(component.identity().bytes(), &expected.identity);
                        let lineage = reader
                            .component_lineage(UseComponents::reference(), component.identity())
                            .unwrap();
                        assert_eq!(lineage.latest_identity().as_ref(), Some(component.identity()));
                        let wrong = ApplicationComponentSetRef::<_, UseComponents, Uses>::from_schema_identifiers(
                            "UseComponents", "Uses",
                        );
                        assert_eq!(
                            reader.component_lineage(wrong, component.identity()).unwrap_err().kind(),
                            WorthQueryInvariantProjectionTraversalDenialKind::ForeignIdentity,
                        );
                        assert_eq!(
                            component
                                .members()
                                .iter()
                                .map(|member| (
                                    member.carrier_id(),
                                    member.case().unwrap().to_owned()
                                ))
                                .collect::<BTreeSet<_>>(),
                            expected.members.iter().cloned().collect()
                        );
                        (component, negative.clone())
                    },
                    Allocation::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    let ((component, original), snapshot, _) = completed.into_parts();
    drop(snapshot);
    // The answer still owns its original Native source and aggregate bodies.
    assert_eq!(component.len(), 2);
    let foreign = install::<Logical>(2, &[(0, 1), (1, 0)]);
    foreign
        .application
        .runtime()
        .with_host_advancement(|phase| {
            let result = foreign
                .projection
                .project(
                    &phase,
                    |reader| {
                        let denied = reader
                            .graph_component_of(
                                UseComponents::reference(),
                                UseSuccessor::reference(),
                                &original,
                            )
                            .err()
                            .expect("a member from another actual projection must refuse");
                        assert_eq!(
                            denied.kind(),
                            WorthQueryInvariantProjectionTraversalDenialKind::ForeignIdentity
                        );
                        let face = reader.resolve_entity(NodeKey::reference(), 0).unwrap();
                        let root = reader.entity_member(&face).unwrap();
                        let cases = reader
                            .derived_member_relations_from(RootUses::reference(), &root)
                            .unwrap();
                        let own = cases
                            .iter()
                            .find(|pair| pair.to().case() == Some("Negative"))
                            .unwrap()
                            .to();
                        assert_eq!(
                            reader
                                .graph_component_of(
                                    UseComponents::reference(),
                                    UseSuccessor::reference(),
                                    own
                                )
                                .unwrap()
                                .len(),
                            2
                        );
                    },
                    Allocation::SystemAllocation,
                )
                .unwrap();
            drop(result);
        })
        .unwrap();
    foreign.close();
}
