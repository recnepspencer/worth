//! Closed corruption vocabulary for the sealed history court receiver.
#[derive(Clone, Copy, Debug)]
pub enum OutputOriginHeaderCorruptionForTest {
    RevivedAnchor,
    EmptyPublicationRoster,
    MissingAnchorMapping,
    AnchorWithoutCreatePatch,
    RepeatedCoAnchors,
    MissingCoAnchorMapping,
    DuplicateEntityGenerations,
    RevivedEntityGenerations,
    RepeatedRelationGenerations,
    RelationWithoutCreatePatch,
    IncompleteOriginMembership,
    UnownedPublication,
    MissingMemberMapping,
    MissingRelationInverse,
    RepeatedRoute,
    RepeatedOutputMember,
    ForeignSource,
    CyclicSource,
    UnownedRelation,
    UnownedEntity,
    CascadeEmptyPublicationRoster,
    CascadeRepeatedAuthoredAnchor,
    CascadeUnownedPublication,
    CascadeIncompleteOriginMembership,
    CascadeRepeatedRoute,
    CascadeForeignSource,
    CascadeCyclicSource,
}

use crate::history::data::canonical_commit_envelope::required_output_origin::header_corruption::origin_header_for_test as origin;
use crate::history::data::CanonicalCommitEnvelope;
use crate::transactions::data::primitives::RecordRef;

pub(super) fn corrupt(
    envelope: &mut CanonicalCommitEnvelope,
    corruption: OutputOriginHeaderCorruptionForTest,
) {
    use OutputOriginHeaderCorruptionForTest as C;
    match corruption {
        C::CascadeEmptyPublicationRoster => envelope.edit_retirement_header_for_test(|event| {
            event.publications.clear();
        }),
        C::CascadeRepeatedAuthoredAnchor => envelope.edit_retirement_header_for_test(|event| {
            event.co_anchors.push(event.co_anchors[0]);
        }),
        C::CascadeUnownedPublication => envelope.edit_retirement_header_for_test(|event| {
            origin(&mut event.publications, 0).source_scope_owned = false
        }),
        C::CascadeIncompleteOriginMembership => envelope.edit_retirement_header_for_test(|event| {
            origin(&mut event.publications, 0).producer.clear()
        }),
        C::CascadeRepeatedRoute => envelope.edit_retirement_header_for_test(|event| {
            let source = event.publications[0].meaning.source_scope;
            origin(&mut event.publications, 1).meaning.source_scope = source;
        }),
        C::CascadeForeignSource => envelope.edit_retirement_header_for_test(|event| {
            // Deliberately move the source outside the recorded generation roster.
            let source = event.publications[0].members[0].id;
            origin(&mut event.publications, 0).meaning.source_scope =
                crate::identity::data::EntityId::new(
                    source.partition_id,
                    source.local_slot_value(),
                    source.generation_value() + 1024,
                );
        }),
        C::CascadeCyclicSource => envelope.edit_retirement_header_for_test(|event| {
            let source = event.publications[0].outputs[0];
            origin(&mut event.publications, 0).meaning.source_scope = source;
        }),
        C::RevivedAnchor => envelope.edit_restoration_header_for_test(|event, _| {
            event.original_anchor = event.anchor;
        }),
        C::EmptyPublicationRoster => envelope.edit_restoration_header_for_test(|event, _| {
            event.publications.clear();
        }),
        C::MissingAnchorMapping => envelope.edit_restoration_header_for_test(|event, _| {
            event.entities.retain(|(_, id)| *id != event.anchor);
        }),
        C::AnchorWithoutCreatePatch => envelope.edit_restoration_header_for_test(|event, patch| {
            patch
                .authoritative_record_patches
                .retain(|p| p.target != RecordRef::Entity(event.anchor));
        }),
        C::RepeatedCoAnchors => envelope.edit_restoration_header_for_test(|event, _| {
            event.co_anchors.push(event.co_anchors[0]);
        }),
        C::MissingCoAnchorMapping => envelope.edit_restoration_header_for_test(|event, _| {
            let pair = event.co_anchors[0];
            event.entities.retain(|p| *p != pair);
        }),
        C::DuplicateEntityGenerations => envelope.edit_restoration_header_for_test(|event, _| {
            let n = event.entities.len();
            event.entities[n - 1].0 = event.entities[n - 2].0;
        }),
        C::RevivedEntityGenerations => envelope.edit_restoration_header_for_test(|event, _| {
            let n = event.entities.len();
            event.entities[n - 1].0 = event.entities[n - 1].1;
        }),
        C::RepeatedRelationGenerations => envelope.edit_restoration_header_for_test(|event, _| {
            event.relations[1].0 = event.relations[0].0;
        }),
        C::RelationWithoutCreatePatch => {
            envelope.edit_restoration_header_for_test(|event, patch| {
                let id = event.relations[0].1;
                patch
                    .authoritative_record_patches
                    .retain(|p| p.target != RecordRef::Relation(id));
            })
        }
        C::IncompleteOriginMembership => envelope.edit_restoration_header_for_test(|event, _| {
            origin(&mut event.publications, 0).producer.clear();
        }),
        C::UnownedPublication => envelope.edit_restoration_header_for_test(|event, _| {
            origin(&mut event.publications, 0).source_scope_owned = false;
        }),
        C::MissingMemberMapping => envelope.edit_restoration_header_for_test(|event, _| {
            let id = event.publications[0].members[0].id;
            event.entities.retain(|(_, new)| *new != id);
        }),
        C::MissingRelationInverse => envelope.edit_restoration_header_for_test(|event, _| {
            event.relations.remove(0);
        }),
        C::RepeatedRoute => envelope.edit_restoration_header_for_test(|event, _| {
            let source = event.publications[0].meaning.source_scope;
            origin(&mut event.publications, 1).meaning.source_scope = source;
        }),
        C::RepeatedOutputMember => envelope.edit_restoration_header_for_test(|event, _| {
            let id = event.publications[0].outputs[0];
            let p = origin(&mut event.publications, 1);
            p.outputs[0] = id;
            p.members[0].id = id;
            p.outputs.sort();
            p.members.sort_by_key(|member| member.id);
        }),
        C::ForeignSource => envelope.edit_restoration_header_for_test(|event, _| {
            origin(&mut event.publications, 0).meaning.source_scope = event.original_anchor
        }),
        C::CyclicSource => envelope.edit_restoration_header_for_test(|event, _| {
            let id = event.publications[0].outputs[0];
            origin(&mut event.publications, 0).meaning.source_scope = id;
        }),
        C::UnownedRelation => envelope.edit_restoration_header_for_test(|event, _| {
            origin(&mut event.publications, 0).relations.remove(0);
        }),
        C::UnownedEntity => envelope.edit_restoration_header_for_test(|event, _| {
            let p = origin(&mut event.publications, 0);
            p.members.remove(0);
            p.outputs.remove(0);
        }),
    }
}
