use std::sync::atomic::{AtomicIsize, Ordering};

use super::CachePadded;

// A sharded atomic counter.
//
// Sharding the length counter of `HashMap` is extremely important,
// as a single point of contention for insertions/deletions significantly
// degrades concurrent performance.
//
// We can take advantage of the fact that `seize::Guard`
pub struct Counter(Box<[CachePadded<AtomicIsize>]>);

impl Default for Counter {
    /// Create a new `Counter`.
    fn default() -> Counter {
        let num_cpus = std::thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(1);

        // Round up to the next power-of-two for fast modulo.
        let shards = (0..num_cpus.next_power_of_two())
            .map(|_| Default::default())
            .collect();

        Counter(shards)
    }
}

impl Counter {
    // Return the shard for the given thread ID.
    #[inline]
    pub fn get(&self) -> &AtomicIsize {
        // Guard thread IDs are essentially perfectly sharded due to
        // the internal thread ID allocator, which makes contention
        // very unlikely even with the exact number of shards as CPUs.
        // FIXME
        //let shard = guard.thread_id() & (self.0.len() - 1);
        let shard = thread_id::get() & (self.0.len() - 1);

        &self.0[shard].value
    }

    // Returns the sum of all counter shards.
    #[inline]
    pub fn sum(&self) -> usize {
        self.0
            .iter()
            .map(|x| x.value.load(Ordering::Relaxed))
            .sum::<isize>()
            .try_into()
            // Depending on the order of deletion/insertions this might be negative,
            // in which case we assume the map is empty.
            .unwrap_or(0)
    }
}
