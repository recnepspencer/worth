use worth_runtime_bridge::facade::BridgeHistoricalResolvedRecordIdentity;

fn main() {
    let grouped_row_label: &str = "relation:0:4:2";
    let _identity = BridgeHistoricalResolvedRecordIdentity::from_relational_record(grouped_row_label);
}
