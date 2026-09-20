"""Audit exact map paths, compiled MC values, bundled scripts and cache hashes.

No downloads or game processes. Every cached BIG is hashed once per invocation,
then reused across all catalogue rows. JSON includes unresolved map requirements.
Usage: python tools/audit_replay_content.py --game "C:/.../Kane's Wrath" --output scratch/coverage.json
"""
import argparse
import collections
import concurrent.futures
import hashlib
import json
import os
import re
import sqlite3
import struct
from datetime import datetime, timezone
from pathlib import Path


def big_index(path):
    entries = {}
    with path.open('rb') as file:
        header = file.read(16)
        if header[:4] not in (b'BIGF', b'BIG4'):
            raise ValueError('Not a BIG archive')
        count, end = struct.unpack('>II', header[8:16])
        size = path.stat().st_size
        if not 16 <= end <= min(size, 64 * 1024**2) or count > (end - 16) // 9:
            raise ValueError('Invalid BIG directory')
        data, at = file.read(end - 16), 0
        for _ in range(count):
            offset, length = struct.unpack_from('>II', data, at)
            stop = data.index(0, at + 8)
            if stop - at - 8 > 4096 or not end <= offset <= offset + length <= size:
                raise ValueError('Invalid BIG record')
            name = data[at + 8:stop].decode('latin1').replace('\\', '/').lower()
            at = stop + 1
            if length:
                if name in entries:
                    raise ValueError('Duplicate BIG filename')
                entries[name] = (offset, length)
    return entries


def decode_refpack(data, limit):
    if len(data) < 2 or data[1] != 0xfb or data[0] & 0x3e != 0x10:
        return data
    width = 4 if data[0] & 0x80 else 3
    at = 2 + (width if data[0] & 1 else 0)
    expected = int.from_bytes(data[at:at + width], 'big')
    at += width
    if expected > limit or at > len(data):
        raise ValueError('Invalid expanded metadata size')
    output = bytearray()
    while at < len(data):
        code, at = data[at], at + 1
        length = distance = 0
        if code < 0x80:
            second, at = data[at], at + 1
            literal, length, distance = code & 3, ((code & 28) >> 2) + 3, ((code & 96) << 3) + second + 1
        elif code < 0xc0:
            second, third = data[at:at + 2]
            at += 2
            literal, length, distance = second >> 6, (code & 63) + 4, ((second & 63) << 8) + third + 1
        elif code < 0xe0:
            second, third, fourth = data[at:at + 3]
            at += 3
            literal, length, distance = code & 3, ((code & 12) << 6) + fourth + 5, ((code & 16) << 12) + (second << 8) + third + 1
        else:
            literal = ((code & 31) + 1) << 2 if code < 0xfc else code & 3
        if len(output) + literal + length > expected or at + literal > len(data):
            raise ValueError('Invalid RefPack command bounds')
        output.extend(data[at:at + literal])
        at += literal
        if length and not 0 < distance <= len(output):
            raise ValueError('Invalid RefPack back reference')
        for _ in range(length):
            output.append(output[-distance])
        if code >= 0xfc:
            if len(output) != expected or at != len(data):
                raise ValueError('Incomplete or trailing RefPack data')
            return output
    raise ValueError('Missing RefPack terminator')


def compiled_maps(path, entries, stock=False):
    result = {}
    with path.open('rb') as file:
        def read(name, limit):
            offset, length = entries[name]
            if length > limit:
                raise ValueError('Oversized map metadata')
            file.seek(offset)
            return decode_refpack(file.read(length), limit)

        for name in entries:
            if not ((stock and name == 'data/mapmetadata.bin') or
                    (not stock and name.startswith('data/additionalmaps/mapmetadata') and name.endswith('.bin'))):
                continue
            binary = read(name, 16 * 1024**2)
            manifest = read(name[:-4] + '.manifest', 1024**2)
            word = lambda data, at: struct.unpack_from('<I', data, at)[0]
            if (len(manifest) < 92 or manifest[:4] != b'\0\1\5\0'
                    or not 1 <= word(manifest, 12) <= 4096
                    or len(manifest) < 48 + word(manifest, 12) * 44
                    or word(binary, 0) != word(manifest, 4)
                    or len(binary) != 4 + word(manifest, 16)
                    ):
                raise ValueError('Unsupported compiled MapMetaData stream')
            cursor = 4
            for item in range(word(manifest, 12)):
                record = 48 + item * 44
                size = word(manifest, record + 32)
                if cursor + size > len(binary):
                    raise ValueError('Compiled asset outside stream')
                data = binary[cursor:cursor + size]
                cursor += size
                if not size or word(manifest, record) != 0x5f969146:
                    continue
                if word(manifest, record + 8) != 1059476008:
                    raise ValueError('Unsupported MapMetaData type version')
                count, start = struct.unpack_from('<II', data, 4)
                if not (0 < count <= 4096 and 12 <= start and start + count * 60 <= len(data)):
                    raise ValueError('Invalid map metadata records')
                for index in range(count):
                    players, crc, length, offset = struct.unpack_from('<IIII', data, start + index * 60 + 20)
                    if not (0 < length <= 4096 and start + count * 60 <= offset <= offset + length <= len(data)):
                        raise ValueError('Invalid map metadata string')
                    asset = data[offset:offset + length].decode('utf8').replace('\\', '/').lower()
                    if not (asset.startswith('data/maps/official/') and asset.endswith('.map')) or '\0' in asset:
                        raise ValueError('Invalid map metadata path')
                    if stock or asset in entries:
                        if asset in result and result[asset]['crc'] != crc:
                            raise ValueError('Conflicting map compatibility values')
                        result[asset] = {'crc': crc, 'capacity': players}
            if cursor != len(binary):
                raise ValueError('Unaccounted compiled asset data')
    return result


def hash_file(path):
    with path.open('rb') as file:
        return hashlib.file_digest(file, 'sha256').hexdigest()


def custom_map(root, asset, crc):
    relative = asset.removeprefix('data/maps/internal/').split('/')
    if (len(relative) != 2 or relative[0] in ('', '.', '..')
            or relative[1] != relative[0] + '.map'
            or any(c in relative[0] for c in '\\/:*?"<>|')):
        return None
    path = root.joinpath(*relative)
    if not path.is_file() or path.stat().st_size > 512 * 1024**2:
        return None
    if not path.resolve().is_relative_to(root.resolve()):
        return None
    checksum = 0
    with path.open('rb') as file:
        while block := file.read(128 * 1024):
            for byte in block:
                checksum = (((checksum << 1) | (checksum >> 31)) + byte) & 0xffffffff
    return str(path) if checksum == crc else None


def verify_pack(manifest, base_sources):
    pack = json.loads(manifest.read_text(encoding='utf8'))
    maps, scripts, copies = {}, False, collections.defaultdict(list)
    for archive in pack['archives']:
        name = archive['name']
        if not name or any(c in name for c in '/\\:'):
            raise ValueError('Invalid cached filename')
        path = manifest.parent / name
        if path.is_symlink() or not path.is_file() or path.stat().st_size != archive['size']:
            raise ValueError('Missing or changed cached archive')
        if hash_file(path) != archive['sha256']:
            raise ValueError('Cached archive hash mismatch')
        entries = big_index(path)
        if list(entries) != archive['entries']:
            raise ValueError('Cached archive index mismatch')
        if name.lower() in ('102scripts.big', pack['revision'].lower() + 'scripts.big'):
            scripts |= 'data/scripts/scripts.lua' in entries
        compiled = compiled_maps(path, entries)
        for asset in entries:
            if asset.endswith('.map'):
                copies[asset].append(compiled.get(asset, {}).get('crc'))
        for asset, detail in compiled.items():
            if asset in maps and maps[asset]['crc'] != detail['crc']:
                raise ValueError('Conflicting map metadata within package')
            maps[asset] = {**detail, 'archive': name}
    maps = {asset: detail for asset, detail in maps.items()
            if all(crc == detail['crc'] for crc in copies[asset])}
    uses_base = (pack['revision'], pack['source'], pack['package_sha256']) in base_sources
    return {'directory': str(manifest.parent), 'revision': pack['revision'], 'source': pack['source'],
            'package_sha256': pack['package_sha256'], 'scripts': scripts or uses_base,
            'script_source': 'base_game' if uses_base and not scripts else 'package', 'maps': maps}


def base_assets(game, version):
    result, seen = set(), set()
    metadata = None
    def visit(config):
        nonlocal metadata
        config = config.resolve()
        if config in seen:
            return
        seen.add(config)
        if len(seen) > 1024:
            raise ValueError('Too many base configuration files')
        for line in config.read_text(encoding='utf-8-sig').splitlines():
            parts = line.strip().split(maxsplit=1)
            if len(parts) != 2:
                continue
            directive, value = parts
            child = config.parent / value.strip('"').replace('\\', '/')
            if directive.lower() == 'add-config' and child.is_file():
                visit(child)
            elif directive.lower() == 'add-big' and child.is_file():
                entries = big_index(child)
                result.update(name for name in entries if name.endswith('.map'))
                if metadata is None and 'data/mapmetadata.bin' in entries:
                    metadata = compiled_maps(child, entries, stock=True)
    if version[0] != 1 or version[1] > 2 or version[2:] != (0, 0):
        raise ValueError('Unsupported engine version')
    engine = f'{version[0]}.{version[1]}'
    if not (game / 'RetailExe' / engine / 'cnc3ep1.dat').is_file():
        raise ValueError('Missing game engine')
    for layer in ('Core', 'Meta'):
        visit(game / layer / engine / 'config.txt')
    return {asset: (metadata or {}).get(asset, {}).get('crc') for asset in result}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--game', type=Path, required=True)
    parser.add_argument('--db', type=Path, default=Path(os.environ['APPDATA']) / 'tacitus/catalogue.sqlite3')
    parser.add_argument('--cache', type=Path, default=Path(os.environ['LOCALAPPDATA']) / 'tacitus/playback')
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--maps', type=Path, default=Path(os.environ['APPDATA']) / "Command & Conquer 3 Kane's Wrath/Maps")
    args = parser.parse_args()
    source_data = json.loads((Path(__file__).resolve().parents[1] / 'src-tauri/resources/map-pack-sources.json').read_text(encoding='utf8'))
    base_sources = {(v['revision'], link['url'], v.get('package_sha256')) for v in source_data['versions']
                    if v.get('script_dependency') == 'base_game' for link in v['links']}
    manifests = sorted((args.cache / 'packages').glob('*/manifest.json'))
    assets, errors, packages = collections.defaultdict(list), [], []
    def verify(manifest):
        try:
            return verify_pack(manifest, base_sources), None
        except Exception as error:
            return None, {'manifest': str(manifest), 'error': str(error)}
    with concurrent.futures.ThreadPoolExecutor(max_workers=3) as pool:
        for index, (pack, error) in enumerate(pool.map(verify, manifests), 1):
            if error:
                errors.append(error)
            else:
                if pack['scripts']:
                    for asset, detail in pack['maps'].items():
                        assets[(asset, detail['crc'])].append({**detail, **{k: pack[k] for k in ('directory', 'revision', 'source', 'script_source')}})
                packages.append({k: v for k, v in pack.items() if k != 'maps'})
            if index % 10 == 0:
                print(f'Verified {index}/{len(manifests)} packages', flush=True)
    connection = sqlite3.connect(args.db.resolve().as_uri() + '?mode=ro', uri=True)
    rows = connection.execute('select id,file_path,file_hash,map_path,map_crc,version_major,version_minor,build_major,build_minor from replays order by id').fetchall()
    stock, results = {}, []
    for id, file, file_hash, raw, raw_crc, *engine in rows:
        path = re.sub(r'^[0-9a-f]{3}', '', raw.lower().replace('\\', '/'))
        asset = path + '/' + path.rsplit('/', 1)[-1] + '.map'
        crc, version = int(raw_crc, 16), tuple(engine)
        row = {'id': id, 'asset': asset, 'compatibility': raw_crc, 'engine': engine}
        try:
            if version not in stock:
                stock[version] = base_assets(args.game, version)
            game_error = None
        except Exception as error:
            game_error = str(error)
        if not file or not Path(file).is_file():
            row['status'] = 'missing_replay'
        elif hash_file(Path(file)) != file_hash:
            row['status'] = 'changed_replay'
        elif game_error:
            row.update(status='invalid_game_installation', error=game_error)
        elif path.startswith('data/maps/internal/'):
            match = custom_map(args.maps, asset, crc)
            row.update(status='verified_custom' if match else 'missing_custom_map')
            if match:
                row['map_file'] = match
        elif not path.startswith('data/maps/official/'):
            row['status'] = 'unsupported_custom_map'
        elif assets.get((asset, crc)):
            row.update(status='verified_community', matches=assets[(asset, crc)])
        elif '__' in path or '1.02+' in path or '1.03' in path:
            row['status'] = 'missing_matching_pack'
        else:
            row['status'] = 'verified_stock' if stock[version].get(asset) == crc else 'missing_stock_map'
        results.append(row)
    summary = dict(collections.Counter(r['status'] for r in results))
    report = {'checked_at': datetime.now(timezone.utc).isoformat(), 'verification': 'Replay hashes, exact assets, compiled map compatibility, package scripts and cached archive hashes; no game simulation.',
              'total_replays': len(rows), 'unique_requirements': len({(r['asset'], r['compatibility'], tuple(r['engine'])) for r in results}),
              'summary': summary, 'packages': packages, 'cache_errors': errors, 'results': results}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + '\n', encoding='utf8')
    print(json.dumps({'total': len(rows), 'summary': summary, 'cache_errors': errors}, indent=2))


if __name__ == '__main__':
    main()
