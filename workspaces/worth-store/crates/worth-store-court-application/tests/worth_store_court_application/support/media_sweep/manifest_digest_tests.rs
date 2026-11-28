use sha2::{Digest, Sha256};
use worth_runtime_world::facade::{
    ProductBranchCreationIntent, ProductBranchCreationPlans, RelationalBranchCreationPlan,
    RuntimeWorldBranchCreationOutcome, RuntimeWorldCancellationSource,
    RuntimeWorldHomeBranchCreationRequest, SignalBranchCreationPlan,
};
use worth_store::physical_runtime::AnchoredRecordName;

use super::manifest_digest::{encode_canonical_home_cut, relational_manifest_digest};
use super::section_digests::Wsrcai01Bindings;
use super::signal_digest::verify_application_journals;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Fault {
    None,
    SignalChecksum,
    WorldChecksum,
    CutOrigin,
    WorldCovDigest,
}

#[derive(Clone, Copy)]
struct EnvelopeOptions<'a> {
    home_id: [u8; 16],
    runtime_id: &'a [u8],
    branch_tag: &'a str,
    hwm: u8,
    signal_byte: u8,
    fault: Fault,
}

fn sha256_all(parts: &[&[u8]]) -> [u8; 32] {
    let mut h = Sha256::new();
    for p in parts {
        h.update(p);
    }
    h.finalize().into()
}

fn build_envelope(opts: EnvelopeOptions<'_>) -> Vec<u8> {
    let mut cp = b"WORTH-REL-CHK25\n\xb3runtime_instance_id".to_vec();
    cp.extend_from_slice(opts.runtime_id);
    cp.push(0xa0 + opts.branch_tag.len() as u8);
    cp.extend_from_slice(opts.branch_tag.as_bytes());
    cp.extend_from_slice(b"\xafrecord_identity");
    cp.push(opts.hwm);

    let mut source_payload = vec![0u8; 40];
    source_payload[16..24].copy_from_slice(&(cp.len() as u64).to_le_bytes());
    source_payload.extend_from_slice(&cp);
    let source = framed(b"WRTHRCS1", 3, &source_payload);

    let mut world_payload = vec![0u8; 229 + 48];
    world_payload[0..4].copy_from_slice(&1u32.to_le_bytes());
    world_payload[4..20].copy_from_slice(&opts.home_id);
    if opts.fault == Fault::WorldCovDigest {
        world_payload[76] ^= 0xff;
    }
    let mut world = vec![0u8; 52];
    world[..8].copy_from_slice(b"WWRLDCP1");
    world[8..12].copy_from_slice(&1u32.to_le_bytes());
    world[12..20].copy_from_slice(&(world_payload.len() as u64).to_le_bytes());
    world[20..52].copy_from_slice(&Sha256::digest(&world_payload));
    if opts.fault == Fault::WorldChecksum {
        world[20] ^= 0x01;
    }
    world.extend_from_slice(&world_payload);
    let world_image_digest: [u8; 32] = Sha256::digest(&world).into();
    let source_digest: [u8; 32] = Sha256::digest(&source).into();
    let (zero32, zero24, zero8) = ([0u8; 32], [0u8; 24], [0u8; 8]);
    let mut cut_origin = zero32;
    if opts.fault == Fault::CutOrigin {
        cut_origin[0] ^= 0xff;
    }
    let cut = encode_canonical_home_cut(&Wsrcai01Bindings {
        home: &opts.home_id,
        covered: &zero8,
        covered_transitions: &zero8,
        native_origin: &cut_origin,
        world_through: &zero24,
        checkpoint_digest: &zero32,
        world_image_digest: &world_image_digest,
        source_digest: &source_digest,
        image_digest: &zero32,
    });

    let roots_hdr = build_group_header(&cut, 0, 0, 9, b"r");
    let mut image = vec![0u8; 64];
    image[..8].copy_from_slice(b"WRTHCAI1");
    image[8..12].copy_from_slice(&1u32.to_le_bytes());
    image[12..16].copy_from_slice(&(cut.len() as u32).to_le_bytes());
    image[24..32].copy_from_slice(&1u64.to_le_bytes());
    let d_dig = sha256_all(&[
        b"worth.relational.checkpoint-admission.directory.v1\0",
        &image[..32],
        &cut,
        &roots_hdr,
    ]);
    image[32..64].copy_from_slice(&d_dig);
    image.extend_from_slice(&cut);
    image.extend_from_slice(&roots_hdr);
    image.extend_from_slice(b"r");
    let image_digest: [u8; 32] = Sha256::digest(&image).into();

    let mut companion_payload = [0u8; 217];
    companion_payload[0] = 1;
    companion_payload[1..17].copy_from_slice(&opts.home_id);
    companion_payload[97..129].copy_from_slice(&source_digest);
    companion_payload[153..185].copy_from_slice(&world_image_digest);
    companion_payload[185..217].copy_from_slice(&image_digest);
    let companion = framed(b"WRWMREG1", 2, &companion_payload);

    let mut native = vec![0u8; 292];
    native[..8].copy_from_slice(b"WSRCAI01");
    native[8..12].copy_from_slice(&1u32.to_le_bytes());
    native[68..84].copy_from_slice(&opts.home_id);
    native[140..172].copy_from_slice(&world_image_digest);
    for (i, child) in [source.as_slice(), &image, &companion]
        .into_iter()
        .enumerate()
    {
        native[172 + i * 8..180 + i * 8].copy_from_slice(&(child.len() as u64).to_le_bytes());
        native[196 + i * 32..228 + i * 32].copy_from_slice(&Sha256::digest(child));
    }
    native.extend_from_slice(&source);
    native.extend_from_slice(&image);
    native.extend_from_slice(&companion);

    let mut sig_payload = [0u8; 101];
    sig_payload[..4].copy_from_slice(&5u32.to_le_bytes());
    sig_payload[4..20].copy_from_slice(&opts.home_id);
    sig_payload[28] = opts.signal_byte;
    let mut signal = framed(b"WSIGOWNR", 2, &sig_payload);
    if opts.fault == Fault::SignalChecksum {
        signal[20] ^= 0x01;
    }
    assemble_waphckp1([0, 1, 2], [&native, &world, &signal])
}

fn framed(magic: &[u8; 8], version: u32, payload: &[u8]) -> Vec<u8> {
    let mut output = magic.to_vec();
    output.extend_from_slice(&version.to_le_bytes());
    output.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    output.extend_from_slice(&Sha256::digest(payload));
    output.extend_from_slice(payload);
    output
}

fn build_group_header(cut: &[u8], pos: u64, commit: u64, meaning: u8, body: &[u8]) -> [u8; 41] {
    let mut hdr = [0u8; 41];
    hdr[0] = meaning;
    hdr[1..9].copy_from_slice(&(body.len() as u64).to_le_bytes());
    hdr[9..41].copy_from_slice(&sha256_all(&[
        b"worth.relational.checkpoint-admission.meaning.v1\0",
        &1u32.to_le_bytes(),
        &(cut.len() as u64).to_le_bytes(),
        cut,
        &pos.to_le_bytes(),
        &commit.to_le_bytes(),
        &[meaning],
        &(body.len() as u64).to_le_bytes(),
        body,
    ]));
    hdr
}

fn assemble_waphckp1(tags: [u16; 3], sections: [&[u8]; 3]) -> Vec<u8> {
    let mut waph = b"WAPHCKP1\x03\x00\x00\x00\x03\x00".to_vec();
    for (tag, section) in tags.into_iter().zip(sections) {
        waph.extend_from_slice(&tag.to_le_bytes());
        waph.extend_from_slice(&(section.len() as u64).to_le_bytes());
    }
    for section in sections {
        waph.extend_from_slice(section);
    }
    waph
}

/// Flip 1 byte in a real `WAPHCKP1` checkpoint's nonterminal pos-1 World
/// record digest (on a `world_through[0] >= 2` prefix) and validly recompute
/// every enclosing `WWRLDCP1`, `WRTHCAI1`, `WSRCAI01`, and `WRWMREG1` checksum
/// so the checkpoint parser accepts it while journal verification fails.
fn mutate_real_world_nonterminal_pos1_digest(real_waph: &[u8]) -> Vec<u8> {
    let u64_at =
        |b: &[u8], off: usize| u64::from_le_bytes(b[off..off + 8].try_into().unwrap()) as usize;
    let tags = [14, 24, 34].map(|o| u16::from_le_bytes(real_waph[o..o + 2].try_into().unwrap()));
    let n_len = u64_at(real_waph, 16);
    let w_len = u64_at(real_waph, 26);
    let native_orig = &real_waph[44..44 + n_len];
    let mut world = real_waph[44 + n_len..44 + n_len + w_len].to_vec();
    let signal = &real_waph[44 + n_len + w_len..];

    let (pos_off, digest_off) = match world[52 + 228] {
        1 => (52 + 229, 52 + 237),
        0 => (52 + 237, 52 + 245),
        other => panic!("unexpected WWRLDCP1 owner_tag {other}"),
    };
    assert_eq!(u64_at(&world, pos_off), 1, "expected pos 1 record first");
    world[digest_off] ^= 0xff;
    let wp_digest = Sha256::digest(&world[52..]);
    world[20..52].copy_from_slice(&wp_digest);
    let new_world_digest: [u8; 32] = Sha256::digest(&world).into();

    let s_len = u64_at(native_orig, 172);
    let i_len = u64_at(native_orig, 180);
    let source = &native_orig[292..292 + s_len];
    let img_orig = &native_orig[292 + s_len..292 + s_len + i_len];
    let mut companion = native_orig[292 + s_len + i_len..].to_vec();

    // Rebuild WRTHCAI1 with updated canonical cut (`world_image_digest`) and
    // validly recomputed group + directory SHA-256 digests.
    let old_cut_len = u32::from_le_bytes(img_orig[12..16].try_into().unwrap()) as usize;
    let row_count = u64_at(img_orig, 16);
    let new_cut = encode_canonical_home_cut(&Wsrcai01Bindings {
        home: &native_orig[68..84],
        covered: &native_orig[12..20],
        covered_transitions: &native_orig[20..28],
        native_origin: &native_orig[36..68],
        world_through: &native_orig[84..108],
        checkpoint_digest: &native_orig[108..140],
        world_image_digest: &new_world_digest,
        source_digest: &native_orig[196..228],
        image_digest: &native_orig[228..260],
    });
    assert_eq!(new_cut.len(), old_cut_len);
    let mut new_img = img_orig[..64].to_vec();
    let mut dir = Sha256::new();
    dir.update(b"worth.relational.checkpoint-admission.directory.v1\0");
    dir.update(&new_img[..32]);
    dir.update(&new_cut);
    new_img.extend_from_slice(&new_cut);
    let mut off = 64 + old_cut_len;
    let rehash_group =
        |pos: u64, commit: u64, off: &mut usize, dir: &mut Sha256, out: &mut Vec<u8>| {
            let len = u64_at(img_orig, *off + 1);
            let body = &img_orig[*off + 41..*off + 41 + len];
            let hdr = build_group_header(&new_cut, pos, commit, img_orig[*off], body);
            dir.update(&hdr);
            out.extend_from_slice(&hdr);
            out.extend_from_slice(body);
            *off += 41 + len;
        };
    rehash_group(0, 0, &mut off, &mut dir, &mut new_img);
    for _ in 0..row_count {
        let pos = u64::from_le_bytes(img_orig[off..off + 8].try_into().unwrap());
        let commit = u64::from_le_bytes(img_orig[off + 8..off + 16].try_into().unwrap());
        dir.update(&img_orig[off..off + 16]);
        new_img.extend_from_slice(&img_orig[off..off + 16]);
        off += 16;
        for _ in 0..8 {
            rehash_group(pos, commit, &mut off, &mut dir, &mut new_img);
        }
    }
    assert_eq!(off, img_orig.len());
    new_img[32..64].copy_from_slice(&dir.finalize());
    let new_img_digest: [u8; 32] = Sha256::digest(&new_img).into();

    companion[205..237].copy_from_slice(&new_world_digest);
    companion[237..269].copy_from_slice(&new_img_digest);
    let c_dig = Sha256::digest(&companion[52..]);
    companion[20..52].copy_from_slice(&c_dig);

    let mut native = native_orig[..292].to_vec();
    native[140..172].copy_from_slice(&new_world_digest);
    native[228..260].copy_from_slice(&new_img_digest);
    native[260..292].copy_from_slice(&Sha256::digest(&companion));
    native.extend_from_slice(source);
    native.extend_from_slice(&new_img);
    native.extend_from_slice(&companion);
    assemble_waphckp1(tags, [&native, &world, signal])
}

#[test]
fn manifest_digest_verifies_typed_bindings_and_all_sections() {
    let base = EnvelopeOptions {
        home_id: [1; 16],
        runtime_id: &[7u8],
        branch_tag: "relational/7/main",
        hwm: 42,
        signal_byte: 0,
        fault: Fault::None,
    };
    let d1 = relational_manifest_digest(&build_envelope(base))
        .expect("valid canonical manifest digests")
        .digest;
    let d2 = relational_manifest_digest(&build_envelope(EnvelopeOptions {
        home_id: [2; 16],
        runtime_id: &[0xce, 0x3b, 0x9a, 0xca, 0x04],
        branch_tag: "relational/1000000004/main",
        ..base
    }))
    .expect("different runtime_instance_id and home_id still digest")
    .digest;
    assert_eq!(d1, d2, "only per-runtime id and home uuid differed");
    assert_ne!(
        d1,
        relational_manifest_digest(&build_envelope(EnvelopeOptions { hwm: 43, ..base }))
            .expect("mutated high-water mark still digests")
            .digest,
        "record_identity change must alter digest"
    );
    assert_ne!(
        d1,
        relational_manifest_digest(&build_envelope(EnvelopeOptions {
            signal_byte: 1,
            ..base
        }))
        .expect("valid mutated signal payload still digests")
        .digest,
        "Signal body change must alter digest"
    );
    for (label, fault) in [
        ("malformed Signal checksum", Fault::SignalChecksum),
        ("corrupted World checksum", Fault::WorldChecksum),
        ("WRTHCAI1 cut binding mismatch", Fault::CutOrigin),
        (
            "WWRLDCP1 terminal coverage digest mismatch",
            Fault::WorldCovDigest,
        ),
    ] {
        assert!(
            relational_manifest_digest(&build_envelope(EnvelopeOptions { fault, ..base }))
                .is_none(),
            "{label} must fail verification"
        );
    }
}

#[test]
fn real_world_prefix_rejects_mutated_nonterminal_record_digest_via_journal_verifier() {
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let dir = tempfile::tempdir().unwrap();
    let location = dir.path().join("home");
    let schema = crate::schema_transition::SOURCE;
    let fixture = crate::durable_run::world::open(&location, schema, false, &execution).unwrap();
    let boot = fixture.bootstrap_root(&execution);
    let source = boot.product_branch().clone();
    let plans = ProductBranchCreationPlans::new(
        RelationalBranchCreationPlan::ReuseExact,
        SignalBranchCreationPlan::ReuseExact,
    );
    let intent = ProductBranchCreationIntent::from_source("second-world-branch", plans).unwrap();
    let port = fixture.home.world().branch_port();
    let cancel = RuntimeWorldCancellationSource::new();
    let req = RuntimeWorldHomeBranchCreationRequest::new(
        source.clone(),
        fixture.admit_initial_program_source(&source),
        intent,
    );
    let mut outcome = port
        .create_home_product_branch(&execution, req, &cancel.token())
        .unwrap();
    while let RuntimeWorldBranchCreationOutcome::ProductUnpublished(effects) = outcome {
        let handle = effects.recovery_handle();
        drop(effects);
        std::thread::yield_now();
        outcome = port
            .resume_creation(&execution, &handle, &cancel.token())
            .unwrap();
    }
    assert!(matches!(
        outcome,
        RuntimeWorldBranchCreationOutcome::Performed(_)
    ));
    drop((source, outcome, boot));
    fixture
        .home
        .checkpoint(&execution)
        .expect("real World checkpoint");
    crate::durable_run::world::close(fixture, &execution);

    let clean = crate::durable_run::open_declaring(&location, schema, &execution).unwrap();
    crate::media_case::with_serving_for_test(&clean, |serving| {
        let records = serving.anchored_records().unwrap();
        let anchor = AnchoredRecordName::new(crate::selected_checkpoint::ANCHOR).unwrap();
        let record = records.read(anchor, 1 << 30).unwrap().expect("checkpoint");

        // Positive control: real >=2-entry `world.branches` prefix passes both
        // checkpoint parsing and end-to-end `verify_application_journals`.
        let ctrl = relational_manifest_digest(record.bytes()).expect("real checkpoint digests");
        assert!(ctrl.world_through[0] >= 2, "got {:?}", ctrl.world_through);
        assert!(verify_application_journals(serving, Some(&ctrl)).is_some());

        // Mutate the nonterminal pos-1 record digest and validly recompute every
        // enclosing checksum: checkpoint parser accepts, journal verifier rejects.
        let mutated_bytes = mutate_real_world_nonterminal_pos1_digest(record.bytes());
        let mutated = relational_manifest_digest(&mutated_bytes).expect("mutated parses");
        assert!(verify_application_journals(serving, Some(&mutated)).is_none());
    });
}
