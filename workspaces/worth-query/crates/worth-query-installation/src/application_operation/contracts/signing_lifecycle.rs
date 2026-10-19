//! Native consequence closure derived from signed-subject declarations, not per-writer registration.
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
        maximum_records: Option<usize>,
        body: Option<
            &worth_query_declaration::facade::application_schema::ApplicationOperationBodyProgram,
        >,
    ) -> Result<Self, ()> {
        let mut groups = BTreeMap::<(&str, &Spec), WorthQueryInstalledSigningLifecycleRule>::new();
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
            let rule = groups.entry((creation.entity(), spec)).or_insert_with(|| {
                WorthQueryInstalledSigningLifecycleRule {
                    evidence_entity: creation.entity().into(),
                    provenance: creation
                        .signing()
                        .expect("readmitted lifecycle")
                        .provenance()
                        .clone(),
                    spec: spec.clone(),
                    subjects: Vec::new(),
                }
            });
            for field in creation.fields() {
                if let Field::CapturedScopeField {
                    source,
                    role: Role::Subject,
                    ..
                } = field
                    && !rule.subjects.contains(source)
                {
                    rule.subjects.push(source.clone());
                }
            }
        }
        let rules: Vec<_> = groups
            .into_values()
            .filter(|rule| {
                touches.scopes().iter().any(|touch| match touch {
                    Touch::DeleteEntity(scope) => scope.entity() == rule.spec.source_entity,
                    Touch::WriteField(scope) => rule.subjects.iter().any(|field| {
                        body.is_none_or(|body| {
                            body.scope_fields()
                                .iter()
                                .any(|step| step.write_target().is_some() && step.field() == field)
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
                .checked_mul(if body.is_some() {
                    1
                } else {
                    maximum_records.ok_or(())?
                })
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
