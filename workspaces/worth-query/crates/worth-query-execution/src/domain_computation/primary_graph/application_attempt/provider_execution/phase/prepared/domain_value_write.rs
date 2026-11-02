//! One publication seal for every realized domain field write, after expansion.
use crate::domain_computation::primary_graph::{
    application_attempt::effect_program::WorthQueryApplicationRealizedEffect as Effect,
    schema_layout::WorthQueryPrimaryGraphLayout as Layout,
    value_decode::{ValueDecodeSession, ValueDecodeState},
    WorthQueryApplicationCommitDenial as Denial, WorthQueryValueDecodeDenial as Stop,
};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_query_installation::facade::{
    AdmittedFieldBindingStop, ApplicationFieldBindingOwnerKind as Owner,
    WorthQueryInstalledApplicationValueBindingCatalog as Catalog,
};

pub(super) fn admit(
    execution: &ExecutionRequest<'_, '_>,
    bindings: &Catalog,
    layout: &Layout,
    effects: &[Effect],
    basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    attachments: &worth_relational::facade::content::RelationalContentAttachments,
) -> Result<(), Denial> {
    let mut content = basis
        .prepare_content_reads(execution, attachments)
        .map_err(|cause| Denial::domain_value_admission_rejected(Stop::Content(cause)))?;
    let state = ValueDecodeState::admitted(bindings.clone());
    let result = state
        .run(execution, |session| {
            let mut admission = WriteAdmission {
                session,
                bindings,
                execution,
                content: &mut content,
            };
            for effect in effects {
                session
                    .admit_write_quantities(1, 0)
                    .map_err(Denial::domain_value_admission_rejected)?;
                match effect {
                    Effect::CreateEntity { kind, fields, .. } => {
                        let name = layout
                            .record_name_admitted(Owner::Entity, *kind, &mut |work| {
                                session.admit_write_quantities(work, 0)
                            })
                            .map_err(Denial::domain_value_admission_rejected)?;
                        // Absence in the actual kind catalogue means an internal
                        // platform record, which has no application domain binding.
                        if let Some(name) = name {
                            admission.fields(Owner::Entity, name, fields)?;
                        }
                    }
                    Effect::UpdateEntity { entity, fields, .. } => {
                        admission.fields(Owner::Entity, entity, fields)?
                    }
                    Effect::PatchOptionalEntityFields { entity, fields, .. } => {
                        for (locator, write) in fields {
                            session
                                .admit_write_quantities(1, 0)
                                .map_err(Denial::domain_value_admission_rejected)?;
                            if let Some(value) = &write.value {
                                admission.field(Owner::Entity, entity, locator, value)?;
                            }
                        }
                    }
                    Effect::CreateRelation { kind, fields, .. } => {
                        let name = layout
                            .record_name_admitted(Owner::Relation, *kind, &mut |work| {
                                session.admit_write_quantities(work, 0)
                            })
                            .map_err(Denial::domain_value_admission_rejected)?;
                        if let Some(name) = name {
                            admission.fields(Owner::Relation, name, fields)?;
                        }
                    }
                    Effect::DeleteEntity { .. }
                    | Effect::DeleteRelation { .. }
                    | Effect::RemoveAspect { .. }
                    | Effect::Emit(_) => {}
                }
            }
            Ok(())
        })
        .map_err(Denial::domain_value_admission_rejected)?;
    result?;
    state
        .check(execution)
        .map_err(Denial::domain_value_admission_rejected)
}
struct WriteAdmission<'a, 'b, 'run, 'host> {
    session: &'a ValueDecodeSession<'b>,
    bindings: &'a Catalog,
    execution: &'a ExecutionRequest<'run, 'host>,
    content: &'a mut worth_relational::facade::content::RelationalContentReadClosure,
}
impl WriteAdmission<'_, '_, '_, '_> {
    fn fields(
        &mut self,
        owner: Owner,
        name: &str,
        fields: &std::collections::BTreeMap<AspectFieldLocator, AspectValue>,
    ) -> Result<(), Denial> {
        for (locator, value) in fields {
            self.session
                .admit_write_quantities(1, 0)
                .map_err(Denial::domain_value_admission_rejected)?;
            self.field(owner, name, locator, value)?;
        }
        Ok(())
    }
    fn field(
        &mut self,
        owner: Owner,
        name: &str,
        locator: &AspectFieldLocator,
        value: &AspectValue,
    ) -> Result<(), Denial> {
        let [field] = locator.field_path().fields() else {
            return Err(Denial::domain_value_admission_rejected(
                Stop::BindingContractMismatch,
            ));
        };
        let mut fund = |work, bytes| self.session.admit_write_quantities(work, bytes);
        let binding = match owner {
            Owner::Entity => self.bindings.field_admitted(
                name,
                locator.aspect().aspect_key().as_str(),
                field.as_str(),
                &mut fund,
            ),
            Owner::Relation => self.bindings.relation_field_admitted(
                name,
                locator.aspect().aspect_key().as_str(),
                field.as_str(),
                &mut fund,
            ),
        }
        .map_err(|cause| {
            Denial::domain_value_admission_rejected(match cause {
                AdmittedFieldBindingStop::Admission(stop) => stop,
                AdmittedFieldBindingStop::AccountingOverflow => Stop::AccountingOverflow,
            })
        })?;
        let Some(binding) = binding else {
            return Ok(());
        };
        // This same seal owns both canonical domain values and typed content fields.
        if binding
            .canonical_contract()
            .is_some_and(|contract| contract.is_domain_value())
        {
            self.session
                .validate_domain_write(binding, value)
                .map_err(Denial::domain_value_rejected)?;
        }
        if binding.scalar_family() == worth_foundational::facade::ScalarAspectType::ContentRef
            && binding.content_format().is_none()
        {
            return Err(Denial::domain_value_admission_rejected(Stop::Content(
                worth_relational::facade::content::RelationalContentDenial::FormatBindingMissing,
            )));
        }
        if let Some(format) = binding.content_format() {
            let AspectValue::ContentRef(identity) = value else {
                return Err(Denial::domain_value_admission_rejected(
                    Stop::BindingContractMismatch,
                ));
            };
            let frame = self
                .content
                .read(
                    self.execution,
                    *identity,
                    worth_foundational::facade::CanonicalDigestId::new(
                        format.declaration().identity,
                    ),
                )
                .map_err(|cause| Denial::domain_value_admission_rejected(Stop::Content(cause)))?;
            crate::domain_computation::primary_graph::content::require_contract(
                self.execution,
                &frame,
                format,
            )
            .map_err(|cause| Denial::domain_value_admission_rejected(Stop::Content(cause)))?;
        }
        Ok(())
    }
}
