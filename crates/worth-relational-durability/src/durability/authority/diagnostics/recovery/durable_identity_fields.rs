use crate::replay::data::ReplayVerificationLayer;
use worth_relational_contracts::internal_port::diagnostics::data::fields::RelationalDiagnosticValue;
use worth_relational_contracts::internal_port::schema::data::relation_integrity::contract_id::ContractId;

pub(super) fn verification_layer_value(
    layer: ReplayVerificationLayer,
) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::string(format!("{layer:?}"))
}

pub(super) fn contract_id_array(contract_ids: &[ContractId]) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::array(
        contract_ids
            .iter()
            .cloned()
            .map(RelationalDiagnosticValue::ContractId),
    )
}
