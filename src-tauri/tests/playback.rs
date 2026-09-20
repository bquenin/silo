//! Portable fixtures for the map/configuration decisions. These never start
//! the game, modify an installed pack, or require a local replay collection.
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use tacitus_lib::playback::{self, ReplayTarget, Report, Status};

struct Fixture {
    root: PathBuf,
    target: ReplayTarget,
}

impl Fixture {
    fn new() -> Self {
        static SEQUENCE: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tacitus-playback-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("RetailExe/1.2")).unwrap();
        fs::create_dir_all(root.join("Patch103")).unwrap();
        fs::write(
            root.join("RetailExe/1.2/cnc3ep1.dat"),
            b"fixture, never executed",
        )
        .unwrap();
        fs::write(root.join("test.SkuDef"), "set-exe RetailExe\\1.2\\cnc3ep1.dat\nadd-config Patch103\\config.txt\nadd-search-path big:\n").unwrap();
        fs::write(
            root.join("Patch103/config.txt"),
            "add-big maps.big\nadd-big 102Scripts.big\n",
        )
        .unwrap();
        fs::write(root.join("Replay #1.KWReplay"), b"fixture replay").unwrap();
        big(
            &root.join("Patch103/102Scripts.big"),
            &["data/scripts/scripts.lua"],
        );
        Self {
            target: ReplayTarget {
                id: 1,
                file_path: Some(
                    root.join("Replay #1.KWReplay")
                        .to_string_lossy()
                        .into_owned(),
                ),
                file_hash: "not-the-real-hash".into(),
                map_name: "[R24] Abandoned Subway".into(),
                map_path: "283data/maps/official/abandoned subway 1.02+__24g".into(),
                map_crc: "19".into(),
                n_players: 2,
                version: [1, 2, 0, 0],
            },
            root,
        }
    }

    fn sku(&self) -> PathBuf {
        self.root.join("test.SkuDef")
    }
    fn check(&self) -> Report {
        playback::check(&self.target, Some(&self.sku())).unwrap()
    }
    fn map(&self, archive: &str, revision: &str) {
        big(&self.root.join("Patch103").join(archive), &[&format!(
            "Data\\Maps\\official\\Abandoned Subway 1.02+__{revision}\\Abandoned Subway 1.02+__{revision}.map")]);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        // All deletion is restricted to the private temporary fixture root.
        assert_eq!(self.root.parent(), Some(std::env::temp_dir().as_path()));
        assert!(self
            .root
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("tacitus-playback-"));
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn big(path: &Path, names: &[&str]) {
    let end = 16 + names.iter().map(|n| 8 + n.len() + 1).sum::<usize>();
    let mut data = b"BIGF".to_vec();
    data.extend_from_slice(&((end + names.len()) as u32).to_be_bytes());
    data.extend_from_slice(&(names.len() as u32).to_be_bytes());
    data.extend_from_slice(&(end as u32).to_be_bytes());
    for (index, name) in names.iter().enumerate() {
        data.extend_from_slice(&((end + index) as u32).to_be_bytes());
        data.extend_from_slice(&1u32.to_be_bytes());
        data.extend_from_slice(name.as_bytes());
        data.push(0);
    }
    data.resize(end + names.len(), 1);
    fs::write(path, data).unwrap();
}

#[test]
fn exact_revision_is_required_even_when_another_revision_is_enabled() {
    let f = Fixture::new();
    f.map("maps.big", "24j");
    let report = f.check();
    assert_eq!(report.status, Status::MapMissing);
    assert_eq!(report.required_revision.as_deref(), Some("R24g"));
    assert!(report.launch_plan.is_none());
    f.map("maps.big", "24g");
    assert_eq!(f.check().status, Status::Ready);
}

#[test]
fn installed_map_must_be_mounted_by_the_selected_configuration() {
    let f = Fixture::new();
    f.map("disabled.big", "24g");
    let report = f.check();
    assert_eq!(report.status, Status::MapDisabled);
    assert_eq!(report.providers.len(), 1);
    assert!(!report.providers[0].enabled);
    fs::write(
        f.root.join("Patch103/config.txt"),
        "add-big disabled.big\nadd-big 102Scripts.big",
    )
    .unwrap();
    assert_eq!(f.check().status, Status::Ready);
}

#[test]
fn thumbnail_and_similar_suffix_do_not_satisfy_map_requirement() {
    let mut f = Fixture::new();
    f.target.map_path = "281data/maps/official/abandoned subway 1.02+__24".into();
    f.map("maps.big", "24g");
    big(
        &f.root.join("Patch103/preview.big"),
        &["data/maps/official/abandoned subway 1.02+__24/abandoned subway 1.02+__24.tga"],
    );
    assert_eq!(f.check().status, Status::MapMissing);
}

#[test]
fn nested_relative_configs_and_big4_are_supported() {
    let f = Fixture::new();
    f.map("maps.big", "24g");
    fs::create_dir(f.root.join("Patch103/nested")).unwrap();
    fs::write(
        f.root.join("Patch103/config.txt"),
        "\u{feff}# content\nadd-config \"nested\\maps.txt\"\nadd-big 102Scripts.big",
    )
    .unwrap();
    fs::write(
        f.root.join("Patch103/nested/maps.txt"),
        "add-big \"..\\maps.big\"\n",
    )
    .unwrap();
    let path = f.root.join("Patch103/maps.big");
    let mut data = fs::read(&path).unwrap();
    data[..4].copy_from_slice(b"BIG4");
    fs::write(path, data).unwrap();
    assert_eq!(f.check().status, Status::Ready);
}

#[test]
fn include_cycles_and_unsupported_directives_are_not_silently_ignored() {
    let f = Fixture::new();
    fs::write(f.root.join("Patch103/config.txt"), "add-config config.txt").unwrap();
    let report = f.check();
    assert_eq!(report.status, Status::Unknown);
    assert!(report.message.contains("cycle"));
    fs::write(
        f.root.join("Patch103/config.txt"),
        "unknown-command maps.big",
    )
    .unwrap();
    assert!(f.check().message.contains("Unsupported"));
}

#[test]
fn malformed_active_archive_blocks_launch_without_allocating_its_claimed_size() {
    let f = Fixture::new();
    let path = f.root.join("Patch103/maps.big");
    let mut header = b"BIGF".to_vec();
    header.extend_from_slice(&[255; 12]);
    fs::write(&path, header).unwrap();
    assert_eq!(f.check().status, Status::Unknown);
    f.map("maps.big", "24g");
    let mut data = fs::read(&path).unwrap();
    data[16..20].copy_from_slice(&u32::MAX.to_be_bytes());
    fs::write(path, data).unwrap();
    assert_eq!(f.check().status, Status::Unknown);
}

#[test]
fn engine_and_unversioned_custom_content_are_separate_requirements() {
    let mut f = Fixture::new();
    f.map("maps.big", "24g");
    f.target.version = [1, 1, 0, 0];
    assert_eq!(f.check().status, Status::EngineMismatch);
    f.target.version = [1, 2, 0, 0];
    f.target.map_path = "281data/maps/official/abandoned subway 1.02+".into();
    assert_eq!(f.check().status, Status::Unknown);
    f.target.map_path = "userdata/maps/custom".into();
    assert_eq!(f.check().status, Status::Unknown);
}

#[test]
fn duplicate_active_map_sources_are_reported_as_ambiguous() {
    let f = Fixture::new();
    f.map("maps.big", "24g");
    f.map("duplicate.big", "24g");
    fs::write(
        f.root.join("Patch103/config.txt"),
        "add-big maps.big\nadd-big duplicate.big\nadd-big 102Scripts.big",
    )
    .unwrap();
    assert_eq!(f.check().status, Status::Unknown);
}

#[test]
fn community_scripts_are_checked_separately_from_the_map() {
    let f = Fixture::new();
    f.map("maps.big", "24g");
    fs::write(f.root.join("Patch103/config.txt"), "add-big maps.big").unwrap();
    let report = f.check();
    assert_eq!(report.status, Status::Unknown);
    assert!(report.message.contains("scripts"));
}

#[test]
fn launch_plan_preserves_paths_and_inherits_the_environment() {
    let f = Fixture::new();
    f.map("maps.big", "24g");
    let report = f.check();
    let plan = report.launch_plan.unwrap();
    let command = playback::launch_command(&plan);
    let args: Vec<_> = command
        .get_args()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    assert_eq!(args.len(), 5);
    assert_eq!(args[0], "-replayGame");
    assert!(args[1].ends_with("Replay #1.KWReplay"));
    assert_eq!(args[2..4], ["-win", "-config"]);
    assert_eq!(
        command.get_current_dir(),
        Some(dunce::canonicalize(&f.root).unwrap().as_path())
    );
    assert_eq!(command.get_envs().count(), 0);
}

#[test]
fn stock_or_unidentified_scripts_cannot_satisfy_a_community_map() {
    let f = Fixture::new();
    f.map("maps.big", "24g");
    fs::create_dir_all(f.root.join("Core/1.0")).unwrap();
    big(
        &f.root.join("Core/1.0/Misc.big"),
        &["data/scripts/scripts.lua"],
    );
    fs::create_dir_all(f.root.join("loose/data/scripts")).unwrap();
    fs::write(
        f.root.join("loose/data/scripts/scripts.lua"),
        b"stock script",
    )
    .unwrap();
    fs::write(f.root.join("test.SkuDef"),
        "set-exe RetailExe/1.2/cnc3ep1.dat\nadd-big Core/1.0/Misc.big\nadd-big Patch103/maps.big\nadd-search-path loose\nadd-search-path big:").unwrap();
    assert_eq!(f.check().status, Status::Unknown);

    // A misleading filename alone also cannot satisfy the requirement.
    big(&f.root.join("Patch103/R24gScripts.big"), &["unrelated.txt"]);
    fs::write(f.root.join("test.SkuDef"),
        "set-exe RetailExe/1.2/cnc3ep1.dat\nadd-big Core/1.0/Misc.big\nadd-big Patch103/maps.big\nadd-big Patch103/R24gScripts.big\nadd-search-path big:").unwrap();
    assert_eq!(f.check().status, Status::Unknown);
    big(
        &f.root.join("Patch103/R24gScripts.big"),
        &["data/scripts/scripts.lua"],
    );
    let report = f.check();
    assert_eq!(report.status, Status::Ready);
    assert!(report
        .warnings
        .iter()
        .any(|w| w.contains("R24gScripts.big")));
}

#[test]
fn missing_configuration_and_replay_have_distinct_statuses() {
    let mut f = Fixture::new();
    assert_eq!(
        playback::check(&f.target, None).unwrap().status,
        Status::NotConfigured
    );
    f.target.file_path = None;
    assert_eq!(f.check().status, Status::ReplayMissing);
}

#[test]
#[cfg(windows)]
fn changed_replay_is_rejected_before_process_creation() {
    let f = Fixture::new();
    f.map("maps.big", "24g");
    let error = playback::launch(&f.target, Some(&f.sku())).unwrap_err();
    assert!(error.to_string().contains("changed since import"));
}
