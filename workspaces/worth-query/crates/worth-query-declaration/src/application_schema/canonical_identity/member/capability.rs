use worth_foundational::facade::CanonicalBasisEntryKind;

use crate::application_schema::canonical_basis::ApplicationSchemaCanonicalBasis;
use crate::application_schema::canonical_capability_identity::append_capability_contract;
use crate::application_schema::ApplicationSchemaMember;

pub(super) fn append_capability_member(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    member: &ApplicationSchemaMember,
) {
    match member {
        ApplicationSchemaMember::ApplicationQueryInspection { definition } => {
            basis.text(format!("{prefix}.kind"), "application-query-inspection");
            basis.extend_embedded(
                definition.canonical_basis().basis(),
                &format!("{prefix}.inspection-meaning"),
                CanonicalBasisEntryKind::Identity,
            );
        }
        ApplicationSchemaMember::ApplicationHistoryQuery { definition } => {
            basis.text(format!("{prefix}.kind"), "application-history-query");
            basis.extend_embedded(
                definition.canonical_basis().basis(),
                &format!("{prefix}.history-meaning"),
                CanonicalBasisEntryKind::Identity,
            );
        }
        ApplicationSchemaMember::ApplicationQuery { definition } => {
            basis.text(format!("{prefix}.kind"), "application-query");
            basis.extend_embedded(
                definition.canonical_basis().basis(),
                &format!("{prefix}.query-meaning"),
                CanonicalBasisEntryKind::Identity,
            );
        }
        ApplicationSchemaMember::ApplicationCapability { contract } => {
            basis.text(format!("{prefix}.kind"), "application-capability");
            append_capability_contract(basis, &format!("{prefix}.contract"), contract);
        }
        ApplicationSchemaMember::ApplicationCapabilityContext {
            context,
            context_type,
        } => {
            basis.text(format!("{prefix}.kind"), "application-capability-context");
            basis.text(format!("{prefix}.context"), context);
            basis.text(format!("{prefix}.context-type"), context_type.as_str());
        }
        ApplicationSchemaMember::ApplicationCapabilityContextEntitySlot {
            context,
            context_type,
            slot,
            slot_type,
            entity,
        } => {
            basis.text(
                format!("{prefix}.kind"),
                "application-capability-context-entity-slot",
            );
            basis.text(format!("{prefix}.context"), context);
            basis.text(format!("{prefix}.context-type"), context_type.as_str());
            basis.text(format!("{prefix}.slot"), slot);
            basis.text(format!("{prefix}.slot-type"), slot_type.as_str());
            basis.text(format!("{prefix}.entity"), entity);
        }
        ApplicationSchemaMember::ApplicationCapabilityProvenance {
            provenance,
            provenance_type,
        } => {
            basis.text(
                format!("{prefix}.kind"),
                "application-capability-provenance",
            );
            basis.text(format!("{prefix}.provenance"), provenance);
            basis.text(
                format!("{prefix}.provenance-type"),
                provenance_type.as_str(),
            );
        }
        _ => unreachable!("capability member router supplied another member family"),
    }
}
