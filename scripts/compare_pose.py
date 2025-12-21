#!/usr/bin/env python3
import argparse
import json
import math
from pathlib import Path


def load_pose(path: Path) -> dict:
    root = json.loads(path.read_text(encoding="utf-8"))
    bones = {}
    for b in root.get("bones", []):
        name = b.get("name")
        if not isinstance(name, str):
            continue
        bones[name] = b

    slots = {}
    for s in root.get("slots", []):
        name = s.get("name")
        if not isinstance(name, str):
            continue
        slots[name] = s

    def load_named_map(key: str) -> dict:
        out = {}
        for item in root.get(key, []) or []:
            name = item.get("name")
            if not isinstance(name, str):
                continue
            out[name] = item
        return out

    return {
        "meta": root,
        "bones": bones,
        "slots": slots,
        "drawOrder": root.get("drawOrder", None),
        "ikConstraints": load_named_map("ikConstraints"),
        "transformConstraints": load_named_map("transformConstraints"),
        "pathConstraints": load_named_map("pathConstraints"),
    }


def get_float(obj: dict, key: str, default: float = 0.0) -> float:
    v = obj.get(key, default)
    if isinstance(v, (int, float)) and math.isfinite(v):
        return float(v)
    return float(default)


def diff_one(r: dict, c: dict) -> dict:
    rw = r.get("world", {})
    cw = c.get("world", {})
    ra = r.get("applied", {})
    ca = c.get("applied", {})

    diffs = {}
    r_active = int(get_float(r, "active", 0.0))
    c_active = int(get_float(c, "active", 0.0))
    diffs["active"] = 0.0 if r_active == c_active else 1.0

    # Inactive bones are not updated by upstream runtimes. Their world/applied values are not
    # meaningful for parity, so only compare transforms for active bones.
    if r_active == 0 and c_active == 0:
        return diffs

    for k in ["a", "b", "c", "d", "x", "y"]:
        diffs[f"world.{k}"] = abs(get_float(rw, k) - get_float(cw, k))
    for k in ["x", "y", "rotation", "scaleX", "scaleY", "shearX", "shearY"]:
        diffs[f"applied.{k}"] = abs(get_float(ra, k) - get_float(ca, k))
    return diffs


def get_list4(obj, key: str):
    v = obj.get(key, None)
    if isinstance(v, list) and len(v) == 4:
        out = []
        for x in v:
            if isinstance(x, (int, float)) and math.isfinite(x):
                out.append(float(x))
            else:
                out.append(0.0)
        return out
    return [0.0, 0.0, 0.0, 0.0]


def get_str(obj: dict, key: str, default: str = "") -> str:
    v = obj.get(key, default)
    return v if isinstance(v, str) else default


def diff_slot(r: dict, c: dict) -> dict:
    diffs = {}
    rc = get_list4(r, "color")
    cc = get_list4(c, "color")
    for i, k in enumerate(["r", "g", "b", "a"]):
        diffs[f"color.{k}"] = abs(rc[i] - cc[i])

    rhd = int(get_float(r, "hasDark", 0.0))
    chd = int(get_float(c, "hasDark", 0.0))
    diffs["hasDark"] = 0.0 if rhd == chd else 1.0

    rdc = get_list4(r, "darkColor")
    cdc = get_list4(c, "darkColor")
    for i, k in enumerate(["r", "g", "b", "a"]):
        diffs[f"darkColor.{k}"] = abs(rdc[i] - cdc[i])

    ra = r.get("attachment", None)
    ca = c.get("attachment", None)
    if isinstance(ra, dict) and isinstance(ca, dict):
        diffs["attachment.name"] = 0.0 if get_str(ra, "name") == get_str(ca, "name") else 1.0
        if "type" in ra and "type" in ca:
            diffs["attachment.type"] = abs(get_float(ra, "type", -1) - get_float(ca, "type", -1))
        else:
            diffs["attachment.type"] = 0.0
    elif ra is None and ca is None:
        diffs["attachment.name"] = 0.0
        diffs["attachment.type"] = 0.0
    else:
        diffs["attachment.name"] = 1.0
        diffs["attachment.type"] = 1.0

    rsi = int(get_float(r, "sequenceIndex", -1))
    csi = int(get_float(c, "sequenceIndex", -1))
    diffs["sequenceIndex"] = 0.0 if rsi == csi else 1.0
    return diffs


def diff_constraint(r: dict, c: dict, keys: list[str]) -> dict:
    diffs = {}
    for k in keys:
        diffs[k] = abs(get_float(r, k) - get_float(c, k))
    return diffs


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("rust_json", type=Path)
    ap.add_argument("cpp_json", type=Path)
    ap.add_argument("--top", type=int, default=20)
    ap.add_argument("--eps", type=float, default=1e-3)
    ap.add_argument("--bone", type=str, default="")
    args = ap.parse_args()

    rust = load_pose(args.rust_json)
    cpp = load_pose(args.cpp_json)

    names = sorted(set(rust["bones"].keys()) | set(cpp["bones"].keys()))
    missing = [n for n in names if n not in rust["bones"] or n not in cpp["bones"]]
    if missing:
        print(f"missing bones: {len(missing)}")
        for n in missing[: min(20, len(missing))]:
            print(f"  {n}")

    worst = []
    for n in names:
        r = rust["bones"].get(n)
        c = cpp["bones"].get(n)
        if r is None or c is None:
            continue
        diffs = diff_one(r, c)
        m = max(diffs.values()) if diffs else 0.0
        if m >= args.eps:
            worst.append((m, n))

    worst.sort(reverse=True)
    print(f"diff >= {args.eps}: {len(worst)}/{len(names)}")
    for m, n in worst[: args.top]:
        print(f"{m:.6g}\t{n}")

    if args.bone:
        n = args.bone
        r = rust["bones"].get(n)
        c = cpp["bones"].get(n)
        if r is None or c is None:
            print(f"\n--bone {n}: missing in one side")
        else:
            print(f"\n--bone {n}: per-field diffs")
            diffs = diff_one(r, c)
            for k in sorted(diffs.keys()):
                d = diffs[k]
                if d >= args.eps:
                    space, field = k.split(".", 1)
                    rv = r.get(space, {}).get(field, None)
                    cv = c.get(space, {}).get(field, None)
                    print(f"  {k}\tdiff={d:.6g}\trust={rv}\tcpp={cv}")

    # Slots.
    slot_names = sorted(set(rust.get("slots", {}).keys()) | set(cpp.get("slots", {}).keys()))
    if slot_names:
        missing_slots = [n for n in slot_names if n not in rust["slots"] or n not in cpp["slots"]]
        if missing_slots:
            print(f"\nmissing slots: {len(missing_slots)}")
            for n in missing_slots[: min(20, len(missing_slots))]:
                print(f"  {n}")

        slot_worst = []
        for n in slot_names:
            r = rust["slots"].get(n)
            c = cpp["slots"].get(n)
            if r is None or c is None:
                continue
            diffs = diff_slot(r, c)
            m = max(diffs.values()) if diffs else 0.0
            if m >= args.eps:
                slot_worst.append((m, n))
        slot_worst.sort(reverse=True)
        print(f"\nslot diff >= {args.eps}: {len(slot_worst)}/{len(slot_names)}")
        for m, n in slot_worst[: args.top]:
            print(f"{m:.6g}\t{n}")

        # If you need per-slot diffs, extend this script with a dedicated `--slot` option.

    # Draw order.
    rdo = rust.get("drawOrder", None)
    cdo = cpp.get("drawOrder", None)
    if rdo is not None or cdo is not None:
        same = rdo == cdo
        if not same:
            print("\ndrawOrder: mismatch")
            print("  rust:", rdo if isinstance(rdo, list) else None)
            print("  cpp :", cdo if isinstance(cdo, list) else None)
        else:
            print("\ndrawOrder: ok")

    # Constraints.
    def compare_named_map(label: str, rust_map: dict, cpp_map: dict, keys: list[str]):
        names = sorted(set(rust_map.keys()) | set(cpp_map.keys()))
        if not names:
            return
        missing = [n for n in names if n not in rust_map or n not in cpp_map]
        if missing:
            print(f"\nmissing {label}: {len(missing)}")
            for n in missing[: min(20, len(missing))]:
                print(f"  {n}")
        worst = []
        for n in names:
            r = rust_map.get(n)
            c = cpp_map.get(n)
            if r is None or c is None:
                continue
            diffs = diff_constraint(r, c, keys)
            m = max(diffs.values()) if diffs else 0.0
            if m >= args.eps:
                worst.append((m, n))
        worst.sort(reverse=True)
        print(f"\n{label} diff >= {args.eps}: {len(worst)}/{len(names)}")
        for m, n in worst[: args.top]:
            print(f"{m:.6g}\t{n}")

    compare_named_map(
        "ikConstraints",
        rust.get("ikConstraints", {}),
        cpp.get("ikConstraints", {}),
        ["mix", "softness", "bendDirection", "active"],
    )
    compare_named_map(
        "transformConstraints",
        rust.get("transformConstraints", {}),
        cpp.get("transformConstraints", {}),
        ["mixRotate", "mixX", "mixY", "mixScaleX", "mixScaleY", "mixShearY", "active"],
    )
    compare_named_map(
        "pathConstraints",
        rust.get("pathConstraints", {}),
        cpp.get("pathConstraints", {}),
        ["position", "spacing", "mixRotate", "mixX", "mixY", "active"],
    )

    if args.bone:
        n = args.bone
        print("\n--- bone ---")
        print("name:", n)
        print("rust:", json.dumps(rust["bones"].get(n, None), indent=2, sort_keys=True))
        print("cpp :", json.dumps(cpp["bones"].get(n, None), indent=2, sort_keys=True))

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
