use worth_store_physical_certification::admit_physical_isolation_entry;

struct CopiedS4RecoveryFields {
    recovered_root: String,
    replayed_frames: usize,
}

fn main() {
    let copied = CopiedS4RecoveryFields {
        recovered_root: String::from("root"),
        replayed_frames: 1,
    };
    let _ = admit_physical_isolation_entry(copied);
}
