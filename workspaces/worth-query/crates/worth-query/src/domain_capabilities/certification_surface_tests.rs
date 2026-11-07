use crate::domain_capabilities::certification::{
    worth_query_domain_capability_certification_surface,
    worth_query_domain_capability_public_surface_inventory,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

#[test]
fn certification_surface_keeps_ordinary_lane_distinct_from_lower_lanes(
) -> Result<(), IdentityDerivationDenial> {
    let surface = worth_query_domain_capability_certification_surface()?;
    let inventory = worth_query_domain_capability_public_surface_inventory();

    assert_eq!(
        surface.public_surface_digest(),
        inventory
            .public_surface_digest()
            .expect("fixture digest must fit")
    );
    assert_eq!(surface.category_count(), inventory.rows().len());
    for row in inventory.rows() {
        assert_ne!(row.ordinary_lane(), row.inspectable_lane());
        assert_ne!(row.inspectable_lane(), row.proof_lane());
        assert_ne!(row.proof_lane(), row.raw_lane());
        assert!(!row.ordinary_lane().contains("raw"));
        assert!(!row.ordinary_lane().contains("proof"));
    }

    Ok(())
}
