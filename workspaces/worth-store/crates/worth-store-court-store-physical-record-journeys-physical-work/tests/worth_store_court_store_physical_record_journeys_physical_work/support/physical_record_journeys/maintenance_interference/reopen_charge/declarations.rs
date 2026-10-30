pub(crate) use {
    worth_store::physical_runtime::PhysicalMutationOutcome,
    worth_store::physical_runtime::PhysicalRetirementDenial,
};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::maintenance_interference::reopen::open_interference};
pub(crate) use {worth_store_testkit_store::physical_record_journeys::maintenance_interference::append, worth_store_testkit_store::physical_record_journeys::maintenance_interference::checkpoint, worth_store_testkit_store::physical_record_journeys::maintenance_interference::initialize, worth_store_testkit_store::physical_record_journeys::maintenance_interference::placement, worth_store_testkit_store::physical_record_journeys::maintenance_interference::segment_files};
pub(crate) use {worth_store_testkit_store::physical_record_journeys::retirement_charge_oracle::assert_one_page_released_net_of_wal, worth_store_testkit_store::physical_record_journeys::retirement_charge_oracle::wal_bytes};

pub(crate) const PAGE_BYTES: u64 = 16 * 1024;

pub(crate) fn charge_on_reopen(root: &std::path::Path) -> u64 {
    let serving = open_interference(root);
    let charged = serving.certification_charged_growth_bytes();
    serving.close();
    charged
}
