
    use {crate::phase_eight_process::history::parent_oracle::durable::root_manifest, crate::phase_eight_process::history::parent_oracle::durable::RawFrame, crate::phase_eight_process::history::parent_oracle::durable::ROOT_MANIFEST_KIND};

    #[test]
    fn root_generation_link_covers_the_last_arena_free_space_reference_bytes() {
        let mut payload = [0_u8; 336];
        payload[..8].copy_from_slice(&1_u64.to_le_bytes());
        payload[232] = 1;
        fn frame(payload: &[u8]) -> RawFrame<'_> {
            RawFrame {
                kind: ROOT_MANIFEST_KIND,
                format: [2, 0, 0, 64, 0, 0, 1, 1, 1, 24],
                identity: 1,
                page_lsn: 1,
                payload,
            }
        }
        let before = root_manifest(&frame(&payload))
            .expect("root reference shape")
            .generation_links;
        payload[304] = 1;
        let after = root_manifest(&frame(&payload))
            .expect("root reference shape")
            .generation_links;
        assert_ne!(before, after);
    }

    #[test]
    fn free_space_member_count_overflow_is_not_valid_semantic_evidence() {
        let mut payload = [0_u8; 168];
        payload[..8].copy_from_slice(&1_u64.to_le_bytes());
        payload[24..32].copy_from_slice(&u64::MAX.to_le_bytes());
        payload[64] = 1;
        payload[144..152].copy_from_slice(&1_u64.to_le_bytes());
        payload[152..160].copy_from_slice(&4096_u64.to_le_bytes());
        payload[160..168].copy_from_slice(&4096_u64.to_le_bytes());
        let frame = RawFrame {
            kind: crate::phase_eight_process::history::parent_oracle::durable::FREE_SPACE_MANIFEST_KIND,
            format: [2, 0, 0, 64, 0, 0, 1, 1, 1, 24],
            identity: 1,
            page_lsn: 1,
            payload: &payload,
        };
        assert!(crate::phase_eight_process::history::parent_oracle::durable::free_space_manifest(&frame).is_none());
    }
