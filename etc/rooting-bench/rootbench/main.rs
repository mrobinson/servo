// Standalone benchmark of the three root collection designs, using the access patterns
// measured in Servo: LIFO releases, releases at distance 2 (TreeIterator-like), and a burst of
// roots released in FIFO order (the post-connection steps of `Node::insert` before fix #5).
//
// Build with: rustc -O --edition 2024 main.rs

use std::hint::black_box;
use std::ptr::{self, NonNull};
use std::time::Instant;

trait Traceable {
    fn trace(&self) -> usize;
}

struct Object(usize);
impl Traceable for Object {
    fn trace(&self) -> usize {
        self.0
    }
}

trait Collection {
    /// Returns the slot (ignored by the searching designs).
    fn root(&mut self, object: *const dyn Traceable) -> u32;
    fn unroot(&mut self, slot: u32, object: *const dyn Traceable);
}

/// Baseline: search from the end, then swap_remove.
#[derive(Default)]
struct SwapRemove(Vec<*const dyn Traceable>);
impl Collection for SwapRemove {
    #[inline(never)]
    fn root(&mut self, object: *const dyn Traceable) -> u32 {
        self.0.push(object);
        0
    }
    #[inline(never)]
    fn unroot(&mut self, _: u32, object: *const dyn Traceable) {
        let roots = &mut self.0;
        let idx = roots
            .iter()
            .rposition(|r| ptr::addr_eq(*r, object))
            .unwrap();
        unsafe {
            let len = roots.len() - 1;
            if len != idx {
                let base = roots.as_mut_ptr();
                ptr::copy_nonoverlapping(base.add(len), base.add(idx), 1);
            }
            roots.set_len(len);
        }
    }
}

/// Fix #5: search from the end, then shift the tail.
#[derive(Default)]
struct Shift(Vec<*const dyn Traceable>);
impl Collection for Shift {
    #[inline(never)]
    fn root(&mut self, object: *const dyn Traceable) -> u32 {
        self.0.push(object);
        0
    }
    #[inline(never)]
    fn unroot(&mut self, _: u32, object: *const dyn Traceable) {
        let roots = &mut self.0;
        let idx = roots
            .iter()
            .rposition(|r| ptr::addr_eq(*r, object))
            .unwrap();
        unsafe {
            let len = roots.len() - 1;
            if len != idx {
                let base = roots.as_mut_ptr();
                ptr::copy(base.add(idx + 1), base.add(idx), len - idx);
            }
            roots.set_len(len);
        }
    }
}

/// Approach 2: slots with a free list.
#[derive(Default)]
struct Slab {
    slots: Vec<Option<NonNull<dyn Traceable>>>,
    free: Vec<u32>,
}
impl Collection for Slab {
    #[inline(never)]
    fn root(&mut self, object: *const dyn Traceable) -> u32 {
        let object = NonNull::new(object as *mut dyn Traceable);
        match self.free.pop() {
            Some(slot) => {
                self.slots[slot as usize] = object;
                slot
            },
            None => {
                self.slots.push(object);
                (self.slots.len() - 1) as u32
            },
        }
    }
    #[inline(never)]
    fn unroot(&mut self, slot: u32, _: *const dyn Traceable) {
        self.slots[slot as usize] = None;
        self.free.push(slot);
    }
}

const OBJECTS: usize = 32768;

fn objects() -> Vec<Object> {
    (0..OBJECTS).map(Object).collect()
}

fn ptr_of(objects: &[Object], i: usize) -> *const dyn Traceable {
    &objects[i % OBJECTS] as &dyn Traceable as *const dyn Traceable
}

/// Nested roots released in LIFO order, `depth` deep.
fn lifo<C: Collection>(c: &mut C, objects: &[Object], ops: usize) {
    let depth = 8;
    let mut stack = Vec::with_capacity(depth);
    for i in 0..ops / depth {
        for d in 0..depth {
            let p = ptr_of(objects, i * depth + d);
            stack.push((c.root(p), p));
        }
        while let Some((slot, p)) = stack.pop() {
            c.unroot(slot, p);
        }
    }
}

/// An iterator that roots the next node and then releases the previous one.
fn iterator<C: Collection>(c: &mut C, objects: &[Object], ops: usize) {
    let p = ptr_of(objects, 0);
    let mut previous = (c.root(p), p);
    for i in 1..ops {
        let p = ptr_of(objects, i);
        let next = (c.root(p), p);
        c.unroot(previous.0, previous.1);
        previous = next;
    }
    c.unroot(previous.0, previous.1);
}

/// A burst of `n` roots released in FIFO order.
fn fifo_burst<C: Collection>(c: &mut C, objects: &[Object], n: usize) {
    let mut roots = Vec::with_capacity(n);
    for i in 0..n {
        let p = ptr_of(objects, i);
        roots.push((c.root(p), p));
    }
    for (slot, p) in roots {
        c.unroot(slot, p);
    }
}

fn time<F: FnMut()>(mut f: F) -> f64 {
    let mut best = f64::MAX;
    for _ in 0..7 {
        let start = Instant::now();
        f();
        best = best.min(start.elapsed().as_secs_f64());
    }
    best
}

fn run<C: Collection + Default>(name: &str, objects: &[Object]) {
    const OPS: usize = 4_000_000;
    let mut c = C::default();
    let lifo_ns = time(|| lifo(black_box(&mut c), objects, OPS)) / OPS as f64 * 1e9;
    let iterator_ns = time(|| iterator(black_box(&mut c), objects, OPS)) / OPS as f64 * 1e9;
    let mut burst_ms = |n| time(|| fifo_burst(black_box(&mut c), objects, n)) * 1e3;
    let small = burst_ms(2600);
    let large = burst_ms(20800);
    println!(
        "{name:12} LIFO {lifo_ns:5.2} ns/root   iterator {iterator_ns:5.2} ns/root   \
         FIFO burst 2,600: {small:7.3} ms   20,800: {large:8.3} ms",
    );
}

fn main() {
    let objects = objects();
    run::<SwapRemove>("swap_remove", &objects);
    run::<Shift>("shift (#5)", &objects);
    run::<Slab>("slab", &objects);
    // Keep the trace method alive so the vtables are real.
    black_box(objects[0].trace());
}
