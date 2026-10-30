use worth_runtime_world::facade::{
    CompositePublicationIntent, RelationalTransactionIntent, WithSignal, WithoutSignal,
};

fn requires_the_signal_stage(_: CompositePublicationIntent<WithSignal>) {}

fn illegal_signal_publication_from_a_retained_signal_intent() {
    let intent = CompositePublicationIntent::<WithoutSignal>::without_signal(
        RelationalTransactionIntent::ordinary(),
    );
    requires_the_signal_stage(intent);
}

fn main() {}
