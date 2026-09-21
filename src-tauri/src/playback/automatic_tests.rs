use super::*;
use sha2::{Digest, Sha256};

struct Fixture {
    _temporary: tempfile::TempDir,
    game: PathBuf,
    cache: PathBuf,
    target: ReplayTarget,
}

fn big(path: &Path, names: &[&str]) {
    big_with_crc(path, names, 0x19);
}

fn big_with_crc(path: &Path, names: &[&str], crc: u32) {
    big_with_metadata(path, names, crc, "data/additionalmaps/mapmetadata_fixture");
}

fn big_with_metadata(path: &Path, names: &[&str], crc: u32, prefix: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut files: Vec<_> = names
        .iter()
        .map(|name| ((*name).to_owned(), vec![1u8]))
        .collect();
    let maps: Vec<_> = names
        .iter()
        .copied()
        .filter(|n| n.ends_with(".map"))
        .collect();
    if !maps.is_empty() {
        let (binary, manifest) = super::super::metadata::fixture(&maps, crc);
        files.push((format!("{prefix}.bin"), binary));
        files.push((format!("{prefix}.manifest"), manifest));
    }
    let end = 16 + files.iter().map(|(name, _)| 9 + name.len()).sum::<usize>();
    let size = end + files.iter().map(|(_, bytes)| bytes.len()).sum::<usize>();
    let mut data = b"BIGF".to_vec();
    data.extend((size as u32).to_be_bytes());
    data.extend((files.len() as u32).to_be_bytes());
    data.extend((end as u32).to_be_bytes());
    let mut offset = end;
    for (name, bytes) in &files {
        data.extend((offset as u32).to_be_bytes());
        data.extend((bytes.len() as u32).to_be_bytes());
        data.extend(name.as_bytes());
        data.push(0);
        offset += bytes.len();
    }
    for (_, bytes) in files {
        data.extend(bytes);
    }
    fs::write(path, data).unwrap();
}

impl Fixture {
    fn new() -> Self {
        let temporary = tempfile::Builder::new()
            .prefix("Tacitus replay é # ")
            .tempdir()
            .unwrap();
        let game = temporary.path().join("Game folder");
        for layer in [
            "Core/1.2",
            "Meta/1.2",
            "RetailExe/1.2",
            "Movies/1.0",
            "Lang-english/1.2",
            "EnglishAudio/1.2",
        ] {
            fs::create_dir_all(game.join(layer)).unwrap();
            fs::write(game.join(layer).join("config.txt"), "add-big base.big\n").unwrap();
            big(&game.join(layer).join("base.big"), &["base/asset"]);
        }
        fs::write(game.join("RetailExe/1.2/cnc3ep1.dat"), b"never executed").unwrap();
        fs::write(
            game.join("CNC3EP1_english_1.2.SkuDef"),
            "user's current mod selection\n",
        )
        .unwrap();
        fs::create_dir(game.join("Patch103")).unwrap();
        fs::write(
            game.join("Patch103/config.txt"),
            "add-big R24j1v1Maps.big\nadd-big 102Scripts.big\n",
        )
        .unwrap();
        let replay = temporary.path().join("Replay é #1.KWReplay");
        fs::write(&replay, b"fixture replay").unwrap();
        Self {
            cache: temporary.path().join("cache"),
            game,
            _temporary: temporary,
            target: ReplayTarget {
                id: 1,
                file_path: Some(replay.to_string_lossy().into_owned()),
                file_hash: hex::encode(Sha256::digest(b"fixture replay")),
                map_name: "[R24] Map".into(),
                map_path: "283data/maps/official/map 1.02+__24g".into(),
                map_crc: "19".into(),
                n_players: 2,
                version: [1, 2, 0, 0],
            },
        }
    }
    fn asset(revision: &str) -> String {
        format!("data/maps/official/map 1.02+__{revision}/map 1.02+__{revision}.map")
    }
    fn local(&self, revision: &str, versioned_scripts: bool) {
        big(
            &self.game.join(format!("Patch103/R{revision}1v1Maps.big")),
            &[&Self::asset(revision)],
        );
        let name = if versioned_scripts {
            format!("R{revision}Scripts.big")
        } else {
            "102Scripts.big".into()
        };
        big(
            &self.game.join("Patch103").join(name),
            &["data/scripts/scripts.lua"],
        );
    }
    fn cached(&self, revision: &str) -> PathBuf {
        self.cached_pack(revision, "1v1")
    }
    fn cached_pack(&self, revision: &str, group: &str) -> PathBuf {
        let pending = tempfile::tempdir_in(self._temporary.path()).unwrap();
        big(
            &pending.path().join(format!("R{revision}{group}Maps.big")),
            &[&Self::asset(revision)],
        );
        big(
            &pending.path().join("102Scripts.big"),
            &["data/scripts/scripts.lua"],
        );
        let cancellation = Cancellation::default();
        content::publish(
            pending.path(),
            &self.cache,
            &format!("R{revision}"),
            "https://example.test/pack",
            "package-hash",
            &Control {
                cancelled: &cancellation,
                progress: &|_| {},
            },
        )
        .unwrap()
    }
    fn prepare(&self) -> Result<Prepared> {
        let cancellation = Cancellation::default();
        prepare(
            &self.target,
            &self.game,
            &self.cache,
            false,
            &Control {
                cancelled: &cancellation,
                progress: &|_| {},
            },
        )
    }
}

#[test]
fn stock_map_requires_the_active_engines_compiled_compatibility() {
    let mut f = Fixture::new();
    f.target.map_path = "283data/maps/official/stock".into();
    big_with_metadata(
        &f.game.join("Core/1.2/base.big"),
        &["data/maps/official/stock/stock.map"],
        0x19,
        "data/mapmetadata",
    );
    assert!(f.prepare().is_ok());
    f.target.map_crc = "1A".into();
    big_with_metadata(
        &f.game.join("Meta/1.2/base.big"),
        &["data/maps/official/stock/stock.map"],
        0x1a,
        "data/mapmetadata",
    );
    assert!(f.prepare().is_err());
}

#[test]
fn reused_map_path_requires_the_replays_compiled_compatibility_value() {
    let mut f = Fixture::new();
    f.cached("24g");
    f.target.map_crc = "1A".into();
    assert!(f
        .prepare()
        .err()
        .unwrap()
        .to_string()
        .contains("not available locally"));
    let stage = tempfile::tempdir_in(f._temporary.path()).unwrap();
    big_with_crc(
        &stage.path().join("R24g1v1Maps.big"),
        &[&Fixture::asset("24g")],
        0x1a,
    );
    big(
        &stage.path().join("102Scripts.big"),
        &["data/scripts/scripts.lua"],
    );
    let cancel = Cancellation::default();
    let pack = content::publish(
        stage.path(),
        &f.cache,
        "R24g",
        "https://example.test/hotfix",
        "verified-hash",
        &Control {
            cancelled: &cancel,
            progress: &|_| {},
        },
    )
    .unwrap();
    let prepared = f.prepare().unwrap();
    assert!(prepared
        .archives
        .iter()
        .all(|p| p.parent() == Some(pack.as_path())));
}

#[test]
fn unversioned_community_map_uses_exact_path_and_compiled_metadata() {
    let mut f = Fixture::new();
    f.target.map_path = "281data/maps/official/map 1.02+ edition".into();
    let stage = tempfile::tempdir_in(f._temporary.path()).unwrap();
    big(
        &stage.path().join("102plusmaps.big"),
        &["data/maps/official/map 1.02+ edition/map 1.02+ edition.map"],
    );
    big(
        &stage.path().join("102Scripts.big"),
        &["data/scripts/scripts.lua"],
    );
    let cancel = Cancellation::default();
    content::publish(
        stage.path(),
        &f.cache,
        "R12",
        "https://example.test/original",
        "verified-hash",
        &Control {
            cancelled: &cancel,
            progress: &|_| {},
        },
    )
    .unwrap();
    assert!(f.prepare().is_ok());
    f.target.map_crc = "18".into();
    assert!(f.prepare().is_err());
}

#[test]
fn a_shadowing_map_with_different_compatibility_cannot_be_prepared() {
    let f = Fixture::new();
    let stage = tempfile::tempdir_in(f._temporary.path()).unwrap();
    for (name, crc) in [("a.big", 0x18), ("z.big", 0x19)] {
        big_with_crc(&stage.path().join(name), &[&Fixture::asset("24g")], crc);
    }
    big(
        &stage.path().join("102Scripts.big"),
        &["data/scripts/scripts.lua"],
    );
    let cancel = Cancellation::default();
    content::publish(
        stage.path(),
        &f.cache,
        "R24g",
        "https://example.test/pack",
        "fixture-hash",
        &Control {
            cancelled: &cancel,
            progress: &|_| {},
        },
    )
    .unwrap();
    assert!(f.prepare().is_err());
}

#[test]
fn imported_pack_requires_exact_map_and_digest_and_prepares_offline() {
    use std::io::Write;
    use zip::{write::SimpleFileOptions, ZipWriter};

    let f = Fixture::new();
    let payload = f._temporary.path().join("payload");
    let package = f._temporary.path().join("original.zip");
    big(&payload.join("R24g1v1Maps.big"), &[&Fixture::asset("24g")]);
    big(
        &payload.join("102Scripts.big"),
        &["data/scripts/scripts.lua"],
    );
    let mut zip = ZipWriter::new(fs::File::create(&package).unwrap());
    for name in ["R24g1v1Maps.big", "102Scripts.big"] {
        zip.start_file(name, SimpleFileOptions::default()).unwrap();
        zip.write_all(&fs::read(payload.join(name)).unwrap())
            .unwrap();
    }
    zip.finish().unwrap();
    let bytes = fs::read(&package).unwrap();
    let hash = hex::encode(Sha256::digest(&bytes));
    let cancel = Cancellation::default();
    let control = Control {
        cancelled: &cancel,
        progress: &|_| {},
    };
    let source = "https://example.test/original.zip";
    assert!(import_package(
        &f.target,
        &f.cache,
        &package,
        source,
        &"0".repeat(64),
        &control
    )
    .unwrap_err()
    .to_string()
    .contains("SHA-256"));
    let mut other = f.target.clone();
    other.map_path = "283data/maps/official/other 1.02+__24g".into();
    assert!(
        import_package(&other, &f.cache, &package, source, &hash, &control)
            .unwrap_err()
            .to_string()
            .contains("exact map asset")
    );
    assert!(!f.cache.join("packages").exists());
    assert!(import_package(
        &f.target,
        &f.cache,
        &package,
        "https://example.test/pack?access_token=secret",
        &hash,
        &control
    )
    .is_err());
    let directory = import_package(&f.target, &f.cache, &package, source, &hash, &control).unwrap();
    assert!(directory.join("manifest.json").is_file());
    assert_eq!(fs::read(&package).unwrap(), bytes);
    assert_eq!(fs::read_dir(f.cache.join("staging")).unwrap().count(), 0);
    let prepared = f.prepare().unwrap();
    assert_eq!(prepared.archives.len(), 2);
}

#[test]
fn r16_replay_can_prepare_from_the_verified_command_post_beta_label() {
    let mut f = Fixture::new();
    f.target.map_path = "283data/maps/official/map 1.02+__16".into();
    let report = inspect(&f.target, Some(&f.game), &f.cache);
    assert!(report.can_play, "{}", report.message);
    assert_eq!(report.required_revision.as_deref(), Some("R16"));
    assert!(!f.cache.exists());
}

#[test]
fn incomplete_historical_cache_does_not_hide_companion_maps() {
    let f = Fixture::new();
    let source = crate::playback::sources::command_post("R16", selection::PackKind::Duel)
        .remove(0)
        .source;
    let cancellation = Cancellation::default();
    let control = Control {
        cancelled: &cancellation,
        progress: &|_| {},
    };
    let base = Fixture::asset("16");
    let companion = "data/maps/official/companion 1.02+__16/companion 1.02+__16.map";
    for complete in [false, true] {
        let pending = tempfile::tempdir_in(f._temporary.path()).unwrap();
        big(&pending.path().join("102plusmaps.big"), &[&base]);
        big(
            &pending.path().join("102Scripts.big"),
            &["data/scripts/scripts.lua"],
        );
        if complete {
            big(&pending.path().join("102plusmapsA.big"), &[companion]);
        }
        content::publish(
            pending.path(),
            &f.cache,
            "R16",
            &source,
            "package-hash",
            &control,
        )
        .unwrap();
        assert!(!content::contains_other_map(&f.cache, &source, "R16", companion).unwrap());
        assert_eq!(
            content::contains_other_map(&f.cache, &source, "R16", "missing.map").unwrap(),
            complete
        );
        assert_eq!(
            content::cached(&f.cache, companion, 0x19, true, &control)
                .unwrap()
                .is_some(),
            complete
        );
    }
}

#[test]
fn disabled_local_pack_uses_a_private_config_and_preserves_user_selection() {
    let f = Fixture::new();
    f.local("24g", true);
    f.local("24j", true);
    let user_sku = fs::read(f.game.join("CNC3EP1_english_1.2.SkuDef")).unwrap();
    let user_config = fs::read(f.game.join("Patch103/config.txt")).unwrap();
    let prepared = f.prepare().unwrap();
    let config_path = PathBuf::from(&prepared.plan.args[3]);
    let text = fs::read_to_string(&config_path).unwrap();
    assert!(text.contains("R24g1v1Maps.big") && text.contains("R24gScripts.big"));
    assert!(!text.contains("R24j") && !config_path.starts_with(&f.game));
    assert_eq!(
        prepared.plan.working_directory,
        dunce::canonicalize(&f.game).unwrap()
    );
    assert!(prepared.plan.args[1].ends_with("Replay é #1.KWReplay"));
    assert_eq!(
        fs::read(f.game.join("CNC3EP1_english_1.2.SkuDef")).unwrap(),
        user_sku
    );
    assert_eq!(
        fs::read(f.game.join("Patch103/config.txt")).unwrap(),
        user_config
    );
    // Holding the preparation owns the config; releasing it cleans only that session.
    assert!(config_path.is_file());
    drop(prepared);
    assert!(!config_path.exists());
    assert!(f.game.join("Patch103/R24g1v1Maps.big").is_file());
}

#[test]
fn cached_revisions_coexist_and_replay_offline_with_their_own_scripts() {
    let mut f = Fixture::new();
    let older = f.cached("24g");
    let newer = f.cached("24j");
    assert!(content::has_asset(&older, &Fixture::asset("24g")).unwrap());
    assert!(!content::has_asset(&newer, &Fixture::asset("24g")).unwrap());
    let prepared = f.prepare().unwrap();
    assert!(prepared.archives.iter().all(|p| p.starts_with(&older)));
    drop(prepared);
    f.target.map_path = "283data/maps/official/map 1.02+__24j".into();
    let prepared = f.prepare().unwrap();
    assert!(prepared.archives.iter().all(|p| p.starts_with(&newer)));
    assert!(older.is_dir() && newer.is_dir());
}

#[test]
fn cached_map_identity_guides_downloads_without_substituting_other_revisions() {
    use super::super::selection::PackKind;
    let f = Fixture::new();
    f.cached_pack("24j", "2v2");
    let game = config::base(&f.game, f.target.version).unwrap();
    let cancellation = Cancellation::default();
    let control = Control {
        cancelled: &cancellation,
        progress: &|_| {},
    };
    let asset = Fixture::asset("24g");
    let known = content::pack_hints(&game, &f.cache, &asset, &control).unwrap();
    assert_eq!(known, [PackKind::TwoVsTwo]);
    // A two-player match on this known four-player map must not select 1v1.
    assert!(selection::pages("R24g", &known, f.target.n_players)[0].ends_with("r24-2vs2-map-pack/"));
    assert!(f.prepare().is_err()); // R24j content cannot satisfy R24g playback.
    f.cached("24g");
    let known = content::pack_hints(&game, &f.cache, &asset, &control).unwrap();
    assert_eq!(known, [PackKind::Duel, PackKind::TwoVsTwo]); // Exact revision first.
    let unrelated = asset.replace("map 1.02+", "map redux 1.02+");
    assert!(content::pack_hints(&game, &f.cache, &unrelated, &control)
        .unwrap()
        .is_empty());
}

#[test]
fn installed_map_indexes_guide_missing_revisions_and_ignore_thumbnails() {
    use super::super::selection::PackKind;
    let f = Fixture::new();
    let game = config::base(&f.game, f.target.version).unwrap();
    big(
        &f.game.join("Patch103/R23z2v2Maps.big"),
        &[&Fixture::asset("23z")],
    );
    big(
        &f.game.join("Patch103/R24k1v1Maps.big"),
        &[&Fixture::asset("24k").replace(".map", ".tga")],
    );
    let cancellation = Cancellation::default();
    let control = Control {
        cancelled: &cancellation,
        progress: &|_| {},
    };
    assert_eq!(
        content::pack_hints(&game, &f.cache, &Fixture::asset("24g"), &control).unwrap(),
        [PackKind::TwoVsTwo]
    );
    assert!(f.prepare().is_err());
    cancellation.cancel();
    assert!(content::pack_hints(&game, &f.cache, &Fixture::asset("24g"), &control).is_err());
}

#[test]
fn unversioned_installed_scripts_do_not_establish_compatibility() {
    let f = Fixture::new();
    f.local("24g", false);
    assert!(f.prepare().is_err());
    let report = inspect(&f.target, Some(&f.game), &f.cache);
    assert!(report.can_play); // Play can obtain matching dependencies.
    assert!(report.message.contains("prepared when you press Play"));
}

#[test]
fn corrupt_cache_and_incomplete_staging_are_not_usable() {
    let f = Fixture::new();
    let cached = f.cached("24g");
    let script = cached.join("102Scripts.big");
    let mut bytes = fs::read(&script).unwrap();
    *bytes.last_mut().unwrap() ^= 1; // Same length, different contents.
    fs::write(script, bytes).unwrap();
    assert!(f.prepare().is_err());
    big(
        &f.cache.join("staging/download-test/R24g1v1Maps.big"),
        &[&Fixture::asset("24g")],
    );
    big(
        &f.cache.join("staging/download-test/102Scripts.big"),
        &["data/scripts/scripts.lua"],
    );
    assert!(f.prepare().is_err());
}

#[test]
fn cancellation_before_or_during_preparation_never_leaves_a_launchable_session() {
    let f = Fixture::new();
    f.local("24g", true);
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert!(prepare(
        &f.target,
        &f.game,
        &f.cache,
        false,
        &Control {
            cancelled: &cancelled,
            progress: &|_| {}
        }
    )
    .is_err());
    let late = Cancellation::default();
    let progress = |event: Progress| {
        if event.phase == "preparing" {
            late.cancel();
        }
    };
    assert!(prepare(
        &f.target,
        &f.game,
        &f.cache,
        false,
        &Control {
            cancelled: &late,
            progress: &progress
        }
    )
    .is_err());
    assert_eq!(fs::read_dir(f.cache.join("sessions")).unwrap().count(), 0);
}

#[test]
fn unknown_map_and_missing_engine_fail_without_downloading() {
    let mut f = Fixture::new();
    f.target.map_path = "userdata/maps/my-custom-map".into();
    assert!(!inspect(&f.target, Some(&f.game), &f.cache).can_play);
    f.target.map_path = "283data/maps/official/map 1.02+__90a".into();
    assert!(!inspect(&f.target, Some(&f.game), &f.cache).can_play);
    f.target.version = [1, 1, 0, 0];
    assert!(!inspect(&f.target, Some(&f.game), &f.cache).can_play);
    assert!(!f.cache.exists());
}

#[test]
fn preserved_cli_session_is_recovered_without_removing_unowned_content() {
    let f = Fixture::new();
    f.local("24g", true);
    let plan = f.prepare().unwrap().keep();
    let path = PathBuf::from(&plan.args[3]);
    assert!(path.is_file());
    fs::create_dir_all(f.cache.join("sessions/replay-user-folder")).unwrap();
    fs::write(f.cache.join("sessions/replay-user-folder/keep.txt"), "user").unwrap();
    cleanup_sessions(&f.cache).unwrap();
    assert!(!path.exists());
    assert!(f
        .cache
        .join("sessions/replay-user-folder/keep.txt")
        .is_file());
    assert!(f.game.join("Patch103/R24g1v1Maps.big").is_file());
}

#[test]
fn changed_replay_is_rejected_before_content_preparation() {
    let f = Fixture::new();
    fs::write(f.target.file_path.as_ref().unwrap(), "different replay").unwrap();
    assert!(f
        .prepare()
        .err()
        .unwrap()
        .to_string()
        .contains("changed since import"));
    assert!(!f.cache.exists());
}
