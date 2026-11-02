//! Whether an installed operation still matches one re-presented candidate.

use worth_query_declaration::facade::application_schema::ApplicationSchemaMember;

use crate::application_operation::compile_portable_operation_contract_record;
use crate::installed_index::WorthQueryInstalledPackageAuthority;

use super::super::installed_contract_support::authority_transcript;
use super::operation_compilation::WorthQueryApplicationOperationCompilation;
use super::WorthQueryInstalledApplicationOperation;

impl<Schema, Operation, Input> WorthQueryInstalledApplicationOperation<Schema, Operation, Input> {
    pub(crate) fn meaning_matches(&self, members: &[ApplicationSchemaMember]) -> bool {
        self.recompiled_contracts_match(members)
    }

    fn recompiled_contracts_match(&self, members: &[ApplicationSchemaMember]) -> bool {
        let Ok(portable_contract) = compile_portable_operation_contract_record(
            &self.core.schema_name,
            members,
            self.core.portable_native_contracts.as_ref(),
            &self.core.operation,
            self.core.portable_contract.input_type().clone(),
        ) else {
            return false;
        };
        let Ok(compilation) = WorthQueryApplicationOperationCompilation::resolve(
            self.core.binding_identity.clone(),
            members,
            &portable_contract,
            self.core.native_contracts.as_ref(),
            &self.core.operation,
            &self.core.input_type,
        ) else {
            return false;
        };
        let Ok(candidate_contracts) = compilation.compile_contracts(
            self.core.contracts.ability_requirements().to_vec(),
            self.core.native_contracts.as_ref(),
        ) else {
            return false;
        };
        candidate_contracts.eq(self.core.contracts.as_ref())
    }

    pub(crate) fn authority_matches(&self, package: &WorthQueryInstalledPackageAuthority) -> bool {
        authority_transcript(
            &package.authority_key,
            &self.core.binding_identity,
            &self.core.operation,
            &self.core.input_type,
            self.core.obligations.identity(),
        )
        .verifies(&self.core.authority_identity)
    }
}
