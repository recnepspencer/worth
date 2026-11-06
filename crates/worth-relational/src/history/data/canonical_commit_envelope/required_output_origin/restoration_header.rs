//! The canonical inverse has an exact fresh-generation anchor and a bijective roster.
use super::*;
use crate::{
    history::data::CodecRetirementClosureStop as Stop,
    publication::patch::data::RecordStructuralChange as Change, transactions::data::RecordRef,
};
use std::collections::{HashMap, HashSet};
use worth_execution::MapKernelContext;

impl CanonicalCommitEnvelope {
    pub(crate) fn validate_restoration_header(
        &self,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<(), Stop> {
        let Some(event) = self.authored_source_restoration() else {
            return Ok(());
        };
        self.validate_authored_restoration_components(
            event,
            self.coupled_authored_publication.is_some(),
            context,
        )
    }

    /// Expected linear work and scratch in patches, generations, publication
    /// membership and route text. Shared source ancestry is resolved once.
    pub(super) fn validate_authored_restoration_components(
        &self,
        event: &CanonicalAuthoredRestoration,
        allow_empty: bool,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<(), Stop> {
        let before = super::header_work::begin(self, context);
        let result = self.validate_indexed_restoration(event, allow_empty, context);
        super::header_work::finish(self, context, before);
        result
    }

    fn validate_indexed_restoration(
        &self,
        event: &CanonicalAuthoredRestoration,
        allow_empty: bool,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<(), Stop> {
        let charge = super::super::super::codec_retirement_closure::charge;
        if self.producer_cascade_retirement.is_some()
            || self.required_output_origin.is_some()
            || self.authored_cascade_retirement.is_some()
            || (!allow_empty && event.publications.is_empty())
            || event.original_anchor == event.anchor
        {
            return Err("invalid recorded authored restoration anchor".into());
        }
        let index = RestorationIndex::build(self, event, context)?;
        if !index
            .entities
            .contains(&(event.original_anchor, event.anchor))
            || !index.created.contains(&RecordRef::Entity(event.anchor))
        {
            return Err("invalid recorded authored restoration anchor".into());
        }
        let mut old_anchors = HashSet::new();
        let mut new_anchors = HashSet::new();
        for &(old, new) in &event.co_anchors {
            charge(context, 5)?;
            if old == event.original_anchor
                || new == event.anchor
                || !old_anchors.insert(old)
                || !new_anchors.insert(new)
                || !index.entities.contains(&(old, new))
                || !index.created.contains(&RecordRef::Entity(new))
            {
                return Err("restoration repeats or omits a co-anchor".into());
            }
        }
        let mut old_entities = HashSet::new();
        let mut new_entities = HashSet::new();
        for &(old, new) in &event.entities {
            charge(context, 3)?;
            if old == new || !old_entities.insert(old) || !new_entities.insert(new) {
                return Err("recorded restoration repeats or revives a native generation".into());
            }
        }
        let mut old_relations = HashSet::new();
        let mut new_relations = HashSet::new();
        for &(old, new) in &event.relations {
            charge(context, 4)?;
            if old == new
                || !old_relations.insert(old)
                || !new_relations.insert(new)
                || !index.created.contains(&RecordRef::Relation(new))
            {
                return Err("restoration repeats or invents a relation allocation".into());
            }
        }
        let mut edges = 0usize;
        let mut members = 0usize;
        let mut routes = HashSet::new();
        let mut outputs = HashSet::new();
        let mut rooted = index.anchors.clone();
        for origin in &event.publications {
            charge(
                context,
                origin.members.len() * 4 + origin.meaning.family.len() + 65,
            )?;
            self.validate_output_origin_header(origin)?;
            if !origin.source_scope_owned
                || !matches!(origin.publication, CanonicalOutputPublication::AuthoredRestoration { retirement, .. } if retirement == event.retirement)
                || origin.members.iter().any(|member| {
                    index.anchors.contains(&member.id)
                        || member.posture != crate::mvcc::RequiredOutputRolePosture::Create
                        || !new_entities.contains(&member.id)
                })
            {
                return Err("restoration omits exact recorded members".into());
            }
            charge(context, origin.relations.len())?;
            edges = edges
                .checked_add(origin.relations.len())
                .ok_or("restoration relation count overflows")?;
            if origin
                .relations
                .iter()
                .any(|edge| !new_relations.contains(&edge.id))
            {
                return Err("restoration relation has no recorded inverse".into());
            }
            members = members
                .checked_add(origin.members.len())
                .ok_or("restoration member count overflows")?;
            if !routes.insert((
                origin.meaning.family.as_str(),
                origin.meaning.source_scope,
                origin.meaning.source_partition,
            )) || origin.outputs.iter().any(|id| !outputs.insert(*id))
            {
                return Err("restoration repeats an original route or member".into());
            }
            index.validate_source(origin.meaning.source_scope, &mut rooted, context)?;
        }
        if edges != event.relations.len() {
            return Err("restoration adds an unowned relation".into());
        }
        if members.checked_add(event.co_anchors.len() + 1) != Some(event.entities.len()) {
            return Err("restoration adds an unowned entity".into());
        }
        Ok(())
    }
}

/// Preserve existential Created-patch membership, including duplicate targets.
/// Source ownership uses the last publication, matching the original scan.
struct RestorationIndex {
    created: HashSet<RecordRef>,
    entities: HashSet<(EntityId, EntityId)>,
    anchors: HashSet<EntityId>,
    source_by_output: HashMap<EntityId, EntityId>,
}

impl RestorationIndex {
    fn build(
        envelope: &CanonicalCommitEnvelope,
        event: &CanonicalAuthoredRestoration,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<Self, Stop> {
        let charge = super::super::super::codec_retirement_closure::charge;
        charge(
            context,
            envelope.patch.authoritative_record_patches.len()
                + event.entities.len()
                + event.co_anchors.len()
                + 1,
        )?;
        let created = envelope
            .patch
            .authoritative_record_patches
            .iter()
            .filter(|patch| patch.structural_change == Change::Created)
            .map(|patch| patch.target.clone())
            .collect();
        let entities = event.entities.iter().copied().collect();
        let anchors = std::iter::once(event.anchor)
            .chain(event.co_anchors.iter().map(|(_, id)| *id))
            .collect();
        let mut source_by_output = HashMap::new();
        for origin in &event.publications {
            charge(context, origin.outputs.len() + 1)?;
            source_by_output.extend(
                origin
                    .outputs
                    .iter()
                    .map(|id| (*id, origin.meaning.source_scope)),
            );
        }
        Ok(Self {
            created,
            entities,
            anchors,
            source_by_output,
        })
    }

    fn validate_source(
        &self,
        mut source: EntityId,
        rooted: &mut HashSet<EntityId>,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<(), Stop> {
        let charge = super::super::super::codec_retirement_closure::charge;
        let mut path = HashSet::new();
        loop {
            charge(context, 3)?;
            if rooted.contains(&source) {
                rooted.extend(path);
                return Ok(());
            }
            if !path.insert(source) {
                return Err("restored source ownership is cyclic".into());
            }
            source = *self
                .source_by_output
                .get(&source)
                .ok_or("restored publication has a foreign source")?;
        }
    }
}
