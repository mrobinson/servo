# Rooting experiments (not for landing)

Tools and notes from the October 2026 investigation of stack-rooting cost in `script`.

## Branches
- `rooting-experiment`: root statistics behind `SERVO_ROOT_STATS=1` (compile time), plus
  this directory. `performance.mark("__rootstats:<label>")` prints `ROOTSTATS <label> {json}`
  to stderr; the label `reset` clears the counters.
- `rooting-attribution`: adds `#[track_caller]` to the rooting entry points and prints
  `ROOTCALLER <label> <count> <file:line>` for the top callers. Only meaningful in a
  `SERVO_ROOT_STATS=1` build.
- `rooting-slab`: root collection where each root stores its slot index, with a free list.
- `rooting-order`: order-preserving unroot (tail shift) and LIFO drop of the
  post-connection list ("fix #5").
- `rooting-reverse-drop`: drops the post-connection roots newest-first. Disproved, see below.
- `rooting-lazy-box`: roots the post-connection list in `Node::insert` with one lazily
  created `RootedTraceableBox<Vec<Dom<Node>>>`.

The last two are based on the owner-document rooting commit plus the statistics commit.

## Running
```
SERVO_ROOT_STATS=1 ./mach build --release   # or without, for timing
cp target/release/servoshell target/release/servoshell-NAME
python3 -I etc/rooting-bench/harness.py micro target/release/servoshell-NAME out.json [--stats]
etc/rooting-bench/rounds.sh micro results/ 7 NAME1 NAME2
SPEEDOMETER_DIR=... etc/rooting-bench/rounds.sh speedometer results/ 3 NAME1 NAME2 -- \
    --suites TodoMVC-JavaScript-ES5,TodoMVC-JavaScript-ES6-Webpack-Complex-DOM,TodoMVC-React-Complex-DOM \
    --iterations 5
python3 -I etc/rooting-bench/compare.py micro results/NAME1-*.json results/NAME2-*.json
```

## Results so far (innerHTML-1600, median of 7 rounds)
- Baseline: ~63 ms. 16.9 M roots; 995 M root-list entries scanned on unroot.
- Slab: 32.6 ms. TodoMVC unchanged. `DomRoot` grows from 16 to 24 bytes.
- Owner-document commit: 16.9 M → 14.4 M roots (−15%), 63.3 → 61.0 ms;
  TodoMVC ES5 and ES6-Webpack about −4%.
- Reverse drop: 90 ms (+49%). `swap_remove` in `RootCollection::unroot` reorders the
  collection, so newest-first is the worst order.
- Lazy box: 30.2 ms (−50%), scanned entries 991 M → 17 M, peak live roots 20,809 → 18.
  createElement-append-10000 +2.3% (two allocations per connected insert, because
  `SmallVec` does not implement mozjs `Trace`). TodoMVC within noise.

## Remaining ideas
- `TreeIterator`: step with `UnrootedTreeIterator` and root only the next node (~24% of roots).
- Remaining `owner_doc()` / `owner_window()` calls, e.g. `add_pending_accessibility_damage`.
- `containing_shadow_root()` hoisted out of the `add_child` loop.
- Lazy box with a `SmallVec` fast path, or `Trace` for `SmallVec` in mozjs.
