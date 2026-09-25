use std::collections::{BTreeMap, BTreeSet};

use crate::history::data::CommitId;
use crate::identity::data::VersionId;
use crate::indexes::data::{
    DerivedIndexGeneration, DerivedIndexGenerationId, DerivedIndexPublicationStatus,
};
use crate::schema::data::SchemaVersionId;

use super::remove_member;

#[derive(Debug, Clone, Default)]
pub(super) struct GenerationScope {
    ids: BTreeSet<DerivedIndexGenerationId>,
    published: GenerationStatusGroup,
    failed: GenerationStatusGroup,
}

impl GenerationScope {
    pub(super) fn insert(&mut self, generation: &DerivedIndexGeneration) {
        self.ids.insert(generation.generation_id);
        self.status_mut(is_published(generation)).insert(generation);
    }

    pub(super) fn remove(&mut self, generation: &DerivedIndexGeneration) {
        self.ids.remove(&generation.generation_id);
        self.status_mut(is_published(generation)).remove(generation);
    }

    pub(super) fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
    pub(super) fn latest(&self) -> Option<DerivedIndexGenerationId> {
        self.ids.last().copied()
    }
    pub(super) fn has_published(&self) -> bool {
        self.has_status(true)
    }
    pub(super) fn has_status(&self, published: bool) -> bool {
        !self.status(published).versions.is_empty()
    }

    pub(super) fn exact(
        &self,
        version: VersionId,
        schema: SchemaVersionId,
    ) -> Option<DerivedIndexGenerationId> {
        self.published
            .versions
            .get(&version)?
            .schemas
            .get(&schema)?
            .last()
            .copied()
    }

    pub(super) fn published_for_commit(
        &self,
        commit: CommitId,
        version: VersionId,
    ) -> Option<DerivedIndexGenerationId> {
        self.published
            .commits
            .get(&(commit, version))?
            .last()
            .copied()
    }

    pub(super) fn candidate(
        &self,
        published: bool,
        version: VersionId,
        schema: SchemaVersionId,
    ) -> Option<DerivedIndexGenerationId> {
        let group = self.status(published);
        if let Some(exact_version) = group.versions.get(&version) {
            return exact_version
                .schemas
                .get(&schema)
                .unwrap_or(&exact_version.ids)
                .last()
                .copied();
        }
        // With no exact version, schema preference still precedes version.
        group
            .schemas
            .get(&schema)
            .and_then(|entries| entries.last().map(|(_, id)| *id))
            .or_else(|| group.versions.last_key_value()?.1.ids.last().copied())
    }

    fn status(&self, published: bool) -> &GenerationStatusGroup {
        if published {
            &self.published
        } else {
            &self.failed
        }
    }

    fn status_mut(&mut self, published: bool) -> &mut GenerationStatusGroup {
        if published {
            &mut self.published
        } else {
            &mut self.failed
        }
    }
}

#[derive(Debug, Clone, Default)]
struct GenerationStatusGroup {
    versions: BTreeMap<VersionId, GenerationVersion>,
    schemas: BTreeMap<SchemaVersionId, BTreeSet<(VersionId, DerivedIndexGenerationId)>>,
    commits: BTreeMap<(CommitId, VersionId), BTreeSet<DerivedIndexGenerationId>>,
}

impl GenerationStatusGroup {
    fn insert(&mut self, generation: &DerivedIndexGeneration) {
        let version = generation.applicability.version_id;
        let schema = generation.applicability.schema_version;
        let id = generation.generation_id;
        let selected = self.versions.entry(version).or_default();
        selected.ids.insert(id);
        selected.schemas.entry(schema).or_default().insert(id);
        self.schemas
            .entry(schema)
            .or_default()
            .insert((version, id));
        self.commits
            .entry((generation.source_commit_id, version))
            .or_default()
            .insert(id);
    }

    fn remove(&mut self, generation: &DerivedIndexGeneration) {
        let version = generation.applicability.version_id;
        let schema = generation.applicability.schema_version;
        let id = generation.generation_id;
        if let Some(selected) = self.versions.get_mut(&version) {
            selected.ids.remove(&id);
            remove_member(&mut selected.schemas, &schema, id);
            if selected.ids.is_empty() {
                self.versions.remove(&version);
            }
        }
        if let Some(entries) = self.schemas.get_mut(&schema) {
            entries.remove(&(version, id));
            if entries.is_empty() {
                self.schemas.remove(&schema);
            }
        }
        remove_member(
            &mut self.commits,
            &(generation.source_commit_id, version),
            id,
        );
    }
}

#[derive(Debug, Clone, Default)]
struct GenerationVersion {
    ids: BTreeSet<DerivedIndexGenerationId>,
    schemas: BTreeMap<SchemaVersionId, BTreeSet<DerivedIndexGenerationId>>,
}

fn is_published(generation: &DerivedIndexGeneration) -> bool {
    generation.status == DerivedIndexPublicationStatus::Published
}
