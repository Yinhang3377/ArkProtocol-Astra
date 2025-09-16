import json
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
GFILE = ROOT / 'target' / 'geiger' / 'geiger-full.json'
OUTDIR = ROOT / 'target' / 'geiger'
OUTFILE = OUTDIR / 'extract_manual.txt'
TARGETS = ['getrandom','signal-hook-registry','secp256k1','backtrace','bytes','smallvec']

if not GFILE.exists():
    print(f'ERROR: geiger file not found: {GFILE}')
    raise SystemExit(1)

OUTDIR.mkdir(parents=True, exist_ok=True)

data = json.loads(GFILE.read_text(encoding='utf-8'))
packages = data.get('packages', [])
index = {}
for p in packages:
    name = p.get('package', {}).get('id', {}).get('name')
    if not name:
        continue
    index.setdefault(name, []).append(p)

lines = []
lines.append('Geiger manual extract for selected crates:')
for t in TARGETS:
    entries = index.get(t)
    if not entries:
        lines.append(f'- {t}: NOT FOUND in geiger-full.json')
        continue
    # combine versions conservatively
    for ent in entries:
        pid = ent.get('package', {}).get('id', {})
        version = pid.get('version')
        forbids = ent.get('unsafety', {}).get('forbids_unsafe')
        used = ent.get('unsafety', {}).get('used', {})
        used_funcs = used.get('functions', {})
        used_exprs = used.get('exprs', {})
        unused = ent.get('unsafety', {}).get('unused', {})
        line = (f'- {t} {version} | forbids_unsafe={forbids} | '
                f'used.functions: safe={used_funcs.get("safe",0)} unsafe={used_funcs.get("unsafe_",0)} | '
                f'used.exprs: safe={used_exprs.get("safe",0)} unsafe={used_exprs.get("unsafe_",0)} | '
                f'unused.exprs.unsafe={unused.get("exprs",{}).get("unsafe_",0)}')
        lines.append(line)

lines.append('\nAutomated recommendations:')
for t in TARGETS:
    entries = index.get(t)
    if not entries:
        lines.append(f'- {t}: MISSING in local geiger -> re-run cargo-geiger on CI; or fetch artifact from Actions')
        continue
    max_used_unsafe = 0
    vers = []
    for ent in entries:
        v = ent.get('package', {}).get('id', {}).get('version')
        vers.append(v)
        u_exprs = ent.get('unsafety', {}).get('used', {}).get('exprs', {}).get('unsafe_',0)
        u_funcs = ent.get('unsafety', {}).get('used', {}).get('functions', {}).get('unsafe_',0)
        max_used_unsafe = max(max_used_unsafe, u_exprs, u_funcs)
    import json
    from pathlib import Path

    ROOT = Path(__file__).resolve().parents[1]
    GFILE = ROOT / 'target' / 'geiger' / 'geiger-full.json'
    OUTDIR = ROOT / 'target' / 'geiger'
    OUTFILE = OUTDIR / 'extract_manual.txt'
    TARGETS = ['getrandom','signal-hook-registry','secp256k1','backtrace','bytes','smallvec']

    if not GFILE.exists():
        print(f'ERROR: geiger file not found: {GFILE}')
        raise SystemExit(1)

    OUTDIR.mkdir(parents=True, exist_ok=True)

    data = json.loads(GFILE.read_text(encoding='utf-8'))
    packages = data.get('packages', [])
    index = {}
    for p in packages:
        name = p.get('package', {}).get('id', {}).get('name')
        if not name:
            continue
        index.setdefault(name, []).append(p)

    lines = []
    lines.append('Geiger manual extract for selected crates:')
    for t in TARGETS:
        entries = index.get(t)
        if not entries:
            lines.append(f'- {t}: NOT FOUND in geiger-full.json')
            continue
        # combine versions conservatively
        for ent in entries:
            pid = ent.get('package', {}).get('id', {})
            version = pid.get('version')
            forbids = ent.get('unsafety', {}).get('forbids_unsafe')
            used = ent.get('unsafety', {}).get('used', {})
            used_funcs = used.get('functions', {})
            used_exprs = used.get('exprs', {})
            unused = ent.get('unsafety', {}).get('unused', {})
            line = (f'- {t} {version} | forbids_unsafe={forbids} | '
                    f'used.functions: safe={used_funcs.get("safe",0)} unsafe={used_funcs.get("unsafe_",0)} | '
                    f'used.exprs: safe={used_exprs.get("safe",0)} unsafe={used_exprs.get("unsafe_",0)} | '
                    f'unused.exprs.unsafe={unused.get("exprs",{}).get("unsafe_",0)}')
            lines.append(line)

    lines.append('\nAutomated recommendations:')
    for t in TARGETS:
        entries = index.get(t)
        if not entries:
            lines.append(f'- {t}: MISSING in local geiger -> re-run cargo-geiger on CI; or fetch artifact from Actions')
            continue
        max_used_unsafe = 0
        vers = []
        for ent in entries:
            v = ent.get('package', {}).get('id', {}).get('version')
            vers.append(v)
            u_exprs = ent.get('unsafety', {}).get('used', {}).get('exprs', {}).get('unsafe_',0)
            u_funcs = ent.get('unsafety', {}).get('used', {}).get('functions', {}).get('unsafe_',0)
            max_used_unsafe = max(max_used_unsafe, u_exprs, u_funcs)
        if max_used_unsafe == 0:
            lines.append(f'- {t} {",".join(vers)}: NO used unsafe (geiger) — low risk from unsafe usage; still check advisories and release notes before upgrading/accepting')
        else:
            lines.append(f'- {t} {",".join(vers)}: FOUND used unsafe count={max_used_unsafe} — actions: (1) check if upgraded patch removes unsafe; (2) audit call sites in your dependency graph; (3) consider pinning to an audited commit or replacing if unsafe surface is unacceptable')

    OUTFILE.write_text('\n'.join(lines), encoding='utf-8')
    print('Wrote', OUTFILE)
