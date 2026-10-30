pub use {
    worth_store::physical_runtime::PhysicalMutationOutcome,
    worth_store::physical_runtime::PhysicalRetirementDenial,
};

pub use crate::physical_record_journeys::maintenance_interference::reopen::open_interference;
pub use {
    crate::physical_record_journeys::maintenance_interference::append,
    crate::physical_record_journeys::maintenance_interference::checkpoint,
    crate::physical_record_journeys::maintenance_interference::initialize,
    crate::physical_record_journeys::maintenance_interference::placement,
    crate::physical_record_journeys::maintenance_interference::segment_files,
};
pub use {
    crate::physical_record_journeys::retirement_charge_oracle::assert_one_page_released_net_of_wal,
    crate::physical_record_journeys::retirement_charge_oracle::wal_bytes,
};

pub const PAGE_BYTES: u64 = 16 * 1024;

pub fn charge_on_reopen(root: &std::path::Path) -> u64 {
    let serving = open_interference(root);
    let charged = serving.certification_charged_growth_bytes();
    serving.close();
    charged
}
