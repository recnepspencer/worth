pub(super) mod driver;
mod observation;
mod publication;
use super::oracle::product::ProductModel;
use super::*;
use std::collections::BTreeMap;

#[test]
fn court_seeded_product_model_checks_each_transition_prefix() {
    for seed in [0x9172, 0x51a7, 0xcafe] {
        driver::run(seed, 6);
    }
}
