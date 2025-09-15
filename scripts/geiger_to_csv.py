#!/usr/bin/env python3
"""Extract unsafe counts for selected crates from target/geiger/geiger-full.json
and write tmp_logs/geiger_top30_aggregated.csv.
"""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
GFILE = ROOT / 'target' / 'geiger' / 'geiger-full.json'
OUTDIR = ROOT / 'tmp_logs'
OUTFILE = OUTDIR / 'geiger_top30_aggregated.csv'
TARGETS = ['getrandom','signal-hook-registry','secp256k1','backtrace','bytes','smallvec']

if not GFILE.exists():
    print('ERROR: geiger-full.json not found at', GFILE)
    raise SystemExit(1)

OUTDIR.mkdir(parents=True, exist_ok=True)

data = json.loads(GFILE.read_text(encoding='utf-8'))
# cargo-geiger JSON layout varies by version. Try common keys.
packages = data.get('packages') or data.get('crates') or []
rows = []
for p in packages:
    # different versions: item may have 'package' or 'id' fields
    pkg = p.get('package') or p
    pid = pkg.get('id') or pkg.get('id', {})
    name = None
    version = None
    if isinstance(pid, dict):
        name = pid.get('name')
        version = pid.get('version')
    else:
        # older layout: p['name'], p['version']
        name = p.get('name')
        version = p.get('version')

    if not name:
        continue

    unsafety = p.get('unsafety') or p.get('unsafety', {})
    used = unsafety.get('used') or {}
    exprs = used.get('exprs') or {}
    funcs = used.get('functions') or used.get('funcs') or {}
    u_exprs = exprs.get('unsafe_') or exprs.get('unsafe', 0)
    u_funcs = funcs.get('unsafe_') or funcs.get('unsafe', 0)
    rows.append((name, version or '', int(u_exprs or 0), int(u_funcs or 0)))

# write CSV for target crates
with open(OUTFILE, 'w', encoding='utf-8') as f:
    f.write('crate,version,unsafe_exprs,unsafe_funcs\n')
    for name,ver,ue,uf in rows:
        if name in TARGETS:
            f.write(f'{name},{ver},{ue},{uf}\n')

print('Wrote', OUTFILE)
