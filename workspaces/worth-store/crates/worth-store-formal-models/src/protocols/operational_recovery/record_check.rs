use worth_store_operations::OperationalControlRecord;

use super::{
    map_operational_control_record, OperationalRecoveryCounterexample, OperationalRecoveryModel,
};

pub fn check_operational_recovery_records(
    records: &[OperationalControlRecord],
) -> Result<OperationalRecoveryModel, OperationalRecoveryCounterexample> {
    let mut model = OperationalRecoveryModel::default();
    for record in records {
        model.apply(&map_operational_control_record(record))?;
    }
    Ok(model)
}
