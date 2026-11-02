"""Lay the SaltyShade 3D overlay (your design) over a fresh upstream TextureCraft checkout.

Usage: python tools/apply_overlay.py <upstream-dir> [--overlay overlay] [--dry-run]

1. Copies overlay/files/** into the checkout (new files such as the theme module).
2. Copies the assets listed under "copy" in overlay/rules.json (logo, icons).
3. Applies the text rules in overlay/rules.json.

It never edits the overlay. Exit code 1 when a required rule no longer fits upstream: the
updater then keeps the previous build instead of shipping one without your design.
"""

import argparse
import json
import re
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def literal_spans(src: str, rust: bool):
    """(start, end) of every string literal's contents. For Rust it skips comments, char
    literals and lifetimes, and understands raw strings, so a stray quote never flips parity.
    Any other file (HTML) is all text."""
    if not rust:
        yield 0, len(src)
        return
    i, n = 0, len(src)
    char_lit = re.compile(r"'(?:[^'\\\n]|\\(?:x[0-9a-fA-F]{2}|u\{[0-9a-fA-F]+\}|.))'")
    raw = re.compile(r'b?r(#*)"')
    while i < n:
        c = src[i]
        if src.startswith("//", i):
            j = src.find("\n", i)
            i = n if j < 0 else j
        elif src.startswith("/*", i):
            depth, i = 1, i + 2
            while i < n and depth:
                if src.startswith("/*", i):
                    depth, i = depth + 1, i + 2
                elif src.startswith("*/", i):
                    depth, i = depth - 1, i + 2
                else:
                    i += 1
        elif c == "'":
            m = char_lit.match(src, i)
            i = m.end() if m else i + 1  # a lifetime: just the tick
        elif (c in "rb") and (m := raw.match(src, i)) and (i == 0 or not (src[i - 1].isalnum() or src[i - 1] == "_")):
            close = '"' + m.group(1)
            j = src.find(close, m.end())
            j = n if j < 0 else j
            yield m.end(), j
            i = j + len(close)
        elif c == '"':
            j = i + 1
            while j < n and src[j] != '"':
                j += 2 if src[j] == "\\" else 1
            yield i + 1, j
            i = j + 1
        else:
            i += 1


def rename_in_literals(src: str, find: str, replace: str, rust: bool) -> tuple[str, int]:
    word = re.compile(re.escape(find) + r"(?![A-Za-z0-9_])")
    out, last, count = [], 0, 0
    for a, b in sorted(literal_spans(src, rust)):
        if a < last:
            continue
        seg, k = word.subn(replace, src[a:b])
        if k:
            out.append(src[last:a])
            out.append(seg)
            last, count = b, count + k
    out.append(src[last:])
    return "".join(out), count


def long_path(p: Path) -> Path:
    """Windows paths past 260 characters need the \\\\?\\ prefix unless long paths are enabled."""
    p = p.resolve()
    if sys.platform == "win32" and not str(p).startswith("\\\\?\\"):
        return Path("\\\\?\\" + str(p))
    return p


def expand(base: Path, patterns):
    seen = []
    for p in patterns:
        for f in sorted(base.glob(p)):
            if f.is_file() and f not in seen:
                seen.append(f)
    return seen


def apply_alt(text: str, alt: dict) -> tuple[str, int]:
    count = alt.get("count", 0)  # 0 = every match
    if alt.get("regex"):
        return re.subn(alt["find"], alt["replace"], text, count=count)
    k = text.count(alt["find"])
    if count:
        k = min(k, count)
    return (text.replace(alt["find"], alt["replace"], k) if k else text), k


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("upstream", type=Path)
    ap.add_argument("--overlay", type=Path, default=ROOT / "overlay")
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()
    up, ov = long_path(args.upstream), long_path(args.overlay)
    spec = json.loads((ov / "rules.json").read_text(encoding="utf-8"))
    ok, warned, failed = 0, 0, 0

    def write(path: Path, data):
        if args.dry_run:
            return
        path.parent.mkdir(parents=True, exist_ok=True)
        if isinstance(data, bytes):
            path.write_bytes(data)
        else:
            path.write_text(data, encoding="utf-8", newline="")

    files = ov / "files"
    for f in sorted(files.rglob("*")) if files.exists() else []:
        if f.is_file():
            write(up / f.relative_to(files), f.read_bytes())
            print(f"  + {f.relative_to(files).as_posix()}")
    for c in spec.get("copy", []):
        src = ov / c["from"]
        if not src.exists():
            print(f"  ! missing overlay asset {c['from']}")
            failed += 1
            continue
        write(up / c["to"], src.read_bytes())
    print(f"  copied {len(spec.get('copy', []))} assets")

    for rule in spec["rules"]:
        targets = expand(up, rule["files"])
        changed = 0
        problems = []
        for f in targets:
            text = f.read_text(encoding="utf-8")
            new = text
            if rule.get("type") == "string_literals":
                new, k = rename_in_literals(text, rule["find"], rule["replace"], f.suffix == ".rs")
                changed += k
            else:
                hits = []
                for alt in rule["alternatives"]:
                    new2, k = apply_alt(new, alt)
                    hits.append(k)
                    if k:
                        new = new2
                        if not rule.get("all_alternatives"):
                            break
                changed += sum(hits)
                if rule.get("all_alternatives"):
                    problems += [f"{f.name}: alternative {i + 1} did not match" for i, k in enumerate(hits) if not k]
            if new != text:
                write(f, new)
        if not targets:
            problems.append("no files matched " + ", ".join(rule["files"]))
        if changed and not problems:
            ok += 1
            print(f"  ok   {rule['desc']} ({changed})")
        elif rule.get("required") and not changed:
            failed += 1
            print(f"  FAIL {rule['desc']}: upstream changed, rule no longer fits")
        else:
            warned += 1
            detail = "; ".join(problems) if problems else "nothing matched"
            print(f"  warn {rule['desc']}: {detail}")

    print(f"overlay: {ok} applied, {warned} warnings, {failed} failures")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
