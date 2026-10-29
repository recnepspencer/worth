worth_proof::authority_marker!(ForgedSourceAuthority);

fn main() {
    let _ = worth_proof::AdmittedConditionalSourceObservation::admit(
        "forged-source",
        ForgedSourceAuthority::witness(),
    );
}
