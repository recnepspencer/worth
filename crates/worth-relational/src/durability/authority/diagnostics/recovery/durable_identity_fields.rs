use crate::diagnostics::data::RelationalDiagnosticValue;
use crate::replay::data::ReplayVerificationLayer;
use crate::schema::data::ContractId;

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
