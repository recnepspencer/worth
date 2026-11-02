//! Narrow owner injection preserving the genuine effect program and typed brand.
use super::*;

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation::primary_graph) fn forge_installed_seal_for_test(
        &mut self,
        donor: worth_query_installation::facade::InstalledOperationView<'_>,
    ) {
        self.read_set.admission.forge_installed_seal_for_test(donor);
    }
}
