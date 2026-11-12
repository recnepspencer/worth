//! Consume the original B preparation only with its actual installed receipt.
use super::super::super::super::bootstrap::native_meaning::PreparedNativeSchemaMeaning;
use super::*;
use worth_foundational::facade::HomeProgramTransitionInstallationBinding;
use worth_proof::AdmittedHomeProgramTransitionSuccessor;

pub struct QueryHomeTransitionPromotionRefused<Schema: ApplicationSchemaComposition, Program> {
    _target: PreparedHomeProgramTransitionTarget<Schema, Program>,
    _receipt: RelationalInitialSchemaInstallationReceipt,
    _installation: AdmittedHomeProgramTransitionSuccessor<HomeProgramTransitionInstallationBinding>,
    cause: QueryHomeTransitionPreparationDenial,
}
impl<Schema: ApplicationSchemaComposition, Program>
    QueryHomeTransitionPromotionRefused<Schema, Program>
{
    pub fn reason(&self) -> &QueryHomeTransitionPreparationDenial {
        &self.cause
    }
}
impl<Schema: ApplicationSchemaComposition, Program>
    PreparedHomeProgramTransitionTarget<Schema, Program>
{
    pub fn finish_home_program_transition(
        self,
        native: &RelationalRuntime,
        receipt: RelationalInitialSchemaInstallationReceipt,
        installation: AdmittedHomeProgramTransitionSuccessor<
            HomeProgramTransitionInstallationBinding,
        >,
    ) -> Result<
        ReadmittedExistingHomeNativeMeaning<Schema, Program>,
        QueryHomeTransitionPromotionRefused<Schema, Program>,
    > {
        let checked = (|| {
            native
                .admit_installed_home_program_target(&self.contracts, &receipt, &installation)
                .map_err(QueryHomeTransitionPreparationDenial::Native)?;
            let installed = self._pending._state.installed.binding_identity();
            let meaning = installation.binding();
            if meaning.successor()
                != [
                    *installed.package_identity(),
                    *installed.schema_identity(),
                    worth_foundational::facade::CanonicalDigestId::new(
                        *self._pending._state.support.initial_revision.as_bytes(),
                    ),
                ]
                || receipt.runtime_instance_id() != native.runtime_instance_id()
                || meaning.target().catalog() != self.contracts.catalog_digest()
                || meaning.target().schema().bytes() != receipt.schema_authority_digest()
            {
                return Err(QueryHomeTransitionPreparationDenial::Native(
                    RelationalHomeProgramContractDenial::TargetCatalogMismatch,
                ));
            }
            self._pending
                ._inventory
                .require_receipt(&receipt)
                .map_err(QueryHomeTransitionPreparationDenial::Compiler)
        })();
        if let Err(cause) = checked {
            return Err(QueryHomeTransitionPromotionRefused {
                _target: self,
                _receipt: receipt,
                _installation: installation,
                cause,
            });
        }
        let HomeTransitionTargetCustody {
            mut _state,
            _layout,
            _managed_sources,
            _source_initializer,
            _inventory,
        } = self._pending;
        _state.transition_installation = Some(installation);
        let schema = PreparedNativeSchemaMeaning {
            managed_sources: _managed_sources,
            source_initializer: _source_initializer,
            layout: _layout,
            receipt,
            expected_invariant_inventory_digest: _inventory.digest(),
        }
        .bind_sources(&native.main_branch_identity());
        Ok(ReadmittedExistingHomeNativeMeaning::from_transition(
            _state, schema,
        ))
    }
}
