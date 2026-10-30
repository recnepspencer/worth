use worth_store_budgets::{BlobHarnessEnvelopeDeclaration, BlobHarnessEnvelopeProfile};

fn main() {
    let _ = BlobHarnessEnvelopeDeclaration::new(
        BlobHarnessEnvelopeProfile::CiMemoryEnvelopeExceeding,
        1,
        2,
        3,
    );
}
