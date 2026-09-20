//! Public download links recorded by Command Post. Credentials are never
//! embedded in the catalogue or needed to use these public mirrors.
use std::sync::OnceLock;

use reqwest::Url;
use serde::Deserialize;

use super::selection::PackKind;

#[derive(Deserialize)]
struct Catalogue {
    versions: Vec<Version>,
}

#[derive(Deserialize)]
struct Version {
    revision: Option<String>,
    kind: Option<PackKind>,
    test: bool,
    archive: Option<String>,
    #[serde(default)]
    additional_archives: Vec<String>,
    links: Vec<Link>,
}

#[derive(Deserialize)]
struct Link {
    url: String,
    download_url: Option<String>,
    status: String,
    disabled: bool,
    login_required: bool,
}

fn catalogue() -> &'static Catalogue {
    static CATALOGUE: OnceLock<Catalogue> = OnceLock::new();
    CATALOGUE.get_or_init(|| {
        serde_json::from_str(include_str!("../../resources/map-pack-sources.json"))
            .expect("The bundled map source catalogue must be valid")
    })
}

fn matches(version: &Version, revision: &str) -> bool {
    !version.test
        && version.kind.is_some()
        && version
            .revision
            .as_deref()
            .is_some_and(|r| r.eq_ignore_ascii_case(revision))
}

fn usable(link: &Link) -> Option<Url> {
    if link.disabled || link.login_required || link.status != "zip_header_verified" {
        return None;
    }
    let url = Url::parse(link.download_url.as_deref()?).ok()?;
    (url.scheme() == "https" && url.username().is_empty() && url.password().is_none())
        .then_some(url)
}

pub fn available(revision: &str) -> bool {
    catalogue().versions.iter().any(|version| {
        matches(version, revision) && version.links.iter().any(|link| usable(link).is_some())
    })
}

/// Some R20e packs still call their archive R201v1Maps.big; R21h uses R21g.
/// Command Post's exact version metadata supplies these aliases. Selection of
/// the replay itself still requires its complete internal asset path.
pub fn map_archive(name: &str, revision: &str) -> bool {
    catalogue().versions.iter().any(|version| {
        matches(version, revision)
            && version
                .archive
                .iter()
                .chain(&version.additional_archives)
                .any(|a| a.eq_ignore_ascii_case(name))
    })
}

/// An older cache may have been published before companion archives were
/// recognized. Its missing map must not suppress a complete download forever.
pub fn complete_map_archives(source: &str, revision: &str, names: &[&str]) -> bool {
    catalogue()
        .versions
        .iter()
        .filter(|version| {
            matches(version, revision) && version.links.iter().any(|l| l.url == source)
        })
        .all(|version| {
            version
                .archive
                .iter()
                .chain(&version.additional_archives)
                .all(|required| names.iter().any(|name| name.eq_ignore_ascii_case(required)))
        })
}

pub struct Candidate {
    pub url: Url,
    pub source: String,
}

pub fn command_post(revision: &str, kind: PackKind) -> Vec<Candidate> {
    catalogue()
        .versions
        .iter()
        .filter(|version| matches(version, revision) && version.kind == Some(kind))
        .flat_map(|version| &version.links)
        .filter_map(|link| {
            Some(Candidate {
                url: usable(link)?,
                source: link.url.clone(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_catalogue_contains_only_shareable_urls() {
        for version in &catalogue().versions {
            for link in &version.links {
                for raw in std::iter::once(&link.url).chain(link.download_url.iter()) {
                    let url = Url::parse(raw).unwrap();
                    assert!(url.username().is_empty() && url.password().is_none());
                    for (key, _) in url.query_pairs() {
                        assert!(![
                            "token",
                            "password",
                            "username",
                            "access_token",
                            "authorization"
                        ]
                        .contains(&key.to_ascii_lowercase().as_str()));
                        assert!(!key.to_ascii_lowercase().starts_with("x-amz-"));
                    }
                }
            }
        }
    }

    #[test]
    fn command_post_sources_preserve_exact_revisions_and_archive_aliases() {
        let sources = command_post("R20e", PackKind::Duel);
        assert!(!sources.is_empty());
        assert_eq!(
            sources[0].url.host_str(),
            Some("drive.usercontent.google.com")
        );
        assert!(sources[0]
            .url
            .query_pairs()
            .any(|(k, v)| k == "id" && v == "1gXufNs20MMYavQlQ-SLZq6lzkoZD8mfg"));
        assert!(map_archive("R201v1Maps.big", "R20e"));
        assert!(!map_archive("R211v1Maps.big", "R20e"));
        assert!(!available("R20e Beta"));
        assert!(command_post("R20e/../../", PackKind::Duel).is_empty());
    }

    #[test]
    fn verified_r16_release_label_matches_its_replay_revision() {
        for (kind, archive) in [
            (PackKind::Duel, "102plusmaps.big"),
            (PackKind::TwoVsTwo, "102plusmaps2.big"),
            (PackKind::Large, "102plusmaps3.big"),
        ] {
            let links = command_post("R16", kind);
            assert_eq!(links.len(), 1);
            assert_eq!(
                links[0].url.host_str(),
                Some("cgf-uploads.fra1.cdn.digitaloceanspaces.com")
            );
            assert!(links[0].url.path().contains("R16%20Beta/"));
            assert!(map_archive(archive, "R16"));
        }
        assert!(available("R16"));
        // Other beta labels have not been established as replay revisions.
        assert!(!available("R15"));
        assert!(!available("R16 Beta"));
        assert!(command_post("R16b", PackKind::Duel).is_empty());
        assert!(map_archive("102plusmapsA.big", "R16"));
        assert!(!map_archive("102plusmaps4.big", "R16"));
    }

    #[test]
    fn r18f_large_pack_excludes_the_mislabeled_r18d_registry_link() {
        let links = command_post("R18f", PackKind::Large);
        assert_eq!(links.len(), 1);
        assert!(links[0].url.path().contains(
            "/R18f/KWCommunityPatch102PlusMaps3_R18f.zip"
        ));
        assert!(!links[0].url.path().contains("R18d"));
        assert!(map_archive("102plusmaps3_18.big", "R18f"));
    }

    #[test]
    fn restricted_broken_and_insecure_links_are_not_automatic_candidates() {
        let mut link = Link {
            url: "https://example.org/pack.zip".into(),
            download_url: Some("https://example.org/pack.zip".into()),
            status: "zip_header_verified".into(),
            disabled: false,
            login_required: false,
        };
        assert!(usable(&link).is_some());
        link.login_required = true;
        assert!(usable(&link).is_none());
        link.login_required = false;
        link.disabled = true;
        assert!(usable(&link).is_none());
        link.disabled = false;
        link.status = "http_404".into();
        assert!(usable(&link).is_none());
        link.status = "zip_header_verified".into();
        link.download_url = Some("http://example.org/pack.zip".into());
        assert!(usable(&link).is_none());
    }
}
