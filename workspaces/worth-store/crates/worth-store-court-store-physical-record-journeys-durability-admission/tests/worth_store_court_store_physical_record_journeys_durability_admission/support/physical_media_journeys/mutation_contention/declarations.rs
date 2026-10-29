pub(crate) use {std::io::Read, std::io::Write};
pub(crate) use {std::path::Path};

pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::MediaShutdownOutcome};
pub(crate) use {worth_store_physical_backend::CertificationConfinementEffect, worth_store_physical_backend::MediaEffectStatus, worth_store_physical_backend::NamespaceConfinementDenialKind};

pub(crate) use {crate::child_dispatch::emit, crate::child_dispatch::run_role};
pub(crate) use {crate::physical_media_journeys::admit_runtime, crate::physical_media_journeys::media_admission};

pub(crate) const EVENT_PREFIX: &str = "C4_EVENT ";
pub(crate) const CONTENDER_COUNT: usize = 8;





pub(crate) use {crate::mutation_contention::counter_evidence::assert_loser_counters, crate::mutation_contention::counter_evidence::campaign_counter_projection, crate::mutation_contention::counter_evidence::exact_counter_projection, crate::mutation_contention::counter_evidence::inspection_counters};
pub(crate) use {crate::physical_media_journeys::mutation_contention::death_boundary::contention_admission, crate::physical_media_journeys::mutation_contention::death_boundary::reach_death_boundary};
pub(crate) use {crate::mutation_contention::independent_observation::create_file_link, crate::mutation_contention::independent_observation::create_sentinel, crate::mutation_contention::independent_observation::expected_store_manifest, crate::mutation_contention::independent_observation::manifest_projection, crate::mutation_contention::independent_observation::observe_tree};
pub(crate) use {crate::mutation_contention::process_barrier::Contender};
pub(crate) const HOSTILE_COMPONENTS: [(&str, NamespaceConfinementDenialKind); 10] = [
    ("", worth_store_physical_backend::NamespaceConfinementDenialKind::EmptyPath),
    (".", worth_store_physical_backend::NamespaceConfinementDenialKind::SpecialComponent),
    ("..", worth_store_physical_backend::NamespaceConfinementDenialKind::SpecialComponent),
    (
        "../outside",
        worth_store_physical_backend::NamespaceConfinementDenialKind::EmbeddedSeparator,
    ),
    (
        "namespace/../../outside",
        worth_store_physical_backend::NamespaceConfinementDenialKind::EmbeddedSeparator,
    ),
    (
        "same/name",
        worth_store_physical_backend::NamespaceConfinementDenialKind::EmbeddedSeparator,
    ),
    (
        "same\\name",
        worth_store_physical_backend::NamespaceConfinementDenialKind::EmbeddedSeparator,
    ),
    ("nul", worth_store_physical_backend::NamespaceConfinementDenialKind::ReservedDeviceName),
    ("name:", worth_store_physical_backend::NamespaceConfinementDenialKind::AlternateDataStream),
    (
        "name ",
        worth_store_physical_backend::NamespaceConfinementDenialKind::NonPortableComponent,
    ),
];

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct CampaignProjection {
    pub(crate) owners: usize,
    pub(crate) contentions: usize,
    pub(crate) target_manifest_shape: Vec<String>,
    pub(crate) sentinel_manifest: String,
    pub(crate) winner_counters: String,
    pub(crate) loser_counters: Vec<String>,
    pub(crate) successor_counters: String,
}

pub(crate) fn run_campaign(enclosure: &Path) -> CampaignProjection {
    std::fs::create_dir(enclosure).unwrap();
    let root = enclosure.join("store");
    let sentinel = enclosure.join("outside-sentinel");
    create_sentinel(&sentinel);
    let sentinel_before = observe_tree(&sentinel);

    let mut contenders = (0..CONTENDER_COUNT)
        .map(|index| crate::mutation_contention::process_barrier::Contender::spawn(&root, index))
        .collect::<Vec<_>>();
    for contender in &contenders {
        assert_eq!(contender.event(), "ready");
    }
    for contender in &mut contenders {
        contender.send(b'S');
    }

    let mut winner = None;
    let mut stable_identity = None;
    let mut winner_owner = None;
    let mut winner_attempt = None;
    let mut winner_runtime = None;
    let mut loser_counters = Vec::new();
    let mut contender_processes = std::collections::BTreeSet::new();
    let mut contentions = 0;
    for (index, contender) in contenders.iter().enumerate() {
        let event = contender.event();
        if let Some(fields) = event.strip_prefix("owned;") {
            assert!(
                winner.replace(index).is_none(),
                "more than one owner admitted"
            );
            stable_identity = Some(field(fields, "store").to_owned());
            winner_owner = Some(field(fields, "owner").to_owned());
            winner_attempt = Some(field(fields, "attempt").to_owned());
            winner_runtime = Some(field(fields, "runtime").to_owned());
            assert!(contender_processes.insert(field(fields, "process").to_owned()));
            assert_eq!(
                field(fields, "confinement_denials"),
                expected_confinements()
            );
        } else {
            assert!(
                event.starts_with("contended;"),
                "unexpected contender event: {event}"
            );
            contentions += 1;
            assert!(contender_processes
                .insert(field(event.strip_prefix("contended;").unwrap(), "process").to_owned()));
            loser_counters
                .push(field(event.strip_prefix("contended;").unwrap(), "stable_counters").into());
        }
    }
    let winner = winner.expect("exactly one process must own mutation");
    assert_eq!(contentions, CONTENDER_COUNT - 1);
    for (index, contender) in contenders.iter_mut().enumerate() {
        if index != winner {
            assert!(contender.wait().success());
        }
    }

    let escape_probe = root.join("staging/confinement-probe");
    create_file_link(&sentinel.join("families/same-name"), &escape_probe);
    contenders[winner].send(b'E');
    let escape_event = contenders[winner].event();
    assert_eq!(field(&escape_event, "outcome"), "open-denied");
    assert_eq!(field(&escape_event, "status"), "denied-before-effect");
    assert_eq!(observe_tree(&sentinel), sentinel_before);
    std::fs::remove_file(&escape_probe).unwrap();

    contenders[winner].send(b'D');
    let death_boundary = contenders[winner].event();
    assert_eq!(field(&death_boundary, "role"), "release_mutation_lease");
    assert_eq!(field(&death_boundary, "ordinal"), "1");
    assert_eq!(field(&death_boundary, "operation"), "none");
    assert_eq!(field(&death_boundary, "handle"), "none");
    assert_eq!(field(&death_boundary, "release_attempts"), "1");
    assert_eq!(field(&death_boundary, "release_completed"), "0");
    assert_eq!(field(&death_boundary, "ownership_releases"), "0");
    let winner_counters = field(&death_boundary, "stable_counters").to_owned();
    contenders[winner].kill();
    let killed = contenders[winner].wait();
    assert!(
        !killed.success(),
        "OS termination must not look like a normal close"
    );

    let successor = run_role("post-death-successor", &root, &[]);
    assert_eq!(successor.value("store"), stable_identity.unwrap());
    assert_ne!(successor.value("owner"), winner_owner.unwrap());
    assert_ne!(successor.value("attempt"), winner_attempt.unwrap());
    assert_ne!(successor.value("runtime"), winner_runtime.unwrap());
    assert!(contender_processes.insert(successor.value("process").to_owned()));
    assert_eq!(contender_processes.len(), CONTENDER_COUNT + 1);
    assert_eq!(
        successor.value("confinement_denials"),
        expected_confinements()
    );
    assert_eq!(successor.value("conserved"), "true");
    assert_eq!(successor.number("live_files"), 0);
    assert_eq!(successor.number("live_directories"), 0);
    assert_eq!(successor.number("ownership_releases"), 1);
    assert_eq!(observe_tree(&sentinel), sentinel_before);
    loser_counters.sort();

    let target_manifest_shape = manifest_projection(&observe_tree(&root));
    assert_eq!(target_manifest_shape, expected_store_manifest());
    CampaignProjection {
        owners: 1,
        contentions,
        target_manifest_shape,
        sentinel_manifest: sentinel_before,
        winner_counters,
        loser_counters,
        successor_counters: successor.value("stable_counters").into(),
    }
}

pub(crate) fn run_child(role: &str, root: &Path) {
    match role {
        "mutation-contender" => contender_child(root),
        "unrelated-inheritance-probe" => contention_probe(root),
        "post-death-successor" => successor_child(root),
        _ => panic!("unsupported mutation role"),
    }
}

pub(crate) fn contender_child(root: &Path) {
    event("ready");
    let mut command = [0_u8; 1];
    std::io::stdin().read_exact(&mut command).unwrap();
    assert_eq!(command[0], b'S');
    let (admission, release_gate) = contention_admission();
    match admit_runtime(root)
        .try_admit_filesystem_media(admission)
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(media) => {
            let denied = run_confinement_corpus(&media);
            let observation = media.observer().snapshot().unwrap();
            event(&format!(
                "owned;store={};runtime={};process={};owner={};attempt={};confinement_denials={denied};counters={};stable_counters={}",
                hex(&media.store_identity().bytes()),
                media.runtime_identity().get(),
                std::process::id(),
                hex(&observation.mutation_owner().owner().bytes()),
                hex(&observation.mutation_owner().attempt().bytes()),
                exact_counter_projection(observation.media_counters()),
                campaign_counter_projection(observation.media_counters()),
            ));
            std::io::stdin().read_exact(&mut command).unwrap();
            assert_eq!(command[0], b'E');
            let effect = media.certification_staging_effect_probe(
                confinement_probe_authority(),
                "confinement-probe",
            );
            event(&format!(
                "escape-probe;outcome={};status={}",
                confinement_effect_name(effect),
                confinement_status_name(effect),
            ));
            std::io::stdin().read_exact(&mut command).unwrap();
            assert_eq!(command[0], b'D');
            let probe = run_role("unrelated-inheritance-probe", root, &[]);
            assert_eq!(probe.value("outcome"), "contended");
            reach_death_boundary(media, release_gate);
        }
        worth_proof::TransitionOutcome::Deferred(deferred) => {
            let counters = deferred.reason().counters();
            assert_loser_counters(counters, false);
            deferred.into_runtime().abort();
            event(&format!(
                "contended;effectful=false;process={};counters={};stable_counters={}",
                std::process::id(),
                exact_counter_projection(counters),
                campaign_counter_projection(counters),
            ));
        }
        worth_proof::TransitionOutcome::Failed(failure) => {
            let counters = inspection_counters(failure.cause());
            assert_loser_counters(counters, true);
            event(&format!(
                "contended;effectful=true;process={};counters={};stable_counters={}",
                std::process::id(),
                exact_counter_projection(counters),
                campaign_counter_projection(counters),
            ));
        }
        _ => panic!("contention returned a non-contention outcome"),
    }
}

pub(crate) fn contention_probe(root: &Path) {
    let runtime = admit_runtime(root);
    match runtime
        .try_admit_filesystem_media(media_admission())
        .into_raw()
    {
        worth_proof::TransitionOutcome::Deferred(deferred) => {
            assert_loser_counters(deferred.reason().counters(), false);
            deferred.into_runtime().abort();
            emit(&[("outcome", "contended".into())]);
        }
        _ => panic!("unrelated child inherited or acquired mutation authority"),
    }
}

pub(crate) fn successor_child(root: &Path) {
    let media = match admit_runtime(root)
        .try_admit_filesystem_media(media_admission())
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("successor failed to acquire the released OS lease"),
    };
    let denied = run_confinement_corpus(&media);
    let observation = media.observer().snapshot().unwrap();
    let observer = media.observer();
    let store = hex(&media.store_identity().bytes());
    assert!(matches!(media.close(), worth_store::physical_runtime::MediaShutdownOutcome::Released(_)));
    let counters = observer.media_counters();
    let fields = [
        ("store", store),
        ("runtime", observation.runtime_identity().get().to_string()),
        ("process", std::process::id().to_string()),
        ("owner", hex(&observation.mutation_owner().owner().bytes())),
        (
            "attempt",
            hex(&observation.mutation_owner().attempt().bytes()),
        ),
        ("confinement_denials", denied.to_string()),
        ("conserved", counters.is_conserved().to_string()),
        ("live_files", counters.live_file_handles().to_string()),
        (
            "live_directories",
            counters.live_directory_handles().to_string(),
        ),
        (
            "ownership_releases",
            counters.ownership_releases().to_string(),
        ),
        ("counters", exact_counter_projection(counters)),
        ("stable_counters", campaign_counter_projection(counters)),
    ];
    emit(&fields);
}

pub(crate) fn run_confinement_corpus(media: &worth_store::physical_runtime::MediaOwnedPhysicalRuntime) -> u64 {
    let before = media.media_counters().confinement_denials();
    let authority = confinement_probe_authority();
    for (component, expected) in HOSTILE_COMPONENTS {
        let denial = media
            .certification_confinement_probe(authority, component)
            .expect_err("hostile component escaped confinement");
        assert_eq!(denial.kind(), expected);
    }
    media.media_counters().confinement_denials() - before
}

pub(crate) fn confinement_probe_authority(
) -> worth_store::physical_runtime::certification::CertificationMediaFaultAuthority {
    media_admission().fault_schedule_authority()
}

pub(crate) fn expected_confinements() -> &'static str {
    "10"
}

pub(crate) fn confinement_effect_name(effect: CertificationConfinementEffect) -> &'static str {
    match effect {
        worth_store_physical_backend::CertificationConfinementEffect::ComponentDenied(_) => "component-denied",
        worth_store_physical_backend::CertificationConfinementEffect::OpenDenied(_) => "open-denied",
        worth_store_physical_backend::CertificationConfinementEffect::WriteReached(_) => "write-reached",
    }
}

pub(crate) fn confinement_status_name(effect: CertificationConfinementEffect) -> &'static str {
    let status = match effect {
        worth_store_physical_backend::CertificationConfinementEffect::ComponentDenied(_) => return "component-denial",
        worth_store_physical_backend::CertificationConfinementEffect::OpenDenied(status)
        | worth_store_physical_backend::CertificationConfinementEffect::WriteReached(status) => status,
    };
    match status {
        worth_store_physical_backend::MediaEffectStatus::DeniedBeforeEffect => "denied-before-effect",
        worth_store_physical_backend::MediaEffectStatus::PartialTransfer => "partial-transfer",
        worth_store_physical_backend::MediaEffectStatus::CompletedEffect => "completed-effect",
        worth_store_physical_backend::MediaEffectStatus::IndeterminateEffect => "indeterminate-effect",
        worth_store_physical_backend::MediaEffectStatus::UnsupportedCapability => "unsupported-capability",
        worth_store_physical_backend::MediaEffectStatus::StaleHandle => "stale-handle",
    }
}

pub(crate) fn event(message: &str) {
    println!("{EVENT_PREFIX}{message}");
    std::io::stdout().flush().unwrap();
}

pub(crate) fn field<'a>(fields: &'a str, name: &str) -> &'a str {
    fields
        .split(';')
        .find_map(|field| field.strip_prefix(&format!("{name}=")))
        .unwrap_or_else(|| panic!("event omitted {name}"))
}

pub(crate) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}