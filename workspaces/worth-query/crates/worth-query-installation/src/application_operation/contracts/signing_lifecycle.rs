//! Native consequence closure derived from signed-subject declarations, not per-writer registration.
#[cfg(test)]
mod tests;
use crate::{
    application_schema::WorthQueryInstalledApplicationSchemaContractCatalog as Catalog,
    domain_operation::{
        WorthQueryOperationEntityTouchScope, WorthQueryOperationFieldTouchScope,
        WorthQueryOperationRelationTouchScope, WorthQueryOperationTouchContract as Contract,
        WorthQueryOperationTouchScope as Touch,
    },
};
use std::collections::BTreeMap;
use worth_foundational::facade::{CanonicalFieldPath, FieldKey};
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationCandidateCardinalityCeiling, ApplicationCandidateRequirements,
        ApplicationCandidateResourceCeiling,
    },
    application_schema::{
        ApplicationFieldBindingLocus as Locus, ApplicationOperationBodyCaptureRole as Role,
        ApplicationOperationBodyCreatedField as Field,
        ApplicationOperationBodySigningLifecycleSpec as Spec, ApplicationSchemaBindingIdentity,
        ApplicationSchemaMember as Member, ApplicationSigningSupersessionParts,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledSigningLifecycleRule {
    evidence_entity: String,
    provenance: Locus,
    spec: Spec,
    subjects: Vec<Locus>,
    paths: Vec<worth_query_declaration::facade::application_schema::ApplicationOperationBodyPathFieldSource>,
    maximum_subjects: usize,
}
impl WorthQueryInstalledSigningLifecycleRule {
    pub fn evidence_entity(&self) -> &str {
        &self.evidence_entity
    }
    pub fn provenance(&self) -> &Locus {
        &self.provenance
    }
    pub fn spec(&self) -> &Spec {
        &self.spec
    }
    pub fn subjects(&self) -> &[Locus] {
        &self.subjects
    }
    pub fn paths(&self) -> &[worth_query_declaration::facade::application_schema::ApplicationOperationBodyPathFieldSource]{
        &self.paths
    }
    pub fn maximum_subjects(&self) -> usize {
        self.maximum_subjects
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::application_operation) struct SigningLifecycleClosure {
    pub(in crate::application_operation) rules: Vec<WorthQueryInstalledSigningLifecycleRule>,
    pub(in crate::application_operation) publication_touches: Contract,
    pub(in crate::application_operation) ceiling: Option<ApplicationCandidateRequirements>,
}

impl SigningLifecycleClosure {
    pub(in crate::application_operation) fn derive(
        binding: &ApplicationSchemaBindingIdentity,
        catalog: &Catalog,
        members: &[Member],
        operation: &str,
        touches: &Contract,
        maximum_records: usize,
        body: Option<
            &worth_query_declaration::facade::application_schema::ApplicationOperationBodyProgram,
        >,
    ) -> Result<Self, ()> {
        let mut groups = BTreeMap::<
            (&str, &Spec),
            (Option<usize>, WorthQueryInstalledSigningLifecycleRule),
        >::new();
        for member in members {
            let Member::OperationBody { body, .. } = member else {
                continue;
            };
            let Some(creation) = body.creation() else {
                continue;
            };
            let Some(spec) = creation.signing_lifecycle() else {
                continue;
            };
            let (bound, rule) = groups.entry((creation.entity(), spec)).or_insert_with(|| {
                (
                    None,
                    WorthQueryInstalledSigningLifecycleRule {
                        evidence_entity: creation.entity().into(),
                        provenance: creation
                            .signing()
                            .expect("readmitted lifecycle")
                            .provenance()
                            .clone(),
                        spec: spec.clone(),
                        subjects: Vec::new(),
                        paths: Vec::new(),
                        maximum_subjects: 1,
                    },
                )
            });
            if let Some(authored) = creation.subject_impact_bound() {
                *bound = Some(bound.unwrap_or(0).max(authored.get()));
            }
            for field in creation.fields() {
                if let Field::CapturedPathField {
                    source,
                    role: Role::Subject,
                    ..
                } = field
                {
                    if !rule.paths.contains(source) {
                        rule.paths.push(source.clone());
                    }
                    if !rule.subjects.contains(&source.field) {
                        rule.subjects.push(source.field.clone());
                    }
                }
                if let Field::CapturedScopeField {
                    source,
                    source_presence,
                    role: Role::Subject,
                    ..
                } = field
                {
                    if !rule.subjects.contains(source) {
                        rule.subjects.push(source.clone());
                    }
                    use worth_query_declaration::facade::{
                        application_query::{
                            ApplicationQueryRootPathMeaning,
                            WorthQueryPortableApplicationQueryRootPathParts,
                        },
                        application_schema::ApplicationOperationBodyPathFieldSource,
                    };
                    let path = ApplicationOperationBodyPathFieldSource {
                        path: ApplicationQueryRootPathMeaning::from_untrusted_parts(
                            WorthQueryPortableApplicationQueryRootPathParts {
                                start_entity: spec.source_entity.clone(),
                                terminal_entity: spec.source_entity.clone(),
                                steps: Vec::new(),
                                guards: Vec::new(),
                            },
                        ),
                        field: source.clone(),
                        presence: *source_presence,
                    };
                    if !rule.paths.contains(&path) {
                        rule.paths.push(path);
                    }
                }
            }
        }
        let rules: Vec<_> = groups
            .into_values()
            .map(|(bound, mut rule)| {
                rule.maximum_subjects =
                    bound.unwrap_or(if body.is_some() { 1 } else { maximum_records });
                rule
            })
            .filter(|rule| {
                touches.scopes().iter().any(|touch| match touch {
                    Touch::DeleteEntity(scope) => {
                        scope.entity() == rule.spec.source_entity
                            || rule.paths.iter().any(|path| {
                                path.path
                                    .steps()
                                    .iter()
                                    .any(|step| step.child_entity() == scope.entity())
                            })
                    }
                    Touch::LinkRelation(scope) => rule.paths.iter().any(|path| {
                        path.path.steps().iter().any(|step| {
                            step.relation() == scope.relation()
                                && body
                                    .is_none_or(|body| link_can_change_existing_anchor(body, step))
                        })
                    }),
                    Touch::UnlinkRelation(scope) => rule.paths.iter().any(|path| {
                        path.path
                            .steps()
                            .iter()
                            .any(|step| step.relation() == scope.relation())
                    }),
                    Touch::WriteField(scope) => rule.subjects.iter().any(|field| {
                        body.is_none_or(|body| {
                            body.scope_fields()
                                .iter()
                                .any(|step| step.write_target().is_some() && step.field() == field)
                                || body
                                    .path_value_assignments()
                                    .iter()
                                    .any(|step| &step.target.field == field)
                        }) && scope.entity() == field.entity()
                            && scope.contract().key().as_str() == field.aspect()
                            && scope
                                .field_path()
                                .fields()
                                .iter()
                                .any(|key| key.as_str() == field.field())
                    }),
                    _ => false,
                })
            })
            .collect();
        let mut publication = touches.scopes().to_vec();
        let mut records = 0usize;
        let mut retained = 0usize;
        for rule in &rules {
            let spec = &rule.spec;
            let count = spec
                .maximum_evidence
                .checked_mul(rule.maximum_subjects)
                .ok_or(())?;
            records = records.checked_add(count).ok_or(())?;
            // Native scalar quote plus the actual fixed effect keys/locators. No payload values are copied into the event.
            let bytes = ApplicationSigningSupersessionParts::quote(operation, &rule.subjects)
                .ok_or(())?
                .checked_add(1024)
                .ok_or(())?;
            retained = retained
                .checked_add(count.checked_mul(bytes).ok_or(())?)
                .ok_or(())?;
            let field = &spec.consequence_field;
            let aspect = catalog.aspect(field.entity(), field.aspect()).ok_or(())?;
            for touch in [
                Touch::CreateEntity(WorthQueryOperationEntityTouchScope::new(
                    binding.clone(),
                    field.entity().into(),
                )),
                Touch::WriteField(WorthQueryOperationFieldTouchScope::new(
                    binding.clone(),
                    field.entity().into(),
                    aspect,
                    CanonicalFieldPath::single(FieldKey::new(field.field()).ok_or(())?),
                )),
                Touch::LinkRelation(WorthQueryOperationRelationTouchScope::new(
                    binding.clone(),
                    spec.consequence_relation.clone(),
                    rule.evidence_entity.clone(),
                    field.entity().into(),
                )),
                Touch::UnlinkRelation(WorthQueryOperationRelationTouchScope::new(
                    binding.clone(),
                    spec.current_relation.clone(),
                    spec.source_entity.clone(),
                    rule.evidence_entity.clone(),
                )),
            ] {
                if !publication.contains(&touch) {
                    publication.push(touch);
                }
            }
        }
        let ceiling = (!rules.is_empty()).then(|| {
            ApplicationCandidateRequirements::fixed_shape(
                ApplicationCandidateCardinalityCeiling::fixed(
                    records, 0, records, records, records, 0,
                ),
                ApplicationCandidateResourceCeiling::representation_bytes(retained),
            )
        });
        let publication_touches = if publication.is_empty() {
            Contract::NotRequired
        } else {
            Contract::Declared {
                graph_roles: touches.graph_roles().to_vec(),
                scopes: publication,
            }
        };
        Ok(Self {
            rules,
            publication_touches,
            ceiling,
        })
    }
}

/// A scope-to-created link cannot change an old reverse capture anchored at the
/// fresh target (and symmetrically for created-to-scope forward captures).
/// Replacement unlinking is checked separately because it removes an old edge.
fn link_can_change_existing_anchor(
    body: &worth_query_declaration::facade::application_schema::ApplicationOperationBodyProgram,
    step: &worth_query_declaration::facade::application_query::ApplicationQueryRootPathStep,
) -> bool {
    use worth_query_declaration::facade::{
        application_query::ApplicationQueryRootPathDirection as Path,
        application_schema::ApplicationOperationBodyCreationLinkDirection as Link,
    };
    body.reference_inputs()
        .iter()
        .any(|reference| reference.relation == step.relation())
        || body.creation().is_none_or(|creation| {
            creation
                .initialization_sites()
                .flat_map(|site| {
                    site.links()
                        .iter()
                        .chain(site.path_links().iter().map(|path| &path.link))
                })
                .any(|link| {
                    link.relation == step.relation()
                        && matches!(
                            (link.direction, step.direction()),
                            (Link::FromScope, Path::Forward) | (Link::ToScope, Path::Reverse)
                        )
                })
        })
}

#[cfg(test)]
mod link_touch_tests;
