

#[allow(unused_imports)] use crate::phase_4_literal_vectors::*;
#[test]
fn physical_work_literal_is_independently_consumable() {
    crate::phase_4_literal_vectors::physical_work_obligation::verify();
}

#[test]
fn durable_frame_family_literals_are_independently_consumable() {
    crate::phase_4_literal_vectors::durable_frames::verify();
}

#[test]
fn every_declared_page_size_literal_is_independently_consumable() {
    crate::phase_4_literal_vectors::page_frames::verify();
}

#[test]
fn wal_literal_is_independently_consumable() {
    crate::phase_4_literal_vectors::wal_frame::verify();
}

#[test]
fn every_checkpoint_record_kind_is_independently_consumable() {
    crate::phase_4_literal_vectors::checkpoint_records::verify();
}
