#[allow(unused_imports)]
use crate::physical_record_journeys::maintenance_interference::page_variant::*;
use {
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::PhysicalPageSizeClass,
    worth_store::physical_runtime::PhysicalRecordFormatDeclaration,
};

use {
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::append,
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::initialize_with_format,
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::limits,
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::placement_for,
};

#[test]
fn thirty_two_kib_pages_publish_the_admitted_record() {
    publish(worth_store::physical_runtime::PhysicalPageSizeClass::KiB32);
}

#[test]
fn sixty_four_kib_pages_publish_the_admitted_record() {
    publish(worth_store::physical_runtime::PhysicalPageSizeClass::KiB64);
}
