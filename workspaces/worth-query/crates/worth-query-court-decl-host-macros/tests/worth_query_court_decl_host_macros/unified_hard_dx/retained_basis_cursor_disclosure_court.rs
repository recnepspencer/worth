//! End-to-end Store M4.4 governed hidden-ordering disclosure court:
//! proves that a governed query whose ordering key is withheld from projection
//! orders and paginates accurately across Directory Home reopen WITHOUT leaking
//! a single plaintext byte of the hidden ordering value into the `whc1_...`
//! wire token, its decoded payload, `canonical_key_position()`, or `Debug`.

use std::num::NonZeroUsize;
use std::time::Duration;

use worth_query_decl::facade::authentication::*;
use worth_query_host::facade::admission::authenticated_principal::*;
use worth_query_host::facade::historical_cursor::{
    AdmittedQueryWindowBinding, AdmittedQueryWindowPage, AdmittedQueryWindowSource,
    CancellationSource, CanonicalCursorKey, CanonicalKeyDomain, ExecutionRequest,
    HistoricalBasisRetentionKind, HistoricalQueryContract, HistoricalReadDenial, ResidencyBudget,
    SchemaId, SchemaVersionId, SchemaVersionIdentity, SerialMemoryBudget, SerialRequest,
    PROJECTION_QUERY_ROWS, RETAINED_BASIS_TOKEN_PREFIX,
};
use worth_query_host::facade::{
    application_installation::ApplicationBuildIdentity, primary_graph::*,
    WorthQueryApplicationHome, WorthQueryHomeConfig, WorthQueryHostSession, WorthQuerySessionQuery,
};

use super::courts::temp_home_path;
use super::retained_basis_cursor_disclosure_query::{
    HiddenOrderProgram, HiddenOrderRuntime, InspectVault, RankedItemRow, RankedItemsIntent,
    ReadVault, ReadVaultAccess,
};
use super::retained_basis_cursor_disclosure_schema::*;

struct GovernedWindowSource<'home> {
    session: WorthQueryHostSession<'home, HiddenSchema, HiddenOrderProgram>,
    vault: String,
}

impl AdmittedQueryWindowSource<RankedItemRow> for GovernedWindowSource<'_> {
    fn read_window(
        &mut self,
        _execution: &ExecutionRequest<'_, '_>,
        window: WorthQueryApplicationRootWindow,
    ) -> Result<AdmittedQueryWindowPage<RankedItemRow>, HistoricalReadDenial> {
        let scope = self.session.fresh_scope();
        let published = self
            .session
            .request(&scope)
            .query(RankedItemsIntent {
                vault: self.vault.clone(),
            })
            .window(window)
            .execute_governed(
                ReadVault::reference(),
                InspectVault::reference(),
                ReadVaultAccess {
                    vault: self.vault.clone(),
                },
            )
            .expect("governed hidden-ordering window executes");
        let page = published
            .into_root_window_page()
            .expect("governed window emits sealed page");
        AdmittedQueryWindowPage::from_executed_page(page)
    }
}

fn decode_base64url_no_pad(input: &str) -> Vec<u8> {
    let mut out = Vec::new();
    let mut acc = 0u32;
    let mut bits = 0u32;
    for b in input.bytes() {
        let val = match b {
            b'A'..=b'Z' => (b - b'A') as u32,
            b'a'..=b'z' => (b - b'a' + 26) as u32,
            b'0'..=b'9' => (b - b'0' + 52) as u32,
            b'-' => 62,
            b'_' => 63,
            _ => panic!("invalid base64url byte {b}"),
        };
        acc = (acc << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((acc >> bits) & 0xFF) as u8);
        }
    }
    out
}

#[test]
fn retained_basis_cursor_hides_governed_ordering_values_in_wire_token_and_resumes_across_reopen() {
    let path = temp_home_path("retained-basis-governed-hidden-ordering");
    let clock = WorthQueryAdmissionClock::system();
    let build = ApplicationBuildIdentity::parse("6b6b6b6b-6b6b-6b6b-6b6b-6b6b6b6b6b6b").unwrap();
    let reader_id = WorthQueryExternalPrincipalIdentity::new(HiddenIssuer, "reader").unwrap();
    let admin_id = WorthQueryExternalPrincipalIdentity::new(HiddenIssuer, "admin").unwrap();
    let open_home = || {
        let reader_id = reader_id.clone();
        let admin_id = admin_id.clone();
        WorthQueryApplicationHome::<HiddenSchema, HiddenOrderProgram>::open_directory(
            &path,
            WorthQueryHomeConfig::new(build, (HiddenOrderRuntime::new(),))
                .with_clock(clock.clone())
                .with_initial_state(move |_, graph, installed| {
                    let pb = installed
                        .principal_binding(HiddenPrincipal::reference())
                        .unwrap();
                    graph.bind_principal(
                        &pb,
                        WorthQueryApplicationPrincipalKey::new("reader-1").unwrap(),
                        1_u64,
                        reader_id,
                        WorthQueryPrincipalMappingStatus::Enabled,
                    )?;
                    graph.bind_principal(
                        &pb,
                        WorthQueryApplicationPrincipalKey::new("admin-2").unwrap(),
                        2_u64,
                        admin_id,
                        WorthQueryPrincipalMappingStatus::Enabled,
                    )?;
                    let vkey =
                        || WorthQueryApplicationEntityKey::new("vault-v1".to_owned()).unwrap();
                    let gkey =
                        || WorthQueryApplicationEntityKey::new("grant-v1".to_owned()).unwrap();
                    graph.bind_entity(
                        WorthQueryApplicationEntitySeed::new(Vault::reference(), vkey())
                            .field(VaultKey::at("VaultKey"), "v1".to_owned()),
                    )?;
                    graph.bind_entity(
                        WorthQueryApplicationEntitySeed::new(VaultGrant::reference(), gkey())
                            .field(GId::at("GId"), "g-v1".to_owned())
                            .field(GAct::at("GAct"), 1_u64)
                            .field(GPurp::at("GPurp"), 1_u64)
                            .field(GStat::at("GStat"), 1_u64)
                            .field(GScope::at("GScope"), "v1".to_owned())
                            .field(GNb::at("GNb"), 0_u64)
                            .field(GNa::at("GNa"), u64::MAX / 2)
                            .field(GDepth::at("GDepth"), 0_u64)
                            .field(GDisc::at("GDisc"), 1_u64),
                    )?;
                    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                        GrGrantee::reference(),
                        "grantee-v1".to_owned(),
                        WorthQueryApplicationEntityKey::new("reader-1".to_owned()).unwrap(),
                        gkey(),
                    ))?;
                    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                        GrGrantor::reference(),
                        "grantor-v1".to_owned(),
                        WorthQueryApplicationEntityKey::new("admin-2".to_owned()).unwrap(),
                        gkey(),
                    ))?;
                    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                        GrResource::reference(),
                        "resource-v1".to_owned(),
                        gkey(),
                        vkey(),
                    ))?;
                    for (idx, item_key) in (1_u64..=6).enumerate() {
                        let rank = format!("CLASSIFIED-OMEGA-RANK-{:02}", 60 - idx * 10);
                        let ikey = || {
                            WorthQueryApplicationEntityKey::new(format!("item-{item_key}")).unwrap()
                        };
                        graph.bind_entity(
                            WorthQueryApplicationEntitySeed::new(Item::reference(), ikey())
                                .field(ItemKey::at("ItemKey"), item_key)
                                .field(Label::at("Label"), format!("Public {item_key}"))
                                .field(SecretRank::at("SecretRank"), rank),
                        )?;
                        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                            Holds::reference(),
                            format!("holds-{item_key}"),
                            vkey(),
                            ikey(),
                        ))?;
                    }
                    Ok(())
                }),
        )
        .expect("open governed hidden-ordering home")
    };
    let auth_reader = |home: &WorthQueryApplicationHome<HiddenSchema, HiddenOrderProgram>| {
        let scope = WorthQueryRequestScope::new(
            None,
            WorthQueryCancellationSource::new().token(),
            clock.clone(),
        );
        let local = WorthQueryLocalProcessPrincipal::new(
            reader_id.clone(),
            HiddenAudience,
            HiddenMethod,
            Duration::from_secs(600),
        )
        .unwrap();
        admit_authentication_adapter(
            home.primary_graph().installed_schema(),
            WorthQueryAuthenticationAdapterAdmission::new(
                WorthQueryAuthenticationAudience::of(HiddenAudience),
                WorthQueryAuthenticationMethod::of(HiddenMethod),
                clock.clone(),
            ),
            local,
        )
        .unwrap()
        .authenticate_local(&scope)
        .unwrap()
    };
    let schema = SchemaVersionIdentity::new(
        SchemaId("hidden.order.schema".into()),
        SchemaVersionId(1),
        [0x6B; 32],
    )
    .unwrap();
    let contract = HistoricalQueryContract::new(
        schema.clone(),
        CanonicalKeyDomain::Entity,
        None,
        PROJECTION_QUERY_ROWS,
        "hidden.order.governed.v1".to_owned(),
    );
    let budget = ResidencyBudget {
        head_working_set_bytes: 128 * 1024,
        historical_working_set_bytes: 64 * 1024,
        max_in_flight_fill_bytes: 32 * 1024,
        streaming_window_bytes: 16 * 1024,
    };
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let exec = ExecutionRequest::serial(&serial);

    let (p1_keys, wire, binding) = {
        let home = open_home();
        let me = auth_reader(&home);
        let session = home.session(&me);
        let receipt = home
            .retain_historical_basis(
                &session,
                "hidden-rank-snap",
                HistoricalBasisRetentionKind::PinLease,
                schema.clone(),
                budget,
            )
            .expect("retain basis");
        let intent = RankedItemsIntent { vault: "v1".into() };
        let contract_digest = intent.installed_contract_digest(&session).unwrap();
        let request_binding_digest = intent.admitted_request_binding_digest(&session).unwrap();
        let probe_scope = session.fresh_scope();
        let snap_ver = session
            .request(&probe_scope)
            .query(intent)
            .window(WorthQueryApplicationRootWindow::first(
                NonZeroUsize::new(1).unwrap(),
            ))
            .execute_governed(
                ReadVault::reference(),
                InspectVault::reference(),
                ReadVaultAccess { vault: "v1".into() },
            )
            .unwrap()
            .into_root_window_page()
            .unwrap()
            .progress()
            .snapshot_version();
        let binding = AdmittedQueryWindowBinding {
            contract_digest,
            request_binding_digest,
            snapshot_version: snap_ver,
        };
        let store = home
            .durable_historical_basis_store(budget)
            .unwrap()
            .unwrap();
        let mut cursor = store
            .open_admitted_query_cursor(
                &exec,
                receipt.commit_id,
                receipt.root_semantic_commitment,
                contract.clone(),
                binding,
                &schema,
                |_| {
                    Ok(Box::new(GovernedWindowSource {
                        session: home.session(&me),
                        vault: "v1".into(),
                    }))
                },
            )
            .expect("open governed hidden-ordering cursor");
        let p1 = cursor.next_page(&exec, 2).expect("page 1");
        assert!(p1.has_more());
        let token = p1.checkpoint_token().clone();
        let wire = token.encode_wire();
        assert!(wire.starts_with(RETAINED_BASIS_TOKEN_PREFIX));
        let wire_payload = decode_base64url_no_pad(&wire[RETAINED_BASIS_TOKEN_PREFIX.len()..]);
        let key_bytes: &[u8] = match token.canonical_key_position() {
            CanonicalCursorKey::Start => &[],
            CanonicalCursorKey::After { logical_key, .. } => logical_key.as_slice(),
        };
        assert_eq!(
            key_bytes.len(),
            84,
            "sealed query cursor key must be fixed 84-byte authenticated ciphertext"
        );
        let debug_str = format!("{token:?}");
        for forbidden in ["CLASSIFIED", "OMEGA", "RANK-20", "RANK-10"] {
            assert!(!wire.contains(forbidden), "wire leaked {forbidden}");
            assert!(!debug_str.contains(forbidden), "Debug leaked {forbidden}");
            assert!(
                !wire_payload
                    .windows(forbidden.len())
                    .any(|w| w == forbidden.as_bytes()),
                "decoded wire payload leaked {forbidden}"
            );
            assert!(
                !key_bytes
                    .windows(forbidden.len())
                    .any(|w| w == forbidden.as_bytes()),
                "canonical_key_position leaked {forbidden}"
            );
        }
        let rows = p1.into_rows();
        assert!(rows
            .iter()
            .all(|r| matches!(r.secret_rank, WorthQueryApplicationDisclosed::Omitted(_))));
        let p1_keys: Vec<u64> = rows.iter().map(|r| r.item_key).collect();
        assert_eq!(p1_keys, vec![6, 5]);
        drop(cursor);
        drop((store, session, me));
        home.close_clean().expect("close clean before reopen");
        (p1_keys, wire, binding)
    };

    // Cold reopen Directory Home and resume from the sealed wire token.
    {
        let home = open_home();
        let me = auth_reader(&home);
        let store = home
            .durable_historical_basis_store(budget)
            .unwrap()
            .unwrap();
        let mut resumed = store
            .resume_admitted_query_cursor_wire(&exec, &wire, contract, binding, &schema, |_| {
                Ok(Box::new(GovernedWindowSource {
                    session: home.session(&me),
                    vault: "v1".into(),
                }))
            })
            .expect("resume governed hidden-ordering cursor after reopen");
        let mut all_keys = p1_keys;
        loop {
            let page = resumed.next_page(&exec, 2).expect("resumed page");
            let more = page.has_more();
            for row in page.into_rows() {
                assert!(matches!(
                    row.secret_rank,
                    WorthQueryApplicationDisclosed::Omitted(_)
                ));
                all_keys.push(row.item_key);
            }
            if !more {
                break;
            }
        }
        assert_eq!(all_keys, vec![6, 5, 4, 3, 2, 1]);
        drop(resumed);
        drop((store, me));
        home.close_clean().expect("final close clean");
    }
    let _ = std::fs::remove_dir_all(path);
}
