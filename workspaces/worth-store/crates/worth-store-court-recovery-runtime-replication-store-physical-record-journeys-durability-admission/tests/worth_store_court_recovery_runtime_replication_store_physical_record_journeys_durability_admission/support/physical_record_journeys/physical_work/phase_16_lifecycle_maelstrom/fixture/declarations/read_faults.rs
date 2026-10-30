use super::*;

pub(crate) fn read_fault_rules(
    authority: &CertificationMediaFaultAuthority,
    after_open: u64,
    gates: &MaelstromPauseGates,
) -> [MediaFaultRule; 3] {
    [
        authority
            .rule(
                worth_store_physical_backend::MediaOperationRole::PositionedRead,
                after_open + 1,
                worth_store_physical_backend::MediaFaultDirective::PauseBefore(
                    gates.first_read.clone(),
                ),
            )
            .for_identified_operation_ordinal(),
        authority
            .rule(
                worth_store_physical_backend::MediaOperationRole::PositionedRead,
                after_open + 2,
                worth_store_physical_backend::MediaFaultDirective::PauseBefore(
                    gates.second_read.clone(),
                ),
            )
            .for_identified_operation_ordinal(),
        authority
            .rule(
                worth_store_physical_backend::MediaOperationRole::PositionedRead,
                1,
                worth_store_physical_backend::MediaFaultDirective::PauseBefore(
                    gates.close_read.clone(),
                ),
            )
            .for_next_identified_operation_after_activation(gates.close_read_activation.clone()),
    ]
}
