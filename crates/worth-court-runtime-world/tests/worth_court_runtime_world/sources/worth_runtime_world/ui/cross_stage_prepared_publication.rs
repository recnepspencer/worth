use worth_runtime_world::facade::{
    PreparedCompositePublicationWithSignal, PreparedCompositePublicationWithoutSignal,
};

fn consume_with_signal(_: PreparedCompositePublicationWithSignal) {}

fn illegal_stage_substitution(prepared: PreparedCompositePublicationWithoutSignal) {
    consume_with_signal(prepared);
}

fn illegal_stage_conversion(
    prepared: PreparedCompositePublicationWithSignal,
) -> PreparedCompositePublicationWithoutSignal {
    PreparedCompositePublicationWithoutSignal::from(prepared)
}

fn main() {}
