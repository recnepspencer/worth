//! Governed hidden-ordering capability, query, and program definitions for
//! `retained_basis_cursor_disclosure_court`.

use worth_query_decl::facade::application_capability::*;
use worth_query_decl::facade::application_query::*;
use worth_query_decl::facade::application_schema::fragment::Registration;
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::*;
use worth_query_host::facade::{primary_graph::*, worth_query_application_program};

use super::retained_basis_cursor_disclosure_schema::*;

worth_query_capability!(pub ReadVault in HiddenSchema, identity "hidden-order.cap.v1");
worth_query_capability_context!(pub VaultCtx in HiddenSchema);
worth_query_capability_provenance!(pub VaultProv in HiddenSchema);
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct ReadVaultAccess {
    pub vault: String,
}
worth_query_structured_value_binding!(pub ReadVaultInput for ReadVaultAccess { identity: "hidden-order.access.v1" });
worth_query_operation!(pub InspectVault for HiddenSchema, input ReadVaultInput);
worth_query_operation_reads!(InspectVault => [VaultKey, GDisc]);

fn tag(v: u64) -> ApplicationEncodedScalarValue<U64ApplicationValueBinding> {
    ApplicationEncodedScalarValue::try_new(v).unwrap()
}

impl ApplicationCapabilityRequest<HiddenSchema, ReadVault> for ReadVaultAccess {
    type Scope = Vault;
    type Context = VaultCtx;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<HiddenSchema, Vault, VaultCtx>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                VaultKey::at("VaultKey"),
                ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                    self.vault.clone(),
                )
                .unwrap(),
            )
            .unwrap(),
            tag(1),
            tag(1),
            ApplicationCapabilityRequestContext::new(VaultCtx::reference()),
        )
        .field(tag(1)))
    }
}

pub struct HiddenAccessReg;
impl Registration<HiddenSchema> for HiddenAccessReg {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<HiddenSchema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<HiddenSchema>, ApplicationSchemaDeclarationDenial>
    {
        let path = ApplicationAuthorizationPathBuilder::from_principal(Prin::reference())
            .forward(GrGrantee::reference())
            .forward(GrResource::reference())
            .allow(Vault::reference());
        let cap = ApplicationCapabilityContractBuilder::new(
            ReadVault::reference(),
            InspectVault::reference(),
            VaultGrant::reference(),
        )
        .target(ApplicationCapabilityTargetDefinition::new(
            ApplicationCapabilityValueBinding::new(GAct::at("GAct"), tag(1)).unwrap(),
            ApplicationCapabilityRelationBinding::from_reference(GrResource::reference()),
            ApplicationCapabilityRelationDimension::not_applicable(),
            ApplicationCapabilityFieldDimension::bound(GDisc::at("GDisc")).unwrap(),
            ApplicationCapabilityValueBinding::new(GPurp::at("GPurp"), tag(1)).unwrap(),
        ))
        .constraints(ApplicationCapabilityConstraintDefinition::new(
            ApplicationCapabilityMagnitudeDimension::not_applicable(),
            ApplicationCapabilityCardinalityDimension::One,
            ApplicationCapabilityCurrentnessDefinition::new(
                ApplicationCapabilityValueBinding::new(GStat::at("GStat"), tag(1)).unwrap(),
                ApplicationCapabilityWorkflowDefinition::new(
                    ApplicationCapabilityFieldBinding::from_reference(GScope::at("GScope"))
                        .unwrap(),
                    ApplicationCapabilityFieldBinding::from_reference(VaultKey::at("VaultKey"))
                        .unwrap(),
                ),
                ApplicationCapabilityValidityDefinition::new(
                    ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                    ApplicationCapabilityFieldBinding::from_reference(GNb::at("GNb")).unwrap(),
                    ApplicationCapabilityFieldBinding::from_reference(GNa::at("GNa")).unwrap(),
                ),
            ),
            VaultCtx::reference(),
        ))
        .delegation(ApplicationCapabilityDelegationDefinition::new(
            ApplicationCapabilityRelationBinding::from_reference(GrParent::reference()),
            ApplicationCapabilityRelationBinding::from_reference(GrGrantor::reference()),
            ApplicationCapabilityRelationBinding::from_reference(GrGrantee::reference()),
            ApplicationCapabilityFieldBinding::from_reference(GDepth::at("GDepth")).unwrap(),
            VaultProv::reference(),
        ))
        .composition(ApplicationCapabilityComposition::new(
            ApplicationCapabilityDecisionComposition::new(
                ApplicationCapabilityAllowRule::new(ApplicationCapabilityGraphRule::any([
                    ApplicationCapabilityGraphClause::new(path),
                ])),
                ApplicationCapabilityDenyRule::not_applicable(),
                ApplicationCapabilityConflictRule::not_applicable(),
            ),
            ApplicationCapabilityActorComposition::new(
                ApplicationCapabilitySeparationOfDutyRule::not_applicable(),
                ApplicationCapabilityDistinctActorRule::not_applicable(),
                ApplicationCapabilityRecordedActorRule::not_applicable(),
            ),
            ApplicationCapabilityPropagationComposition::new(
                ApplicationCapabilityDelegationRule::forbidden(),
                ApplicationCapabilityDisclosureRule::permit([
                    ApplicationCapabilityScopeGuard::requiring([
                        ApplicationCapabilityAcceptedValues::one_of(GDisc::at("GDisc"), [tag(1)])
                            .unwrap(),
                    ]),
                ]),
            ),
        ))
        .elevation(ApplicationCapabilityElevationRule::not_applicable())
        .build();
        Ok(builder
            .capability_context(VaultCtx::reference())
            .capability_provenance(VaultProv::reference())
            .operation(
                InspectVault::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(InspectVault::reference(), 32)
            .operation_read_field(InspectVault::reference(), VaultKey::at("VaultKey"))
            .capability(cap))
    }
}

pub struct EmptyParams;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RankedItemRow {
    pub item_key: u64,
    pub label: String,
    pub secret_rank: WorthQueryApplicationDisclosed<String>,
}

impl KnownProjectedHeapBytes for RankedItemRow {
    fn known_heap_bytes(&self) -> usize {
        self.item_key
            .known_heap_bytes()
            .saturating_add(self.label.known_heap_bytes())
            .saturating_add(self.secret_rank.known_heap_bytes())
    }
}
worth_query_structured_value_binding!(pub EmptyParamsB for EmptyParams { identity: "hidden-order.empty.v1" });
worth_query_structured_value_binding!(pub RankedRowB for RankedItemRow { identity: "hidden-order.row.v1" });
pub struct KeySlot;
pub struct LabelSlot;
pub struct RankSlot;
worth_query_portable_type!(KeySlot => "hidden-order.key-slot.v1");
worth_query_portable_type!(LabelSlot => "hidden-order.label-slot.v1");
worth_query_portable_type!(RankSlot => "hidden-order.rank-slot.v1");
worth_query_application_query!(
    pub RankedItemsQuery for HiddenSchema,
    identity "hidden-order.ranked-items.v1",
    parameters EmptyParamsB,
    result RankedRowB,
    scope Vault => "Vault",
    name "ranked_items"
);

fn item_key_field(
) -> ApplicationQueryResultFieldRef<RankedItemsQuery, KeySlot, HiddenSchema, ItemKey> {
    ApplicationQueryResultFieldRef::new("item_key", ItemKey::at("ItemKey"))
}
fn label_field() -> ApplicationQueryResultFieldRef<RankedItemsQuery, LabelSlot, HiddenSchema, Label>
{
    ApplicationQueryResultFieldRef::new("label", Label::at("Label"))
}
fn secret_rank_field(
) -> ApplicationQueryResultFieldRef<RankedItemsQuery, RankSlot, HiddenSchema, SecretRank> {
    ApplicationQueryResultFieldRef::new("secret_rank", SecretRank::at("SecretRank"))
}

impl WorthQueryApplicationProjection<HiddenSchema, RankedItemsQuery> for RankedItemRow {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, HiddenSchema, RankedItemsQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            item_key: row.field(item_key_field())?,
            label: row.field(label_field())?,
            secret_rank: row.disclosed_field(secret_rank_field())?,
        })
    }
}

#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct RankedItemsIntent {
    pub vault: String,
}
worth_query_structured_value_binding!(pub RankedItemsIn for RankedItemsIntent { identity: "hidden-order.intent.v1" });
fn ranked_scope(i: RankedItemsIntent) -> String {
    i.vault
}
worth_query_query_binding!(
    pub RankedItemsBinding for RankedItemsIntent,
    schema HiddenSchema,
    identity "hidden-order.binding.v1",
    input RankedItemsIn,
    query RankedItemsQuery,
    parameters EmptyParamsB => |_: &RankedItemsIntent| ApplicationQueryParameterSet::new(),
    result RankedRowB,
    principal HiddenPrincipal,
    mapping ExtMap,
    principal_entity Prin,
    principal_identity u64,
    identity_binding U64ApplicationValueBinding,
    scope VaultKey,
    field VaultKey::at("VaultKey"),
    value ranked_scope,
    limits results 16
);

pub struct HiddenQueryReg;
impl Registration<HiddenSchema> for HiddenQueryReg {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<HiddenSchema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<HiddenSchema>, ApplicationSchemaDeclarationDenial>
    {
        let shape = ApplicationQueryResultShapeBuilder::<
            HiddenSchema,
            RankedItemsQuery,
            Item,
            RankedItemRow,
            RankedRowB,
        >::new(Item::reference())
        .field(item_key_field())
        .field(label_field())
        .field(secret_rank_field())
        .build();
        let disclosure = ApplicationQueryDisclosureContract::governed_by(
            HiddenRankDisclosure,
            ReadVault::reference(),
        )
        .use_field_by(
            SecretRank::at("SecretRank"),
            tag(1),
            ApplicationQueryInfluenceContract::permit([
                ApplicationQueryObservableInfluence::Ordering,
            ]),
        )
        .unwrap()
        .disclose_field_by(
            item_key_field(),
            tag(1),
            ApplicationQueryInfluenceContract::permit_all(),
        )
        .disclose_field_by(
            label_field(),
            tag(1),
            ApplicationQueryInfluenceContract::permit_all(),
        )
        .disclose_field_by(
            secret_rank_field(),
            tag(2),
            ApplicationQueryInfluenceContract::forbid_all(),
        );
        let def = ApplicationQueryDefinitionBuilder::declare(RankedItemsQuery::reference())
            .root(Item::reference())
            .scope(Vault::reference())
            .result_shape(shape)
            .cardinality(ApplicationQueryCardinality::Many)
            .disclosure(disclosure)
            .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
            .lanes(ApplicationQueryLaneEligibility::one_shot())
            .public()
            .root_path(
                ApplicationQueryRootPath::from(Vault::reference()).forward(Holds::reference()),
            )
            .order_by(
                secret_rank_field(),
                ApplicationQueryOrderingDirection::Ascending,
            );
        Ok(builder
            .application_query(def)
            .application_query_binding::<RankedItemsBinding>())
    }
}

worth_query_application_program! {
    pub program HiddenOrderProgram for HiddenSchema {
        identity: "hidden-order.program.v1",
        contribution: HiddenOrderContribution {
            identity: "hidden-order.program.contrib.v1",
            members: [VaultFrag, HiddenPrincipal, HiddenAccessReg, HiddenQueryReg],
        },
        feature: HiddenOrderFeature => HiddenOrderDefinition / HiddenOrderRuntime {
            identity: "hidden-order.feature.v1",
        }
    }
}
