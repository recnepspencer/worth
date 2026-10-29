use worth_foundational::{AuthoritativeRecordAspectPatch, PortableRecordAspectPatch};

fn require_authoritative(_: AuthoritativeRecordAspectPatch) {}

fn main() {
    require_authoritative(PortableRecordAspectPatch::new([]));
}
