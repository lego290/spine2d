#!/usr/bin/env python3
import argparse
import json
import math
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, List, Tuple


@dataclass(frozen=True)
class TriRef:
    draw_index: int
    tri_index: int


@dataclass(frozen=True)
class Triangle:
    page: int
    blend: str
    v: Tuple[Tuple[float, float, float, float], Tuple[float, float, float, float], Tuple[float, float, float, float]]
    c: Tuple[int, int, int]
    dc: Tuple[int, int, int]
    ref: TriRef


def _load_json(path: Path) -> dict:
    with path.open("r", encoding="utf-8") as f:
        return json.load(f)


def _is_finite(x: float) -> bool:
    return isinstance(x, (int, float)) and math.isfinite(x)


def _triangles(doc: dict) -> List[Triangle]:
    draws = doc.get("draws", [])
    out: List[Triangle] = []
    for draw_i, draw in enumerate(draws):
        page = int(draw.get("page", 0))
        blend = str(draw.get("blend", "unknown"))
        pos = draw.get("positions", [])
        uvs = draw.get("uvs", [])
        colors = draw.get("colors", [])
        dark_colors = draw.get("dark_colors", [])
        idx = draw.get("indices", [])
        num_vertices = int(draw.get("num_vertices", 0))

        if len(pos) != num_vertices * 2 or len(uvs) != num_vertices * 2:
            raise ValueError(
                f"draw[{draw_i}]: invalid positions/uvs length: "
                f"positions={len(pos)} uvs={len(uvs)} num_vertices={num_vertices}"
            )
        if len(colors) != num_vertices or len(dark_colors) != num_vertices:
            raise ValueError(
                f"draw[{draw_i}]: invalid colors length: "
                f"colors={len(colors)} dark_colors={len(dark_colors)} num_vertices={num_vertices}"
            )
        if len(idx) % 3 != 0:
            raise ValueError(f"draw[{draw_i}]: indices length not divisible by 3: {len(idx)}")

        def vertex(i: int) -> Tuple[float, float, float, float, int, int]:
            if i < 0 or i >= num_vertices:
                raise ValueError(f"draw[{draw_i}]: index out of range: {i} / {num_vertices}")
            x, y = pos[2 * i], pos[2 * i + 1]
            u, v = uvs[2 * i], uvs[2 * i + 1]
            c = int(colors[i])
            dc = int(dark_colors[i])
            return (float(x), float(y), float(u), float(v), c, dc)

        for tri_i in range(0, len(idx), 3):
            i0, i1, i2 = int(idx[tri_i]), int(idx[tri_i + 1]), int(idx[tri_i + 2])
            v0 = vertex(i0)
            v1 = vertex(i1)
            v2 = vertex(i2)
            out.append(
                Triangle(
                    page=page,
                    blend=blend,
                    v=((v0[0], v0[1], v0[2], v0[3]), (v1[0], v1[1], v1[2], v1[3]), (v2[0], v2[1], v2[2], v2[3])),
                    c=(v0[4], v1[4], v2[4]),
                    dc=(v0[5], v1[5], v2[5]),
                    ref=TriRef(draw_index=draw_i, tri_index=tri_i // 3),
                )
            )
    return out


def _abs_diff(a: float, b: float) -> float:
    return abs(float(a) - float(b))


def _max_component_diff(
    ta: Triangle, tb: Triangle, eps_pos: float, eps_uv: float
) -> Tuple[bool, float, float]:
    max_pos = 0.0
    max_uv = 0.0
    for va, vb in zip(ta.v, tb.v):
        ax, ay, au, av = va
        bx, by, bu, bv = vb

        if not (_is_finite(ax) and _is_finite(ay) and _is_finite(au) and _is_finite(av)):
            return (False, float("inf"), float("inf"))
        if not (_is_finite(bx) and _is_finite(by) and _is_finite(bu) and _is_finite(bv)):
            return (False, float("inf"), float("inf"))

        max_pos = max(max_pos, _abs_diff(ax, bx), _abs_diff(ay, by))
        max_uv = max(max_uv, _abs_diff(au, bu), _abs_diff(av, bv))

    ok = max_pos <= eps_pos and max_uv <= eps_uv
    return (ok, max_pos, max_uv)

def _unpack_aarrggbb(x: int) -> Tuple[int, int, int, int]:
    a = (x >> 24) & 0xff
    r = (x >> 16) & 0xff
    g = (x >> 8) & 0xff
    b = (x >> 0) & 0xff
    return (a, r, g, b)

def _max_color_channel_diff(a: int, b: int) -> int:
    aa, ar, ag, ab = _unpack_aarrggbb(a)
    ba, br, bg, bb = _unpack_aarrggbb(b)
    return max(abs(aa - ba), abs(ar - br), abs(ag - bg), abs(ab - bb))


def main() -> int:
    ap = argparse.ArgumentParser(description="Compare Spine render dumps (C++ oracle vs Rust).")
    ap.add_argument("a", type=Path, help="First JSON (e.g. C++ oracle)")
    ap.add_argument("b", type=Path, help="Second JSON (e.g. Rust render_dump)")
    ap.add_argument("--eps-pos", type=float, default=1e-4, help="Position epsilon")
    ap.add_argument("--eps-uv", type=float, default=1e-5, help="UV epsilon")
    ap.add_argument("--check-colors", action="store_true", help="Compare packed vertex light colors")
    ap.add_argument("--check-dark-colors", action="store_true", help="Compare packed vertex dark colors")
    ap.add_argument("--eps-color", type=int, default=1, help="Color channel epsilon (0-255)")
    ap.add_argument("--ignore-page", action="store_true", help="Ignore atlas page mismatch")
    ap.add_argument("--ignore-blend", action="store_true", help="Ignore blend mode mismatch")
    args = ap.parse_args()

    doc_a = _load_json(args.a)
    doc_b = _load_json(args.b)
    tris_a = _triangles(doc_a)
    tris_b = _triangles(doc_b)

    min_len = min(len(tris_a), len(tris_b))
    max_pos = 0.0
    max_uv = 0.0
    max_color = 0
    max_dark_color = 0

    for i in range(min_len):
        ta = tris_a[i]
        tb = tris_b[i]

        if not args.ignore_page and ta.page != tb.page:
            print(f"Mismatch at triangle #{i}: page {ta.page} != {tb.page}")
            print(f"  A: draw={ta.ref.draw_index} tri={ta.ref.tri_index} blend={ta.blend}")
            print(f"  B: draw={tb.ref.draw_index} tri={tb.ref.tri_index} blend={tb.blend}")
            return 1

        if not args.ignore_blend and ta.blend != tb.blend:
            print(f"Mismatch at triangle #{i}: blend {ta.blend} != {tb.blend}")
            print(f"  A: draw={ta.ref.draw_index} tri={ta.ref.tri_index} page={ta.page}")
            print(f"  B: draw={tb.ref.draw_index} tri={tb.ref.tri_index} page={tb.page}")
            return 1

        ok, dpos, duv = _max_component_diff(ta, tb, args.eps_pos, args.eps_uv)
        max_pos = max(max_pos, dpos)
        max_uv = max(max_uv, duv)
        if not ok:
            print(f"Mismatch at triangle #{i}: max_pos_diff={dpos} max_uv_diff={duv}")
            print(f"  A: draw={ta.ref.draw_index} tri={ta.ref.tri_index} page={ta.page} blend={ta.blend}")
            print(f"  B: draw={tb.ref.draw_index} tri={tb.ref.tri_index} page={tb.page} blend={tb.blend}")
            print("  A vertices (x,y,u,v):")
            for v in ta.v:
                print(f"    {v}")
            print("  B vertices (x,y,u,v):")
            for v in tb.v:
                print(f"    {v}")
            return 1

        if args.check_colors:
            for ca, cb in zip(ta.c, tb.c):
                d = _max_color_channel_diff(ca, cb)
                max_color = max(max_color, d)
                if d > args.eps_color:
                    print(f"Mismatch at triangle #{i}: max_color_channel_diff={d}")
                    print(f"  A: draw={ta.ref.draw_index} tri={ta.ref.tri_index} page={ta.page} blend={ta.blend}")
                    print(f"  B: draw={tb.ref.draw_index} tri={tb.ref.tri_index} page={tb.page} blend={tb.blend}")
                    print(f"  A colors: {[hex(x) for x in ta.c]}")
                    print(f"  B colors: {[hex(x) for x in tb.c]}")
                    return 1

        if args.check_dark_colors:
            for da, db in zip(ta.dc, tb.dc):
                d = _max_color_channel_diff(da, db)
                max_dark_color = max(max_dark_color, d)
                if d > args.eps_color:
                    print(f"Mismatch at triangle #{i}: max_dark_color_channel_diff={d}")
                    print(f"  A: draw={ta.ref.draw_index} tri={ta.ref.tri_index} page={ta.page} blend={ta.blend}")
                    print(f"  B: draw={tb.ref.draw_index} tri={tb.ref.tri_index} page={tb.page} blend={tb.blend}")
                    print(f"  A dark_colors: {[hex(x) for x in ta.dc]}")
                    print(f"  B dark_colors: {[hex(x) for x in tb.dc]}")
                    return 1

    if len(tris_a) != len(tris_b):
        print(f"Triangle count mismatch: {len(tris_a)} != {len(tris_b)} (matched {min_len})")
        return 1

    extras = ""
    if args.check_colors:
        extras += f" max_color_channel_diff={max_color}"
    if args.check_dark_colors:
        extras += f" max_dark_color_channel_diff={max_dark_color}"
    print(f"OK: triangles={len(tris_a)} max_pos_diff={max_pos} max_uv_diff={max_uv}{extras}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
