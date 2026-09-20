//! Pick a likely pack before downloading it. Pack categories describe map
//! capacity, not team assignments: four-player FFA and 2v2 use the same pool.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackKind {
    Duel,
    TwoVsTwo,
    Large,
    Legacy,
    Combined,
    Arcade,
}

impl PackKind {
    pub fn from_page(page: &str) -> Option<Self> {
        match pack_label(page) {
            "1v1 map pack" => Some(Self::Duel),
            "2v2 map pack" => Some(Self::TwoVsTwo),
            "4v4 map pack" => Some(Self::Large),
            "legacy map pack" => Some(Self::Legacy),
            "all-in-one map pack" => Some(Self::Combined),
            "arcade map pack" => Some(Self::Arcade),
            _ => None,
        }
    }

    pub fn from_archive(name: &str) -> Option<Self> {
        let name = name.to_ascii_lowercase();
        if name.starts_with('f') && name.contains("mappack") && name.ends_with(".big") {
            return Some(Self::Arcade);
        }
        match name.as_str() {
            "102plusmaps.big" | "102plusmapsa.big" => return Some(Self::Duel),
            "102plusmaps2.big" | "102plusmaps2a.big" => return Some(Self::TwoVsTwo),
            "102plusmaps3.big" | "102plusmaps3a.big" => return Some(Self::Large),
            _ => {}
        }
        for (suffix, kind) in [
            ("1v1maps.big", Self::Duel),
            ("2v2maps.big", Self::TwoVsTwo),
            ("3v3maps.big", Self::Large),
            ("4v4maps.big", Self::Large),
            ("legacymaps.big", Self::Legacy),
            ("allinonemaps.big", Self::Combined),
            ("allmaps.big", Self::Combined),
            ("maps.big", Self::Combined),
        ] {
            if let Some(revision) = name.strip_suffix(suffix).and_then(|s| s.strip_prefix('r')) {
                if valid_revision(revision) {
                    return Some(kind);
                }
            }
        }
        None
    }
}

fn valid_revision(revision: &str) -> bool {
    let digits = revision.bytes().take_while(u8::is_ascii_digit).count();
    digits > 0
        && (digits == revision.len()
            || (digits + 1 == revision.len() && revision.as_bytes()[digits].is_ascii_alphabetic()))
}

/// A cross-revision lookup key only. Launch still requires the complete,
/// revision-specific asset; thumbnails and similarly named maps do not match.
pub fn map_identity(asset: &str) -> Option<&str> {
    let (directory, file) = asset
        .strip_prefix("data/maps/official/")?
        .rsplit_once('/')?;
    if directory.contains('/') || file.strip_suffix(".map")? != directory {
        return None;
    }
    let (name, revision) = directory.rsplit_once("__")?;
    (!name.is_empty() && valid_revision(revision)).then_some(name)
}

/// Human label for the pack behind a download page, e.g. "1v1 map pack".
pub fn pack_label(page: &str) -> &'static str {
    let slug = page
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    if slug.contains("1vs1") || slug.contains("1v1") {
        "1v1 map pack"
    } else if slug.contains("2vs2") || slug.contains("2v2") {
        "2v2 map pack"
    } else if slug.contains("4vs4") || slug.contains("4v4") {
        "4v4 map pack"
    } else if slug.contains("legacy") {
        "legacy map pack"
    } else if slug.contains("all-in-one") {
        "all-in-one map pack"
    } else if slug == "arcade-map-pack" {
        "arcade map pack"
    } else {
        "map pack"
    }
}

pub fn pages(revision: &str, known: &[PackKind], players: u32) -> Vec<String> {
    use PackKind::*;
    let major: String = revision
        .chars()
        .skip(1)
        .take_while(char::is_ascii_digit)
        .collect();
    if major == "25" {
        return vec!["https://kaneswrath.com/download/r25-all-in-one-map-pack/".into()];
    }
    // A known map-to-pack match outranks match size: two players can choose
    // a four-player map. Other revisions provide hints, never substitutes.
    let mut kinds = Vec::new();
    for kind in known
        .iter()
        .copied()
        .chain(match players {
            1..=2 => Some(Duel),
            3..=4 => Some(TwoVsTwo),
            5..=8 => Some(Large),
            _ => None,
        })
        .chain([Duel, TwoVsTwo, Large, Legacy, Combined, Arcade])
    {
        let known_map = known.contains(&kind);
        if (!known_map && ((kind == Duel && players > 2) || (kind == TwoVsTwo && players > 4)))
            || (kind == Combined && major != "24")
            || kinds.contains(&kind)
        {
            continue;
        }
        kinds.push(kind);
    }
    let mut pages = Vec::new();
    for kind in kinds {
        let slug = match kind {
            Duel => format!("r{major}-1vs1-map-pack"),
            TwoVsTwo => format!("r{major}-2vs2-map-pack"),
            Large => format!(
                "r{major}-{}-map-pack",
                if major == "24" { "4v4" } else { "4vs4" }
            ),
            Legacy => format!("r{major}-legacy-map-pack"),
            Combined => format!("r{major}-all-in-one-map-pack"),
            Arcade => "arcade-map-pack".into(),
        };
        pages.push(format!("https://kaneswrath.com/download/{slug}/"));
        if kind == Legacy && major == "23" {
            pages.push("https://kaneswrath.com/download/legacy-map-pack-r23/".into());
        }
    }
    pages
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn participant_count_selects_pack_capacity_without_trying_smaller_packs() {
        assert!(pages("R23z", &[], 2)[0].ends_with("r23-1vs1-map-pack/"));
        for players in [3, 4] {
            let choices = pages("R23z", &[], players);
            assert!(choices[0].ends_with("r23-2vs2-map-pack/"));
            assert!(!choices.iter().any(|page| page.contains("1vs1")));
        }
        for players in 5..=8 {
            let choices = pages("R24g", &[], players);
            assert!(choices[0].ends_with("r24-4v4-map-pack/"));
            assert!(!choices
                .iter()
                .any(|page| page.contains("1vs1") || page.contains("2vs2")));
        }
        assert!(pages("R22j", &[], 6)[0].ends_with("r22-4vs4-map-pack/"));
    }

    #[test]
    fn known_maps_override_match_size_and_provider_layouts_remain_supported() {
        let choices = pages("R23z", &[PackKind::TwoVsTwo], 2);
        assert!(choices[0].ends_with("r23-2vs2-map-pack/"));
        assert_eq!(
            choices.iter().filter(|page| page.contains("2vs2")).count(),
            1
        );
        let legacy = pages("R23z", &[PackKind::Legacy], 2);
        assert!(legacy[0].ends_with("r23-legacy-map-pack/"));
        assert!(legacy[1].ends_with("legacy-map-pack-r23/"));
        assert_eq!(
            pages("R25i", &[PackKind::Duel], 2),
            ["https://kaneswrath.com/download/r25-all-in-one-map-pack/"]
        );
        assert!(pages("R24g", &[], 0)
            .iter()
            .any(|page| page.contains("all-in-one")));
    }

    #[test]
    fn map_keys_strip_only_revisions_and_ignore_thumbnails_and_similar_names() {
        let asset = "data/maps/official/map 1.02+__24g/map 1.02+__24g.map";
        assert_eq!(map_identity(asset), Some("map 1.02+"));
        assert_eq!(map_identity(&asset.replace(".map", ".tga")), None);
        assert_ne!(
            map_identity(asset),
            map_identity(&asset.replace("map 1.02+", "map redux 1.02+"))
        );
        assert_eq!(
            PackKind::from_archive("R23z2v2Maps.big"),
            Some(PackKind::TwoVsTwo)
        );
        assert_eq!(
            PackKind::from_archive("R24g4v4Maps.big"),
            Some(PackKind::Large)
        );
        assert_eq!(PackKind::from_archive("102Scripts.big"), None);
        assert_eq!(
            PackKind::from_archive("102plusmaps.big"),
            Some(PackKind::Duel)
        );
        assert_eq!(
            PackKind::from_archive("102plusmaps2A.big"),
            Some(PackKind::TwoVsTwo)
        );
        assert_eq!(
            PackKind::from_archive("102plusmaps3.big"),
            Some(PackKind::Large)
        );
        assert_eq!(PackKind::from_archive("random2v2maps.big"), None);
    }
}
