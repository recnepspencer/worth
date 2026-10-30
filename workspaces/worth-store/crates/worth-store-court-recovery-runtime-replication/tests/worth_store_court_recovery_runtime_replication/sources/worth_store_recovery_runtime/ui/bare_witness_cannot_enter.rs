use worth_proof::AuthorityWitness;
use worth_store_recovery_runtime::PhysicalRecoveryPlatformAuthority;

worth_proof::authority_marker!(pub CallerAuthority);

fn requires_platform(_: PhysicalRecoveryPlatformAuthority) {}

fn main() {
    let witness: AuthorityWitness<CallerAuthority> = CallerAuthority::witness();
    requires_platform(witness);
}
