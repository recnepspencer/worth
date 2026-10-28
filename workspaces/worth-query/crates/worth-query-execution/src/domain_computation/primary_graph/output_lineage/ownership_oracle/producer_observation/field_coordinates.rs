//! Genuine performed inputs and query revisions meet only through installed fields.
use super::*;
use primary_graph::{
    admitted_capture::checkpoint,
    application_checkpoint::facts::FactPayload,
    schema_layout::{InstalledFieldCoordinate, WorthQueryPrimaryGraphLayout},
    source_facts::WorthQueryObservedFieldRevision,
    WorthQueryApplicationObservedFact as Fact, WorthQueryObservedSource,
};
use worth_execution::MapKernelContext;
use worth_foundational::facade::{AspectFieldLocator, LocatorAuthority};
use worth_query_installation::facade::WorthQueryInstalledApplicationSchemaContractCatalog;
use worth_relational::facade::identity::VersionId;

pub(super) fn inspect<Query>(
    context: &mut MapKernelContext<'_, '_>,
    runtime: &Application,
    original: &WorthQueryObservedSource<Query>,
) {
    let supplier = original.original_settled_producer().unwrap();
    let OriginalSettledProducerSource::Committed {
        changes,
        inputs: Some(inputs),
        ..
    } = supplier.source()
    else {
        panic!("the actual committed producer retained its performed inputs")
    };
    let captured = CapturedFields {
        original,
        inputs,
        layout: &runtime.primary_provider.graph.layout,
        contracts: runtime.installed_schema().native_contracts(),
        producer_version: changes.commit_reference().version_id,
    };
    captured.inspect_handlers(context);
    captured.inspect_source_revisions(context);
}

/// Borrow the actual published owners; never author a replacement fact or revision.
struct CapturedFields<'a, Query> {
    original: &'a WorthQueryObservedSource<Query>,
    inputs: &'a FactPayload,
    layout: &'a WorthQueryPrimaryGraphLayout,
    contracts: &'a WorthQueryInstalledApplicationSchemaContractCatalog,
    producer_version: VersionId,
}
impl<Query> CapturedFields<'_, Query> {
    fn inspect_handlers(&self, context: &mut MapKernelContext<'_, '_>) {
        let mut count = 0;
        for fact in self.inputs.iter() {
            let Fact::Field {
                entity_id,
                kind,
                locator,
                ..
            } = fact
            else {
                continue;
            };
            let owner = self
                .layout
                .entity_name(*kind)
                .expect("actual handler kind is installed");
            eprintln!("performed handler entity={entity_id:?} kind={kind:?} locator={locator:?}; producer={:?}", self.producer_version);
            assert_eq!(locator.aspect().authority(), LocatorAuthority::Planned);
            let resolved = self
                .layout
                .resolve_field_coordinate(self.contracts, owner, locator, |work| {
                    checkpoint(context, work)
                })
                .unwrap()
                .expect("a genuine completed handler field resolves");
            assert_eq!(resolved.entity_kind(), *kind);
            count += 1;
        }
        assert!(
            count > 0,
            "the actual producer must retain completed handler reads"
        );
    }

    fn inspect_source_revisions(&self, context: &mut MapKernelContext<'_, '_>) {
        let mut joined = 0;
        let mut different_fields = 0;
        for fact in self.inputs.iter() {
            let Fact::SourceFieldRevision {
                entity_id,
                locator,
                native_revision: Some(revision),
            } = fact
            else {
                continue;
            };
            eprintln!("performed source entity={entity_id:?} locator={locator:?} revision={revision:?}; producer={:?}", self.producer_version);
            assert_eq!(
                locator.aspect().authority(),
                LocatorAuthority::Authoritative
            );
            assert_ne!(
                revision.version(),
                self.producer_version,
                "original input revisions precede the producer effect"
            );
            let [key] = locator.field_path().fields() else {
                panic!("fixture's actual source field")
            };
            for row in self.original.original_row_footprints() {
                for observed in &row.aspects {
                    if observed.entity != *entity_id
                        || observed.aspect != *locator.aspect().aspect_key()
                        || observed.field != *key
                    {
                        continue;
                    }
                    eprintln!(
                        "query full entity={:?} revision={:?} original={revision:?}",
                        observed.entity, observed.native_revision
                    );
                    let source = self.inspect_binding(context, locator, observed);
                    different_fields +=
                        self.refuse_different_handler_fields(context, locator, observed, source);
                    if observed.native_revision == Some(*revision) {
                        joined += 1;
                    }
                }
            }
        }
        assert!(
            joined > 0,
            "at least one genuine original field revision reaches this query unchanged"
        );
        assert!(
            different_fields > 0,
            "a genuine different-field handler read exercises refusal"
        );
    }

    fn inspect_binding<'a>(
        &'a self,
        context: &mut MapKernelContext<'_, '_>,
        original: &AspectFieldLocator,
        observed: &WorthQueryObservedFieldRevision,
    ) -> InstalledFieldCoordinate<'a> {
        let source = self
            .layout
            .resolve_field_coordinate(self.contracts, &observed.entity_name, original, |work| {
                checkpoint(context, work)
            })
            .unwrap()
            .unwrap();
        let planned = self
            .layout
            .field_locator(
                &observed.entity_name,
                observed.aspect.as_str(),
                observed.field.as_str(),
            )
            .unwrap();
        assert_ne!(
            planned, original,
            "the original authority distinction remains visible"
        );
        let installed = self
            .layout
            .resolve_field_coordinate(self.contracts, &observed.entity_name, planned, |work| {
                checkpoint(context, work)
            })
            .unwrap()
            .unwrap();
        assert!(source.same_field(&installed));
        assert_eq!(source.contract_revision(), observed.contract_revision);
        self.refuse_other_authorities(context, &observed.entity_name, original);
        source
    }

    fn refuse_different_handler_fields(
        &self,
        context: &mut MapKernelContext<'_, '_>,
        original: &AspectFieldLocator,
        observed: &WorthQueryObservedFieldRevision,
        source: InstalledFieldCoordinate<'_>,
    ) -> usize {
        let mut different = 0;
        for handler in self.inputs.iter() {
            let Fact::Field {
                entity_id,
                kind,
                locator,
                ..
            } = handler
            else {
                continue;
            };
            if *entity_id != observed.entity {
                continue;
            }
            let field = self
                .layout
                .resolve_field_coordinate(self.contracts, &observed.entity_name, locator, |work| {
                    checkpoint(context, work)
                })
                .unwrap()
                .unwrap();
            assert_eq!(field.entity_kind(), *kind);
            if locator.field_path() != original.field_path()
                || locator.aspect().aspect_key() != original.aspect().aspect_key()
            {
                assert!(
                    !source.same_field(&field),
                    "a real read of another field is insufficient"
                );
                different += 1;
            }
        }
        different
    }

    fn refuse_other_authorities(
        &self,
        context: &mut MapKernelContext<'_, '_>,
        owner: &str,
        original: &AspectFieldLocator,
    ) {
        for authority in [
            LocatorAuthority::Derived,
            LocatorAuthority::Projected,
            LocatorAuthority::SupportOnly,
            LocatorAuthority::ReceiptBearing,
        ] {
            // Hostile descriptors grant nothing, even with the genuine source's keys.
            let hostile = AspectFieldLocator::new(
                authority,
                original.aspect().aspect_key().clone(),
                original.field_path().clone(),
            );
            assert!(self
                .layout
                .resolve_field_coordinate(self.contracts, owner, &hostile, |work| checkpoint(
                    context, work
                ),)
                .unwrap()
                .is_none());
        }
    }
}
