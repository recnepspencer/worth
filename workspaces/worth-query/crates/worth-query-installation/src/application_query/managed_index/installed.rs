//! Typed installation reissue retains the actual immutable catalog entry.
use super::{compiled::CompiledManagedIndex, *};
use crate::application_query::WorthQueryInstalledApplicationQueryBinding;
use std::marker::PhantomData;
use worth_query_declaration::facade::application_query::{
    ApplicationManagedIndexBinding, ApplicationQueryBinding,
};

/// One exact installed provider/source association; it grants no root or read permission.
pub struct WorthQueryInstalledManagedIndex<
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
> {
    compiled: Arc<CompiledManagedIndex>,
    source: Arc<WorthQueryInstalledApplicationQueryBinding<S, I::SourceBinding>>,
    marker: PhantomData<fn() -> I>,
}

impl<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>> Clone
    for WorthQueryInstalledManagedIndex<S, I>
{
    fn clone(&self) -> Self {
        Self {
            compiled: Arc::clone(&self.compiled),
            source: Arc::clone(&self.source),
            marker: PhantomData,
        }
    }
}

impl<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>>
    WorthQueryInstalledManagedIndex<S, I>
{
    pub fn binding_identity(
        &self,
    ) -> &worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity
    {
        &self.compiled.schema
    }
    pub fn definition(&self) -> &ErasedApplicationManagedIndexDefinition {
        self.compiled.recipe.meaning()
    }
    pub fn source_read(&self) -> &WorthQueryInstalledApplicationQueryBinding<S, I::SourceBinding> {
        &self.source
    }

    /// Compare the actual installed Source entry before its execution is retained.
    pub fn admits_source(
        &self,
        source: &crate::application_query::WorthQueryInstalledBoundQuery<S, I::SourceBinding>,
    ) -> bool {
        source.is_compiled_query(&self.compiled.source.query())
    }

    /// Equal descriptive names do not substitute for the actual installed entry.
    pub fn same_installation(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.compiled, &other.compiled)
    }

    /// Revalidate the actual receiving catalog, including its source and parent owners.
    pub fn is_installed_in(&self, schema: &WorthQueryInstalledApplicationSchema<S>) -> bool {
        schema
            .query_catalog
            .get_managed_index(&self.compiled.recipe.meaning().parts().name)
            .is_some_and(|actual| Arc::ptr_eq(&self.compiled, &actual))
    }

    pub fn admits_parent<Q: ApplicationQueryBinding<S>>(
        &self,
        parent: &WorthQueryInstalledApplicationQueryBinding<S, Q>,
    ) -> bool {
        self.compiled
            .parents
            .get(parent.identity())
            .is_some_and(|actual| parent.query().is_compiled_query(&actual.query()))
    }
}

impl<S: ApplicationSchema> WorthQueryInstalledApplicationSchema<S> {
    /// Resolve a typed owner during setup; requests reuse this genuine installed leaf.
    pub fn installed_managed_index<I: ApplicationManagedIndexBinding<S>>(
        &self,
    ) -> Result<WorthQueryInstalledManagedIndex<S, I>, Denial> {
        let compiled = self
            .query_catalog
            .get_managed_index(I::NAME)
            .ok_or_else(|| denied(Kind::InvalidManagedIndexDefinition, I::NAME))?;
        if compiled.schema != self.binding_identity() || !compiled.recipe.matches::<S, I>() {
            return Err(denied(Kind::ManagedIndexBindingMeaningChanged, I::NAME));
        }
        let source = self.installed_query_binding::<I::SourceBinding>()?;
        if !source.query().is_compiled_query(&compiled.source.query()) {
            return Err(denied(Kind::ManagedIndexBindingMeaningChanged, I::NAME));
        }
        Ok(WorthQueryInstalledManagedIndex {
            compiled,
            source: Arc::new(source),
            marker: PhantomData,
        })
    }
}

/// The receiving catalog's exact Index/Parent association, without live root authority.
#[derive(Clone)]
pub struct WorthQueryInstalledManagedIndexParent {
    compiled: Arc<CompiledManagedIndex>,
    access: Arc<WorthQueryInstalledManagedIndexAccessContract>,
}
impl WorthQueryInstalledManagedIndexParent {
    pub fn access_contract(&self) -> &Arc<WorthQueryInstalledManagedIndexAccessContract> {
        &self.access
    }
    pub fn binding_identity(
        &self,
    ) -> &worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity
    {
        &self.compiled.schema
    }
    pub fn definition(&self) -> &ErasedApplicationManagedIndexDefinition {
        self.compiled.recipe.meaning()
    }
    pub fn owns_access_contract(
        &self,
        contract: &Arc<WorthQueryInstalledManagedIndexAccessContract>,
    ) -> bool {
        Arc::ptr_eq(&self.access, contract)
    }
    pub fn same_association(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.compiled, &other.compiled) && Arc::ptr_eq(&self.access, &other.access)
    }
}
impl<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>>
    WorthQueryInstalledManagedIndex<S, I>
{
    /// Only an association issued by this actual compiled Index may dispatch its adapter.
    pub fn admits_association(&self, parent: &WorthQueryInstalledManagedIndexParent) -> bool {
        Arc::ptr_eq(&self.compiled, &parent.compiled)
    }
}
impl<S: ApplicationSchema> WorthQueryInstalledApplicationSchema<S> {
    /// Erase marker types only after resolving the actual typed Parent and its selector.
    pub fn installed_managed_index_parent<Q: ApplicationQueryBinding<S>, Stop>(
        &self,
        parent: &WorthQueryInstalledApplicationQueryBinding<S, Q>,
        admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<
        Option<WorthQueryInstalledManagedIndexParent>,
        crate::application_query::WorthQueryInstalledQueryBindingAdmissionStop<Stop>,
    > {
        use crate::application_query::WorthQueryInstalledQueryBindingAdmissionStop as Failure;
        admit(1, 0).map_err(Failure::Admission)?;
        let Some(selection) = parent.query().read_graph().managed_index_selection() else {
            return Ok(None);
        };
        let name = &selection.parts().index;
        let work = self
            .query_catalog
            .managed_index_lookup_work(name)
            .ok_or(Failure::AccountingOverflow)?;
        admit(work, 0).map_err(Failure::Admission)?;
        let compiled = self.query_catalog.get_managed_index(name).ok_or_else(|| {
            Failure::Installation(denied(Kind::InvalidManagedIndexDefinition, name))
        })?;
        let work = compiled
            .parents
            .len()
            .checked_add(1)
            .and_then(|count| count.checked_mul(parent.identity().len().checked_add(1)?))
            .and_then(|work| u64::try_from(work).ok())
            .ok_or(Failure::AccountingOverflow)?;
        admit(work, 0).map_err(Failure::Admission)?;
        // Exact compiled Parent ownership also fixes its selector and full
        // read-family meaning; no per-read recipe reconstruction is needed.
        let actual = compiled
            .parents
            .get(parent.identity())
            .filter(|actual| parent.query().is_compiled_query(&actual.query()))
            .ok_or_else(|| {
                Failure::Installation(denied(Kind::ManagedIndexBindingMeaningChanged, name))
            })?;
        admit(3, 0).map_err(Failure::Admission)?;
        if compiled.schema != self.binding_identity()
            || parent.query().binding_identity() != &compiled.schema
        {
            return Err(Failure::Installation(denied(
                Kind::ManagedIndexBindingMeaningChanged,
                name,
            )));
        }
        let access = Arc::clone(&actual.access);
        Ok(Some(WorthQueryInstalledManagedIndexParent {
            compiled,
            access,
        }))
    }
}
