//! First-class installed history meaning, separate from ordinary graph projection.
use super::{
    binding::WorthQueryCompiledApplicationQueryBinding,
    WorthQueryApplicationQueryInstallationDenial,
    WorthQueryApplicationQueryInstallationDenialKind as Kind,
    WorthQueryInstalledApplicationQueryBinding,
};
use crate::application_schema::WorthQueryInstalledApplicationSchema;
use std::{collections::BTreeMap, marker::PhantomData, sync::Arc};
use worth_foundational::facade::AspectKey;
use worth_query_declaration::facade::{
    application_query::{
        ApplicationHistoryQueryBinding, ApplicationHistoryTarget, ApplicationQueryBinding,
        ApplicationQueryDisclosureContract, ErasedApplicationHistoryQueryDefinition,
    },
    application_schema::{
        ApplicationFieldBindingOwnerKind, ApplicationSchema, ApplicationSchemaBindingIdentity,
        ApplicationSchemaMember,
    },
};

pub(crate) struct CompiledApplicationHistoryQuery {
    schema: ApplicationSchemaBindingIdentity,
    definition: ErasedApplicationHistoryQueryDefinition,
    subject: Arc<WorthQueryCompiledApplicationQueryBinding>,
    aspects: Arc<[AspectKey]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationHistoryInstallationDenial {
    HistoryNotInstalled,
    MarkerMismatch,
    SubjectBinding(WorthQueryApplicationQueryInstallationDenial),
}

/// Exact installed history/query association; it carries no subject, root, or read permission.
pub struct WorthQueryInstalledApplicationHistoryQuery<
    Schema: ApplicationSchema,
    History: ApplicationHistoryQueryBinding<Schema>,
> {
    compiled: Arc<CompiledApplicationHistoryQuery>,
    subject: WorthQueryInstalledApplicationQueryBinding<Schema, History::SubjectRead>,
    _history: PhantomData<fn() -> History>,
}

impl<Schema: ApplicationSchema, History: ApplicationHistoryQueryBinding<Schema>>
    WorthQueryInstalledApplicationHistoryQuery<Schema, History>
{
    pub fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        &self.compiled.schema
    }
    pub fn definition(&self) -> &ErasedApplicationHistoryQueryDefinition {
        &self.compiled.definition
    }
    pub fn subject_read(
        &self,
    ) -> &WorthQueryInstalledApplicationQueryBinding<Schema, History::SubjectRead> {
        &self.subject
    }
    pub fn target(&self) -> &ApplicationHistoryTarget {
        &self.compiled.definition.parts().target
    }
    pub fn selected_aspects(&self) -> &[AspectKey] {
        &self.compiled.aspects
    }
    pub fn metadata_disclosure(&self) -> &ApplicationQueryDisclosureContract {
        &self.compiled.definition.parts().metadata_disclosure
    }
}

impl<Schema: ApplicationSchema> WorthQueryInstalledApplicationSchema<Schema> {
    /// Resolve actual typed inventory once at installation, before request use.
    pub fn installed_history_query<History: ApplicationHistoryQueryBinding<Schema>>(
        &self,
    ) -> Result<
        WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
        WorthQueryApplicationHistoryInstallationDenial,
    > {
        use WorthQueryApplicationHistoryInstallationDenial as D;
        if !self
            .member_provenance
            .admits_history::<History, History::SubjectRead>(
                History::IDENTITY,
                &History::PORTABLE_TYPE_IDENTITY,
            )
        {
            return Err(D::MarkerMismatch);
        }
        let compiled = self
            .query_catalog
            .get_history(History::IDENTITY)
            .ok_or(D::HistoryNotInstalled)?;
        if compiled.definition.parts().history_type != History::PORTABLE_TYPE_IDENTITY
            || !compiled
                .definition
                .parts()
                .subject_read
                .matches_descriptor(&History::SubjectRead::descriptor())
        {
            return Err(D::MarkerMismatch);
        }
        let subject = self
            .installed_query_binding::<History::SubjectRead>()
            .map_err(D::SubjectBinding)?;
        if subject.query().identity() != compiled.subject.query_identity() {
            return Err(D::MarkerMismatch);
        }
        Ok(WorthQueryInstalledApplicationHistoryQuery {
            compiled,
            subject,
            _history: PhantomData,
        })
    }
}

pub(crate) fn compile<Schema: ApplicationSchema>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    subjects: &BTreeMap<String, Arc<WorthQueryCompiledApplicationQueryBinding>>,
) -> Result<
    BTreeMap<String, Arc<CompiledApplicationHistoryQuery>>,
    WorthQueryApplicationQueryInstallationDenial,
> {
    let mut histories = BTreeMap::new();
    for member in schema.installed_declaration().members() {
        let ApplicationSchemaMember::ApplicationHistoryQuery { definition } = member else {
            continue;
        };
        let parts = definition.parts();
        let denied = || {
            WorthQueryApplicationQueryInstallationDenial::new(
                Kind::QueryMeaningChanged,
                &parts.name,
            )
        };
        definition.validate().map_err(|_| denied())?;
        let subject = subjects
            .get(parts.subject_read.parts().binding.as_str())
            .filter(|binding| parts.subject_read.matches_descriptor(binding.descriptor()))
            .ok_or_else(denied)?;
        let (owner, name) = match &parts.target {
            ApplicationHistoryTarget::Entity { entity } => {
                (ApplicationFieldBindingOwnerKind::Entity, entity)
            }
            ApplicationHistoryTarget::Relation { relation, .. } => {
                (ApplicationFieldBindingOwnerKind::Relation, relation)
            }
        };
        let aspects = parts
            .aspects
            .iter()
            .map(|selected| {
                let actual = schema
                    .native_contracts()
                    .aspect_for_owner(owner, name, &selected.aspect)
                    .ok_or_else(denied)?;
                if actual.contract().identity() != selected.identity
                    || actual.contract().revision() != selected.revision
                {
                    return Err(denied());
                }
                Ok(actual.contract().key().clone())
            })
            .collect::<Result<Vec<_>, _>>()?;
        let compiled = CompiledApplicationHistoryQuery {
            schema: schema.binding_identity(),
            definition: definition.clone(),
            subject: Arc::clone(subject),
            aspects: aspects.into(),
        };
        if histories
            .insert(parts.name.clone(), Arc::new(compiled))
            .is_some()
        {
            return Err(denied());
        }
    }
    Ok(histories)
}
