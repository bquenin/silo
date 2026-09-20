"""Render the public map download catalogue. No network access or credentials.

Usage: python tools/render_map_sources.py
The machine-readable source is embedded in Tacitus at build time.
"""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def render(data):
    cp = [(version, link) for version in data['versions'] for link in version['links']]
    web = data.get('website_links', [])
    manual = data.get('manual_sources', [])
    links = [link for _, link in cp] + web + manual
    unique = {link['url'] for link in links}
    verified = {link['url'] for link in links if link['status'] == 'zip_header_verified'}
    pack_names = {'102plusmaps': '1v1', '102plusmaps2': '2v2',
                  '102plusmaps3': '4v4', '102pluslegacymaps': 'Legacy',
                  '102plusmapsa': 'Predatore 1', '102plusmaps2a': 'Predatore 2',
                  '102plusmaps3a': 'Predatore 3', '102plusmaps4': 'Pack 4'}
    status_names = {'zip_header_verified': 'ZIP header verified', 'not_zip': 'No ZIP returned',
                    'http_404': 'HTTP 404', 'TimeoutError': 'Timed out', 'unchecked': 'Unchecked',
                    'zip_index_verified': 'ZIP headers and indexes verified'}
    def status(link):
        return status_names.get(link['status'], link['status'])
    lines = [
        '# Historical map pack sources', '',
        f"Checked {data['checked_at']}. **{len(unique)} distinct source links**, including "
        f"**{len(verified)} public ZIP downloads whose headers were verified**. "
        f"Command Post was queried for {len(data['versions'])} registered pack versions.", '',
        'A header check confirms that a public URL returned ZIP bytes; it does not validate the '
        'whole archive or prove replay compatibility. Unavailable links remain in the catalogue '
        'as research leads. Login pages and transient failures are not proof that a pack no longer exists.', '',
        '[Machine-readable catalogue](../src-tauri/resources/map-pack-sources.json). '
        'This file is generated with `python tools/render_map_sources.py`.', '',
        '## Source order in Tacitus', '',
        'Tacitus checks installed and cached content first. For each likely map category, it '
        'tries verified public links from Command Post before the exact-version list on '
        'kaneswrath.com. A failed download or extraction advances to the fallback. It never '
        'substitutes another revision. Test releases and alternate pack families are recorded '
        'here but excluded from automatic selection.', '',
        'The catalogue embeds only shareable URLs. Downloading these files does not require '
        'a Command Post account. The discovery registry itself uses a Command Post session; '
        'credentials and session-bound download URLs are excluded.', '',
        'Metadata provenance: [Command Post public metadata ZIP]('
        + data['metadata_url'] + ') and the Command Post `fetch_files.php` registry, queried '
        'by exact `metapack_name` and `meta_version_id`. The version identifier also supplies '
        'the map archive name: for example R20e uses `R201v1Maps.big`, while R21h uses '
        '`R21g1v1Maps.big`. Internal asset paths still determine the exact map revision.', '',
        'Installer support covers ZIPs containing BIG files, Unicode non-solid DEFLATE NSIS, '
        'and Unicode chunked LZMA NSISBI. Other installer layouts fail without being executed. '
        'Not every historical pack listed here has been fully extracted or replay-tested.', '',
        '## R20e', '',
        '| Pack | Command Post link | Public ZIP size |',
        '| --- | --- | ---: |',
    ]
    for version, link in cp:
        if version['revision'] == 'R20e' and version['set'] in ('102plusmaps', '102plusmaps2', '102plusmaps3'):
            size = f"{link['bytes']:,} bytes" if link.get('bytes') else 'Unknown'
            lines.append(f"| {pack_names[version['set']]} | [Download]({link['url']}) | {size} |")
    lines += ['', 'The 1v1 and 2v2 packages were fully downloaded and extracted as data. '
              'Tacitus prepared catalogue replays 405 (Tournament Highlands) and 443 '
              '(Redzone Rampage) using their exact R20e assets and matching scripts. '
              'The packs contain 69 and 49 map assets respectively. These checks prepared '
              'launch configurations without starting the game.', '',
              'The 1v1 ZIP SHA-256 is '
              '`7c0ab9ddfd58cd5b44b134a19eea01ff6b3fa90d2e232dab033d46edd1a6147b`.', '',
              '## Command Post links', '',
              '| Revision | Pack | Link | Check |', '| --- | --- | --- | --- |']
    for version, link in cp:
        label = version['version'] + (' (test)' if version['test'] else '')
        pack = pack_names.get(version['set'], version['set'])
        if version['kind'] == 'combined':
            pack = 'All-in-one'
        lines.append(f"| {label} | {pack} | [Download]({link['url']}) | {status(link)} |")
    lines += ['', '## Website mirrors', '',
              '| Package | Download | Version list | Check |', '| --- | --- | --- | --- |']
    for link in web:
        lines.append(f"| {link['name']} | [Download]({link['url']}) | [Source]({link['page']}) | {status(link)} |")
    lines += ['', '## Additional sources', '']
    for link in manual:
        lines.append(f"- [{link['name']}]({link['url']}): {link['notes']} ({status(link)}).")
    lines += ['', 'The R20 MEGA folder was linked by [CNCSeries]('
              'https://cncseries.ru/kw-patch-1-02/). Its three ZIPs were checked using public '
              'file metadata, decrypted byte ranges, and ZIP central directories. They contain '
              'the R20e 1v1, 2v2 and 4v4 installers. MEGA is a manual fallback; Tacitus currently '
              'downloads R20e through the Command Post links above.', '',
              '## Leads that did not yield public downloads', '',
              '- [Masterleaf R19 MEGA folder](https://mega.nz/folder/lHkkUDZb#S9s1jiTvxti-ZA3Gmtjlbg): public API returned unavailable.',
              '- [Masterleaf R18 Drive folder](https://drive.google.com/drive/folders/1BGQolbVOsNtRds4b4JQ37_Ph6yb3HwlZ): HTTP 404.',
              '- Shatabrick R12d links: [pack 1](https://www.shatabrick.net/downloads/102plusmp_d1.php), '
              '[pack 2](https://www.shatabrick.net/downloads/102plusmp_d2.php), '
              '[pack 3](https://www.shatabrick.net/downloads/102plusmp_d3.php). Requests timed out.',
              '- Archived R20 pages were recovered through the Wayback Machine, but did not '
              'expose additional downloadable ZIPs.', '']
    return '\n'.join(lines)


if __name__ == '__main__':
    data = json.loads((ROOT / 'src-tauri/resources/map-pack-sources.json').read_text(encoding='utf8'))
    (ROOT / 'docs/map-pack-sources.md').write_text(render(data), encoding='utf8')
    print('Wrote docs/map-pack-sources.md')
