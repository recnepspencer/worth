//! The source and query targets share current admission, never original authority.
use super::super::{source_original::SourceOriginal, WorthQueryApplicationInspectionTarget};
use super::*;
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationInspectionContract,
    WorthQueryInstalledApplicationQueryBinding,
};

#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) enum InspectionSubject<
    'a,
> {
    Query(&'a super::super::super::application_observation::WorthQueryApplicationQueryObservation),
    Source(&'a SourceOriginal),
}
impl InspectionSubject<'_> {
    pub(super) fn validate<Schema: ApplicationSchema, Binding: ApplicationQueryBinding<Schema>>(
        self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        installed: &WorthQueryInstalledApplicationQueryBinding<Schema, Binding>,
        selected: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
            '_,
            Schema,
        >,
        scope: worth_relational::facade::identity::EntityId,
    ) -> Result<WorthQueryInstalledApplicationInspectionContract, InspectionAdmissionDenial> {
        let contract = installed
            .query()
            .binding_observation()
            .and_then(|binding| binding.inspection_contract())
            .ok_or(InspectionAdmissionDenial::MissingInspection)?;
        let valid = match self {
            Self::Query(original) => {
                validate_original_affinity(runtime, installed, original)?;
                let same = |selection: &crate::domain_computation::primary_graph::WorthQueryApplicationBasisSelectionIdentity| matches!(selection,
                    crate::domain_computation::primary_graph::WorthQueryApplicationBasisSelectionIdentity::Product(original) if original.same_branch_occurrence_observation(selected.product().observation()));
                same(&original.source.selection)
                    && same(original.basis.selection())
                    && original.source.retained_scope_selector().subject() == scope
            }
            Self::Source(original) => {
                original.runtime == runtime.runtime.authority_identity().as_u64()
                    && original.disclosure.schema()
                        == contract.graph_obligations().binding_identity()
                    && original
                        .disclosure
                        .association()
                        .declaration()
                        .parts()
                        .query
                        == contract.definition().parts().binding
                    && original.disclosure.scope().subject() == scope
                    && original
                        .publication
                        .same_branch_occurrence_observation(selected.product().observation())
            }
        };
        if !std::ptr::eq(runtime, selected.application()) || !valid {
            return Err(InspectionAdmissionDenial::ForeignIssuedAccess);
        }
        Ok(contract)
    }
    pub(super) fn capture(
        self,
        execution: &ExecutionRequest<'_, '_>,
        before: Option<
            &super::super::super::application_observation::WorthQueryApplicationQueryObservation,
        >,
        contract: WorthQueryInstalledApplicationInspectionContract,
        memory: &mut Option<ExecutionMemoryReservation>,
    ) -> Result<Arc<WorthQueryApplicationInspectionTarget>, InspectionAdmissionDenial> {
        match self {
            Self::Query(original) => WorthQueryApplicationInspectionTarget::capture(
                execution, original, before, contract, memory,
            ),
            Self::Source(original) if before.is_none() => {
                WorthQueryApplicationInspectionTarget::capture_source(
                    execution, original, contract, memory,
                )
            }
            _ => return Err(InspectionAdmissionDenial::ForeignIssuedAccess),
        }
        .map_err(InspectionAdmissionDenial::Target)
    }
}
