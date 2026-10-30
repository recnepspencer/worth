use worth_foundational::{
    FoundationalBoundaryEvidenceCategory, FoundationalBoundaryEvidenceLocality,
};

fn needs_category(_category: FoundationalBoundaryEvidenceCategory) {}

fn main() {
    let locality = FoundationalBoundaryEvidenceLocality::Current;
    needs_category(locality);
}
