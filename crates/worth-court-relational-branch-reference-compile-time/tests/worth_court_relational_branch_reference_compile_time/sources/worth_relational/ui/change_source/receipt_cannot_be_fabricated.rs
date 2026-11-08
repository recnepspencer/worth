use worth_relational::facade::change_source::RelationalChangeReceipt;

fn reconstruct(value: RelationalChangeReceipt) -> RelationalChangeReceipt {
    RelationalChangeReceipt { ..value }
}

fn main() {
    let _ = reconstruct;
}
