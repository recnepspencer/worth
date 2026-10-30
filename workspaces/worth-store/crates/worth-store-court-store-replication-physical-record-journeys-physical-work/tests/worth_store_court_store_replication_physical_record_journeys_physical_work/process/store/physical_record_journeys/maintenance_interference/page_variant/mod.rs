#[allow(unused_imports)]
use crate::physical_record_journeys::maintenance_interference::page_variant::*;

#[test]
fn thirty_two_kib_pages_publish_the_admitted_record() {
    publish(worth_store::physical_runtime::PhysicalPageSizeClass::KiB32);
}

#[test]
fn sixty_four_kib_pages_publish_the_admitted_record() {
    publish(worth_store::physical_runtime::PhysicalPageSizeClass::KiB64);
}
