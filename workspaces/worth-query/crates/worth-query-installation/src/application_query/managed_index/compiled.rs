//! Immutable installed meaning and its actual compiled source/parent owners.
use super::*;
use worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity;

pub(crate) struct CompiledManagedIndex {
    pub(super) schema: ApplicationSchemaBindingIdentity,
    pub(super) recipe: ApplicationManagedIndexBindingDescriptor,
    pub(super) source: Arc<Binding>,
    pub(super) parents: BTreeMap<String, CompiledManagedIndexParent>,
}

pub(super) struct CompiledManagedIndexParent {
    pub(super) binding: Arc<Binding>,
    pub(super) access: Arc<WorthQueryInstalledManagedIndexAccessContract>,
}
impl CompiledManagedIndexParent {
    pub(super) fn query(&self) -> Arc<super::super::binding::WorthQueryCompiledApplicationQuery> {
        self.binding.query()
    }
}

pub(super) struct ValidatedManagedIndex {
    pub(super) recipe: ApplicationManagedIndexBindingDescriptor,
    pub(super) source: Arc<Binding>,
    pub(super) parents: BTreeMap<String, Arc<Binding>>,
}

impl ValidatedManagedIndex {
    pub(super) fn install(
        self,
        schema: ApplicationSchemaBindingIdentity,
    ) -> Result<CompiledManagedIndex, Denial> {
        let mut parents = BTreeMap::new();
        for (name, binding) in self.parents {
            let access = WorthQueryInstalledManagedIndexAccessContract::compile(
                schema.clone(),
                &self.recipe,
                &self.source,
                &binding,
            )?;
            parents.insert(
                name,
                CompiledManagedIndexParent {
                    binding,
                    access: Arc::new(access),
                },
            );
        }
        Ok(CompiledManagedIndex {
            schema,
            recipe: self.recipe,
            source: self.source,
            parents,
        })
    }
}
