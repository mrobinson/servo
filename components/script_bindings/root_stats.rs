/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! EXPERIMENT: statistics about the stack root collection, used to investigate rooting
//! performance. Only compiled in when building with `SERVO_ROOT_STATS=1`; otherwise every
//! recording function is a no-op that the compiler removes.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::panic::Location;

/// Whether rooting statistics are recorded.
pub const ROOT_STATS_ENABLED: bool = option_env!("SERVO_ROOT_STATS").is_some();

const HISTOGRAM_BUCKETS: usize = 24;

/// Counters for one thread's root collection.
pub(crate) struct RootStats {
    /// Number of roots added.
    roots: Cell<u64>,
    /// Number of roots removed.
    unroots: Cell<u64>,
    /// Number of entries compared while searching for roots to remove.
    scanned: Cell<u64>,
    /// Longest search for a root to remove.
    max_scan: Cell<u64>,
    /// Number of entries moved while removing roots.
    moved: Cell<u64>,
    /// Number of roots removed from the end of the collection (LIFO order).
    lifo_unroots: Cell<u64>,
    /// Search distances: bucket 0 counts distance 0, bucket `i` counts distances in
    /// `[2^(i - 1), 2^i)`.
    scan_histogram: [Cell<u64>; HISTOGRAM_BUCKETS],
    /// Highest number of live roots.
    peak_live: Cell<usize>,
    /// Highest number of slots in the collection, live or not.
    peak_slots: Cell<usize>,
    /// Highest number of bytes allocated by the collection.
    peak_bytes: Cell<usize>,
    /// Number of roots created by each caller.
    callers: RefCell<Option<HashMap<&'static Location<'static>, u64>>>,
}

impl RootStats {
    pub(crate) const fn new() -> Self {
        Self {
            roots: Cell::new(0),
            unroots: Cell::new(0),
            scanned: Cell::new(0),
            max_scan: Cell::new(0),
            moved: Cell::new(0),
            lifo_unroots: Cell::new(0),
            scan_histogram: [const { Cell::new(0) }; HISTOGRAM_BUCKETS],
            peak_live: Cell::new(0),
            peak_slots: Cell::new(0),
            peak_bytes: Cell::new(0),
            callers: RefCell::new(None),
        }
    }

    /// Record the caller that created a root.
    pub(crate) fn record_caller(&self, caller: &'static Location<'static>) {
        if !ROOT_STATS_ENABLED {
            return;
        }
        *self.callers.borrow_mut().get_or_insert_with(HashMap::new).entry(caller).or_default() += 1;
    }

    /// Record a root being added, with the resulting size of the collection.
    #[inline(always)]
    pub(crate) fn record_root(&self, live: usize, slots: usize, bytes: usize) {
        if !ROOT_STATS_ENABLED {
            return;
        }
        self.roots.set(self.roots.get() + 1);
        self.peak_live.set(self.peak_live.get().max(live));
        self.peak_slots.set(self.peak_slots.get().max(slots));
        self.peak_bytes.set(self.peak_bytes.get().max(bytes));
    }

    /// Record a root being removed: `scanned` entries were compared to find it, `moved`
    /// entries were moved to remove it and `was_last` says whether it was the most recent root.
    #[inline(always)]
    pub(crate) fn record_unroot(&self, scanned: u64, moved: u64, was_last: bool) {
        if !ROOT_STATS_ENABLED {
            return;
        }
        self.unroots.set(self.unroots.get() + 1);
        self.scanned.set(self.scanned.get() + scanned);
        self.max_scan.set(self.max_scan.get().max(scanned));
        self.moved.set(self.moved.get() + moved);
        if was_last {
            self.lifo_unroots.set(self.lifo_unroots.get() + 1);
        }
        let bucket = (u64::BITS - scanned.leading_zeros()) as usize;
        let bucket = &self.scan_histogram[bucket.min(HISTOGRAM_BUCKETS - 1)];
        bucket.set(bucket.get() + 1);
    }

    /// Print the statistics to stderr as a single `ROOTSTATS <label> <json>` line and reset
    /// them. The peaks restart from the current size of the collection.
    pub(crate) fn dump_and_reset(&self, label: &str, live: usize, slots: usize, bytes: usize) {
        if !ROOT_STATS_ENABLED {
            return;
        }
        let histogram: Vec<u64> = self.scan_histogram.iter().map(Cell::get).collect();
        let last_nonzero = histogram.iter().rposition(|count| *count != 0).map_or(0, |i| i + 1);
        let json = format!(
            "{{\"roots\": {}, \"unroots\": {}, \"scanned\": {}, \"max_scan\": {}, \
             \"moved\": {}, \"lifo_unroots\": {}, \"scan_histogram\": {:?}, \
             \"live\": {}, \"slots\": {}, \"bytes\": {}, \"peak_live\": {}, \
             \"peak_slots\": {}, \"peak_bytes\": {}}}",
            self.roots.get(),
            self.unroots.get(),
            self.scanned.get(),
            self.max_scan.get(),
            self.moved.get(),
            self.lifo_unroots.get(),
            &histogram[..last_nonzero],
            live,
            slots,
            bytes,
            self.peak_live.get(),
            self.peak_slots.get(),
            self.peak_bytes.get(),
        );
        eprintln!("ROOTSTATS {label} {json}");
        if let Some(callers) = self.callers.borrow_mut().take() {
            let mut callers: Vec<_> = callers.into_iter().collect();
            callers.sort_by(|a, b| b.1.cmp(&a.1));
            for (caller, count) in callers.iter().take(60) {
                eprintln!("ROOTCALLER {label} {count} {caller}");
            }
        }

        for counter in [
            &self.roots,
            &self.unroots,
            &self.scanned,
            &self.max_scan,
            &self.moved,
            &self.lifo_unroots,
        ] {
            counter.set(0);
        }
        for bucket in &self.scan_histogram {
            bucket.set(0);
        }
        self.peak_live.set(live);
        self.peak_slots.set(slots);
        self.peak_bytes.set(bytes);
    }
}
