//! Corrupt descriptive headers while preserving the enclosing genuine authority.
use super::{
    CanonicalAuthoredRestoration, CanonicalAuthoredRetirement, CanonicalCommitEnvelopeBody,
};
use crate::history::{data::CanonicalCommitEnvelope, OutputOriginHeaderCorruptionForTest};
use crate::transactions::data::RecordRef;
use std::sync::Arc;
impl CanonicalCommitEnvelope {
    pub(crate) fn corrupt_output_origin_header_for_test(
        &mut self,
        corruption: OutputOriginHeaderCorruptionForTest,
    ) {
        corrupt(&mut self.body, corruption);
    }
}

fn corrupt(
    envelope: &mut CanonicalCommitEnvelopeBody,
    corruption: OutputOriginHeaderCorruptionForTest,
) {
    use OutputOriginHeaderCorruptionForTest as C;
    if matches!(
        corruption,
        C::CascadeEmptyPublicationRoster
            | C::CascadeRepeatedAuthoredAnchor
            | C::CascadeUnownedPublication
            | C::CascadeIncompleteOriginMembership
            | C::CascadeRepeatedRoute
            | C::CascadeForeignSource
            | C::CascadeCyclicSource
    ) {
        let event = envelope
            .authored_cascade_retirement
            .as_ref()
            .expect("lawful cascade");
        let mut event = CanonicalAuthoredRetirement {
            _decoded_memory: event._decoded_memory.clone(),
            anchor: event.anchor,
            owned_members: event.owned_members,
            birth: event.birth,
            co_anchors: event.co_anchors.clone(),
            publications: event.publications.clone(),
        };
        match corruption {
            C::CascadeEmptyPublicationRoster => event.publications.clear(),
            C::CascadeRepeatedAuthoredAnchor => event.co_anchors.push(event.co_anchors[0]),
            C::CascadeUnownedPublication => {
                origin(&mut event.publications, 0).source_scope_owned = false
            }
            C::CascadeIncompleteOriginMembership => {
                origin(&mut event.publications, 0).producer.clear()
            }
            C::CascadeRepeatedRoute => {
                let source = event.publications[0].meaning.source_scope;
                origin(&mut event.publications, 1).meaning.source_scope = source;
            }
            C::CascadeForeignSource => {
                // Deliberately move the source outside the recorded generation roster.
                let source = event.publications[0].members[0].id;
                origin(&mut event.publications, 0).meaning.source_scope =
                    crate::identity::data::EntityId::new(
                        source.partition_id,
                        source.local_slot_value(),
                        source.generation_value() + 1024,
                    );
            }
            C::CascadeCyclicSource => {
                let source = event.publications[0].outputs[0];
                origin(&mut event.publications, 0).meaning.source_scope = source;
            }
            _ => unreachable!("cascade corruption classified above"),
        }
        envelope.authored_cascade_retirement = Some(Arc::new(event));
        return;
    }
    let event = envelope
        .authored_cascade_restoration
        .as_ref()
        .expect("lawful restoration");
    let mut event = CanonicalAuthoredRestoration {
        _decoded_memory: event._decoded_memory.clone(),
        owned_members: event.owned_members,
        retirement: event.retirement,
        original_anchor: event.original_anchor,
        anchor: event.anchor,
        co_anchors: event.co_anchors.clone(),
        entities: event.entities.clone(),
        relations: event.relations.clone(),
        publications: event.publications.clone(),
    };
    match corruption {
        C::RevivedAnchor => event.original_anchor = event.anchor,
        C::EmptyPublicationRoster => event.publications.clear(),
        C::MissingAnchorMapping => event.entities.retain(|(_, id)| *id != event.anchor),
        C::AnchorWithoutCreatePatch => envelope
            .patch
            .authoritative_record_patches
            .retain(|p| p.target != RecordRef::Entity(event.anchor)),
        C::RepeatedCoAnchors => event.co_anchors.push(event.co_anchors[0]),
        C::MissingCoAnchorMapping => {
            let pair = event.co_anchors[0];
            event.entities.retain(|p| *p != pair);
        }
        C::DuplicateEntityGenerations => {
            let n = event.entities.len();
            event.entities[n - 1].0 = event.entities[n - 2].0;
        }
        C::RevivedEntityGenerations => {
            let n = event.entities.len();
            event.entities[n - 1].0 = event.entities[n - 1].1;
        }
        C::RepeatedRelationGenerations => event.relations[1].0 = event.relations[0].0,
        C::RelationWithoutCreatePatch => {
            let id = event.relations[0].1;
            envelope
                .patch
                .authoritative_record_patches
                .retain(|p| p.target != RecordRef::Relation(id));
        }
        C::IncompleteOriginMembership => origin(&mut event.publications, 0).producer.clear(),
        C::UnownedPublication => origin(&mut event.publications, 0).source_scope_owned = false,
        C::MissingMemberMapping => {
            let id = event.publications[0].members[0].id;
            event.entities.retain(|(_, new)| *new != id);
        }
        C::MissingRelationInverse => {
            event.relations.remove(0);
        }
        C::RepeatedRoute => {
            let source = event.publications[0].meaning.source_scope;
            origin(&mut event.publications, 1).meaning.source_scope = source;
        }
        C::RepeatedOutputMember => {
            let id = event.publications[0].outputs[0];
            let p = origin(&mut event.publications, 1);
            p.outputs[0] = id;
            p.members[0].id = id;
            p.outputs.sort();
            p.members.sort_by_key(|member| member.id);
        }
        C::ForeignSource => {
            origin(&mut event.publications, 0).meaning.source_scope = event.original_anchor
        }
        C::CyclicSource => {
            let id = event.publications[0].outputs[0];
            origin(&mut event.publications, 0).meaning.source_scope = id;
        }
        C::UnownedRelation => {
            origin(&mut event.publications, 0).relations.remove(0);
        }
        C::UnownedEntity => {
            let p = origin(&mut event.publications, 0);
            p.members.remove(0);
            p.outputs.remove(0);
        }
        _ => unreachable!("restoration corruption classified above"),
    }
    envelope.authored_cascade_restoration = Some(Arc::new(event));
}

// These are descriptive canonical headers, not the envelope's diagnostic or
// authority-bearing fields. Keep the enclosing genuine envelope intact.
fn copy_header<T: serde::Serialize + serde::de::DeserializeOwned>(value: &T) -> T {
    rmp_serde::from_slice(&rmp_serde::to_vec_named(value).expect("header encodes"))
        .expect("descriptive header decodes")
}
fn origin(
    publications: &mut [Arc<crate::history::data::CanonicalRequiredOutputOrigin>],
    index: usize,
) -> &mut crate::history::data::CanonicalRequiredOutputOrigin {
    let original = &publications[index];
    let mut copied: crate::history::data::CanonicalRequiredOutputOrigin =
        copy_header(original.as_ref());
    // Carry the real issuer; serde never reconstructs this proof from a projection.
    copied.issuer = original.issuer.clone();
    publications[index] = Arc::new(copied);
    Arc::get_mut(&mut publications[index]).expect("owned corrupted header")
}
