//! Canonical signed subject and candidate identity; no runtime ordinals enter meaning.
use super::super::WorthQueryApplicationRealizedEffect as Effect;
use super::sealing::WorthQuerySignedCreationPreparationDenial as Denial;
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactSequence;
use std::collections::BTreeMap;
use worth_foundational::facade::{
    canonical_basis_value_for_aspect_value, AspectFieldLocator, AspectValue, CanonicalBasisDomain,
    CanonicalBasisEntry, CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue,
    CanonicalDigestId, CanonicalIntegerWidth,
};
use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;
use worth_relational::facade::{identity::EntityId, transactions::EntityReference};
const DOMAIN: CanonicalBasisDomain = CanonicalBasisDomain::Future("worth-query.signed-creation");
mod subject;
pub(super) use subject::certified_subject;

pub(super) struct IdentityEntries(Vec<CanonicalBasisEntry>);
impl IdentityEntries {
    pub(super) fn new() -> Self {
        Self(Vec::new())
    }
    pub(super) fn text(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.value(name, CanonicalBasisValue::ExactText(value.into().into()));
    }
    pub(super) fn digest(&mut self, name: &str, value: [u8; 32]) {
        self.value(
            name,
            CanonicalBasisValue::BytesDigest(CanonicalDigestId::new(value)),
        );
    }
    fn number(&mut self, name: impl Into<String>, value: u64) {
        self.value(
            name,
            CanonicalBasisValue::UnsignedInteger {
                width: CanonicalIntegerWidth::Bits64,
                value: u128::from(value),
            },
        );
    }
    fn value(&mut self, name: impl Into<String>, value: CanonicalBasisValue) {
        self.0.push(CanonicalBasisEntry::new(
            DOMAIN,
            CanonicalBasisLocus::Named(name.into().into()),
            CanonicalBasisEntryKind::Value,
            value,
        ));
    }
    pub(super) fn entity(&mut self, name: &str, entity: EntityId) {
        self.number(format!("{name}.partition"), entity.partition_value_u64());
        self.number(format!("{name}.slot"), entity.local_slot_value());
        self.number(
            format!("{name}.generation"),
            u64::from(entity.generation_value()),
        );
    }
    fn reference(&mut self, name: &str, reference: &EntityReference) -> Result<(), ()> {
        match reference {
            EntityReference::Existing(entity) => {
                self.text(format!("{name}.kind"), "existing");
                self.entity(name, *entity);
            }
            EntityReference::Created(created) => {
                self.text(format!("{name}.kind"), "created");
                self.number(
                    format!("{name}.partition"),
                    u64::from(created.partition_id.0),
                );
                self.number(
                    format!("{name}.entity-kind"),
                    u64::from(created.kind_id.as_u32()),
                );
                self.text(
                    format!("{name}.key"),
                    created.client_key.as_raw_str().ok_or(())?,
                );
            }
        }
        Ok(())
    }
    fn fields(
        &mut self,
        name: &str,
        fields: &BTreeMap<AspectFieldLocator, AspectValue>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<(), Denial> {
        self.number(format!("{name}.count"), fields.len() as u64);
        for (index, (locator, value)) in fields.iter().enumerate() {
            check_interruption(context)?;
            let prefix = format!("{name}.{index}");
            self.path(&prefix, locator, context)?;
            self.value(
                format!("{prefix}.value"),
                canonical_basis_value_for_aspect_value(value),
            );
        }
        Ok(())
    }
    fn path(
        &mut self,
        prefix: &str,
        locator: &AspectFieldLocator,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<(), Denial> {
        check_interruption(context)?;
        self.text(
            format!("{prefix}.aspect"),
            locator.aspect().aspect_key().as_str(),
        );
        self.number(
            format!("{prefix}.segments"),
            locator.field_path().fields().len() as u64,
        );
        for (index, field) in locator.field_path().fields().iter().enumerate() {
            check_interruption(context)?;
            self.text(format!("{prefix}.segment.{index}"), field.as_str());
        }
        Ok(())
    }
    pub(super) fn derive(
        self,
        rule: &str,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<
        ([u8; 32], WorthQueryCanonicalWorkEvidence),
        super::sealing::WorthQuerySignedCreationPreparationDenial,
    > {
        super::canonical::derive(self.0, DOMAIN, rule, request, context)
    }
}

pub(super) fn candidate(
    effects: &[Effect],
    request: &worth_execution::ExecutionRequest<'_, '_>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<
    ([u8; 32], WorthQueryCanonicalWorkEvidence),
    super::sealing::WorthQuerySignedCreationPreparationDenial,
> {
    let mut entries = IdentityEntries::new();
    check_interruption(context)?;
    entries.number("effect-count", effects.len() as u64);
    for (index, effect) in effects.iter().enumerate() {
        check_interruption(context)?;
        let prefix = format!("effect.{index}");
        match effect {
            Effect::CreateEntity {
                kind,
                key,
                fields,
                partition,
            } => {
                check_interruption(context)?;
                entries.text(format!("{prefix}.kind"), "create");
                entries.number(format!("{prefix}.entity-kind"), u64::from(kind.as_u32()));
                entries.text(format!("{prefix}.key"), key);
                match partition {
                    super::super::WorthQueryApplicationCreationPartition::Issued => {
                        entries.text(format!("{prefix}.partition-mode"), "issued")
                    }
                    super::super::WorthQueryApplicationCreationPartition::Context(partition) => {
                        entries.text(format!("{prefix}.partition-mode"), "context");
                        entries.number(format!("{prefix}.partition"), u64::from(partition.0));
                    }
                }
                entries.fields(&prefix, fields, context)?;
            }
            Effect::UpdateEntity {
                entity,
                entity_id,
                fields,
            } => {
                check_interruption(context)?;
                entries.text(format!("{prefix}.kind"), "update");
                entries.text(format!("{prefix}.entity-kind"), entity);
                entries.entity(&prefix, *entity_id);
                entries.fields(&prefix, fields, context)?;
            }
            Effect::PatchOptionalEntityFields {
                entity,
                entity_id,
                fields,
            } => {
                check_interruption(context)?;
                entries.text(format!("{prefix}.kind"), "optional");
                entries.text(format!("{prefix}.entity-kind"), entity);
                entries.entity(&prefix, *entity_id);
                entries.number(format!("{prefix}.count"), fields.len() as u64);
                for (index, (locator, write)) in fields.iter().enumerate() {
                    check_interruption(context)?;
                    let field = format!("{prefix}.{index}");
                    entries.path(&field, locator, context)?;
                    entries.value(
                        format!("{field}.value"),
                        write.value.as_ref().map_or(
                            CanonicalBasisValue::Null,
                            canonical_basis_value_for_aspect_value,
                        ),
                    );
                }
            }
            Effect::CreateRelation {
                kind,
                key,
                from,
                to,
            } => {
                check_interruption(context)?;
                entries.text(format!("{prefix}.kind"), "link");
                entries.number(format!("{prefix}.relation-kind"), u64::from(kind.as_u32()));
                entries.text(format!("{prefix}.key"), key);
                entries
                    .reference(&format!("{prefix}.from"), from)
                    .map_err(|_| {
                        super::sealing::WorthQuerySignedCreationPreparationDenial::Canonicalization
                    })?;
                entries
                    .reference(&format!("{prefix}.to"), to)
                    .map_err(|_| {
                        super::sealing::WorthQuerySignedCreationPreparationDenial::Canonicalization
                    })?;
            }
            // Native body declarations do not emit or delete. Do not silently
            // omit opaque retained payloads or future effect kinds from signing.
            Effect::DeleteEntity { .. } | Effect::DeleteRelation { .. } | Effect::Emit(_) => {
                return Err(
                    super::sealing::WorthQuerySignedCreationPreparationDenial::Canonicalization,
                )
            }
        }
    }
    entries.derive("worth-query-signed-creation-candidate-v1", request, context)
}

pub(super) fn subject(
    fields: &BTreeMap<AspectFieldLocator, AspectValue>,
    scope: EntityId,
    facts: &(impl FactSequence + ?Sized),
    request: &worth_execution::ExecutionRequest<'_, '_>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<
    ([u8; 32], WorthQueryCanonicalWorkEvidence),
    super::sealing::WorthQuerySignedCreationPreparationDenial,
> {
    let mut entries = IdentityEntries::new();
    check_interruption(context)?;
    entries.entity("scope", scope);
    entries.fields("subject", fields, context)?;
    for (index, fact) in facts.iter().enumerate() {
        check_interruption(context)?;
        if let super::super::WorthQueryApplicationObservedFact::SourceFieldRevision {
            entity_id,
            locator,
            native_revision,
        } = fact
        {
            let revision = native_revision.ok_or(
                super::sealing::WorthQuerySignedCreationPreparationDenial::CandidateMismatch,
            )?;
            let prefix = format!("dependency.{index}");
            entries.entity(&prefix, *entity_id);
            entries.path(&prefix, locator, context)?;
            entries.number(format!("{prefix}.revision"), revision.version().0);
            entries.text(
                format!("{prefix}.presence"),
                match revision.presence() {
                    worth_relational::facade::runtime::RelationalFieldPresence::Present => {
                        "present"
                    }
                    worth_relational::facade::runtime::RelationalFieldPresence::Absent => "absent",
                },
            );
        }
    }
    entries.derive("worth-query-signed-creation-subject-v1", request, context)
}

/// Poll the active request without accumulating a resource bill.
pub(in crate::domain_computation::primary_graph::application_attempt) fn check_interruption(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<(), Denial> {
    context.checkpoint(0).map_err(|cause| {
        Denial::Execution(
            crate::domain_computation::primary_graph::application_contribution::advancement_denial(
                worth_execution::WorkCeilingDenial::Stopped(cause),
            ),
        )
    })
}
