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
) -> Result<(), Denial> {
    let state = ValueDecodeState::admitted(bindings.clone());
    let result = state
        .run(execution, |session| {
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
                            fields_admitted(session, bindings, Owner::Entity, name, fields.iter())?;
                        }
                    }
                    Effect::UpdateEntity { entity, fields, .. } => {
                        fields_admitted(session, bindings, Owner::Entity, entity, fields.iter())?
                    }
                    Effect::PatchOptionalEntityFields { entity, fields, .. } => {
                        for (locator, write) in fields {
                            session
                                .admit_write_quantities(1, 0)
                                .map_err(Denial::domain_value_admission_rejected)?;
                            if let Some(value) = &write.value {
                                field_admitted(
                                    session,
                                    bindings,
                                    Owner::Entity,
                                    entity,
                                    locator,
                                    value,
                                )?;
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
                            fields_admitted(
                                session,
                                bindings,
                                Owner::Relation,
                                name,
                                fields.iter(),
                            )?;
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
fn fields_admitted<'a>(
    session: &ValueDecodeSession<'_>,
    bindings: &Catalog,
    owner: Owner,
    name: &str,
    fields: impl Iterator<Item = (&'a AspectFieldLocator, &'a AspectValue)>,
) -> Result<(), Denial> {
    for (locator, value) in fields {
        session
            .admit_write_quantities(1, 0)
            .map_err(Denial::domain_value_admission_rejected)?;
        field_admitted(session, bindings, owner, name, locator, value)?;
    }
    Ok(())
}
fn field_admitted(
    session: &ValueDecodeSession<'_>,
    bindings: &Catalog,
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
    let mut fund = |work, bytes| session.admit_write_quantities(work, bytes);
    let binding = match owner {
        Owner::Entity => bindings.field_admitted(
            name,
            locator.aspect().aspect_key().as_str(),
            field.as_str(),
            &mut fund,
        ),
        Owner::Relation => bindings.relation_field_admitted(
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
    // The immutable installed catalogue is the authority for domain membership.
    // Internal fields have no entry. Structured artifacts incur no round trip.
    if let Some(binding) = binding.filter(|binding| {
        binding
            .canonical_contract()
            .is_some_and(|contract| contract.is_domain_value())
    }) {
        session
            .validate_domain_write(binding, value)
            .map_err(Denial::domain_value_rejected)?;
    }
    Ok(())
}
