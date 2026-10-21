"""Summarize harness result files grouped by variant.

Usage: compare.py micro|speedometer FILE... (file names are VARIANT-ROUND.json)
For each test, prints the median across rounds of each variant's per-round median, and the
change relative to the first variant.
"""

import json
import os
import statistics
import sys
from collections import defaultdict


def variant_of(path):
    return os.path.basename(path).rsplit("-", 1)[0]


def load(mode, paths):
    per_variant = defaultdict(lambda: defaultdict(list))
    memory = defaultdict(lambda: defaultdict(list))
    for path in paths:
        with open(path) as f:
            data = json.load(f)
        variant = variant_of(path)
        if mode == "micro":
            for test, result in data["results"].items():
                per_variant[variant][test].append(result["median"])
        else:
            iterations = data["results"]
            per_variant[variant]["Score"].append(
                statistics.mean(iteration["score"] for iteration in iterations))
            for suite in iterations[0]["tests"]:
                per_variant[variant][suite].append(
                    statistics.median(iteration["tests"][suite]["total"] for iteration in iterations))
        for key, value in data.get("memory", {}).items():
            memory[variant][key].append(value)
    return per_variant, memory


def main():
    mode, paths = sys.argv[1], sys.argv[2:]
    per_variant, memory = load(mode, paths)
    variants = list(dict.fromkeys(variant_of(p) for p in paths))
    tests = list(per_variant[variants[0]].keys())
    base = variants[0]
    header = f"{'test':34}" + "".join(f"{v:>28}" for v in variants)
    print(header)
    for test in tests:
        row = f"{test:34}"
        base_value = statistics.median(per_variant[base][test])
        for variant in variants:
            values = per_variant[variant][test]
            value = statistics.median(values)
            spread = (max(values) - min(values)) / value * 100 if value else 0
            change = (value - base_value) / base_value * 100 if base_value else 0
            cell = f"{value:9.2f} ±{spread:4.1f}%"
            if variant != base:
                cell += f" ({change:+5.1f}%)"
            row += f"{cell:>28}"
        print(row)
    print()
    for key in ("avg_rss_mb", "peak_rss_mb"):
        row = f"{key:34}"
        for variant in variants:
            values = memory[variant][key]
            row += f"{statistics.median(values):>20.1f} MB ±{(max(values) - min(values)) / 2:5.1f}"
        print(row)


if __name__ == "__main__":
    main()
