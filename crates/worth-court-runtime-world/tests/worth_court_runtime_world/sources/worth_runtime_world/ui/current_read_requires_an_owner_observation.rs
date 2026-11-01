use worth_runtime_world::facade::{
    CompositePublicationIntent, ProductBranchReferenceDescription, WithoutSignal,
};

fn deny(
    intent: &mut CompositePublicationIntent<WithoutSignal>,
    description: ProductBranchReferenceDescription,
) {
    intent.require_current_read(description);
}

fn main() {}
