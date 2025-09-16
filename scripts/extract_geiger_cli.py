import json
import os
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
GFILE = ROOT / 'target' / 'geiger' / 'geiger-full.json'
OUTDIR = ROOT / 'target' / 'geiger'
OUTFILE = OUTDIR / 'extract_cli.txt'
TARGETS = ['getrandom','signal-hook-registry','secp256k1','backtrace','bytes','smallvec']

if not GFILE.exists():
    print(f'ERROR: geiger file not found: {GFILE}')
    raise SystemExit(1)

OUTDIR.mkdir(parents=True, exist_ok=True)

with open(GFILE, 'r', encoding='utf-8') as f:
    data = json.load(f)

packages = data.get('packages', [])
index = {}
for p in packages:
    name = p.get('package', {}).get('id', {}).get('name')
    if not name:
        continue
    index.setdefault(name, []).append(p)

report_lines = []
report_lines.append('Geiger extract for target crates:')
for t in TARGETS:
    entries = index.get(t)
    if not entries:
        report_lines.append(f'- {t}: NOT FOUND in geiger-full.json')
        continue
    for ent in entries:
        pid = ent.get('package', {}).get('id', {})
        version = pid.get('version')
        src = pid.get('source')
        forbids = ent.get('unsafety', {}).get('forbids_unsafe')
        used = ent.get('unsafety', {}).get('used', {})
        used_funcs = used.get('functions', {})
        used_exprs = used.get('exprs', {})
        unused = ent.get('unsafety', {}).get('unused', {})
        line = (f'- {t} {version} | forbids_unsafe={forbids} | '
                f'used.functions: safe={used_funcs.get("safe",0)} unsafe={used_funcs.get("unsafe_",0)} | '
                f'used.exprs: safe={used_exprs.get("safe",0)} unsafe={used_exprs.get("unsafe_",0)} | '
                f'unused.exprs.unsafe={unused.get("exprs",{}).get("unsafe_",0)}')
        report_lines.append(line)

# Add short recommendations
report_lines.append('\nRecommendations (automated hints):')
for t in TARGETS:
    entries = index.get(t)
    if not entries:
        report_lines.append(f'- {t}: missing — re-run unsafe scanner on linux/CI to get stable metrics')
        continue
    # pick max unsafe counts across versions for conservative advice
    max_used_unsafe = 0
    versions = []
    for ent in entries:
        v = ent.get('package', {}).get('id', {}).get('version')
        versions.append(v)
        u_exprs = ent.get('unsafety', {}).get('used', {}).get('exprs', {}).get('unsafe_',0)
        u_funcs = ent.get('unsafety', {}).get('used', {}).get('functions', {}).get('unsafe_',0)
        max_used_unsafe = max(max_used_unsafe, u_exprs, u_funcs)
    if max_used_unsafe==0:
        report_lines.append(f'- {t} {",".join(versions)}: no "used" unsafe found — low risk from unsafe usage (still review changelogs).')
    else:
        report_lines.append(f'- {t} {",".join(versions)}: found used unsafe ({max_used_unsafe}) — review code paths, prefer upgrading to latest patch/minor, or sandbox/review unsafe sites.')

# write file and print
with open(OUTFILE, 'w', encoding='utf-8') as f:
    f.write('\n'.join(report_lines))

print('\n'.join(report_lines))
print('\nWrote to: ' + str(OUTFILE))
