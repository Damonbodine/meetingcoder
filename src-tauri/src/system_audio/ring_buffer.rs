use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

/// Single-producer single-consumer lock-free ring buffer for f32 samples.
/// - Overwrites oldest samples when capacity is exceeded.
/// - Push is non-blocking and allocation-free.
/// - Drain allocates a Vec for the returned chunk.
pub struct SpscRingBuffer {
    buf: Box<[f32]>,
    cap: usize,
    // Write index (next position to write). Monotonic counter (mod cap used for slot).
    head: AtomicUsize,
    // Read index (next position to read). Monotonic counter.
    tail: AtomicUsize,
    // Total number of samples overwritten (oldest dropped) due to capacity limits.
    overwritten: AtomicU64,
}

impl SpscRingBuffer {
    pub fn new(capacity_samples: usize) -> Arc<Self> {
        let capacity = capacity_samples.max(1);
        let buf = vec![0.0f32; capacity].into_boxed_slice();
        Arc::new(Self {
            buf,
            cap: capacity,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            overwritten: AtomicU64::new(0),
        })
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.cap
    }

    /// Returns the current number of samples available to read.
    #[inline]
    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        head.saturating_sub(tail).min(self.cap)
    }

    /// Total overwritten samples since creation.
    #[inline]
    pub fn overwritten_count(&self) -> u64 {
        self.overwritten.load(Ordering::Relaxed)
    }

    /// Push samples into the ring buffer, overwriting oldest when full.
    /// Single producer only.
    pub fn push(&self, samples: &[f32]) {
        if samples.is_empty() {
            return;
        }

        let cap = self.cap;
        // Fast path: if incoming larger than capacity, only keep the last `cap` samples
        let (src, drop_count) = if samples.len() > cap {
            let drop = samples.len() - cap;
            (&samples[drop..], drop as u64)
        } else {
            (samples, 0)
        };

        if drop_count > 0 {
            self.overwritten.fetch_add(drop_count, Ordering::Relaxed);
        }

        let to_write = src.len();
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);

        // Calculate new head and how many will be overwritten beyond available space
        let available = head.saturating_sub(tail);
        let mut will_overflow = 0usize;
        if available + to_write > cap {
            will_overflow = available + to_write - cap;
        }

        if will_overflow > 0 {
            // Advance tail to drop the oldest samples
            self.tail.store(tail + will_overflow, Ordering::Release);
            self.overwritten
                .fetch_add(will_overflow as u64, Ordering::Relaxed);
        }

        // Copy into ring in up to two segments
        let mut write_index = head % cap;
        let first = (cap - write_index).min(to_write);
        // Safety: single producer guarantees exclusive write to these slots while writing
        unsafe {
            let dst_ptr = self.buf.as_ptr() as *mut f32;
            std::ptr::copy_nonoverlapping(src.as_ptr(), dst_ptr.add(write_index), first);
            if to_write > first {
                std::ptr::copy_nonoverlapping(src.as_ptr().add(first), dst_ptr, to_write - first);
            }
        }

        // Publish new head
        self.head.store(head + to_write, Ordering::Release);
    }

    /// Drain up to n samples from the ring into a Vec. Single consumer only.
    pub fn drain_n(&self, n: usize) -> Vec<f32> {
        if n == 0 {
            return Vec::new();
        }

        let cap = self.cap;
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Relaxed);
        let available = head.saturating_sub(tail).min(cap);
        let to_read = available.min(n);
        if to_read == 0 {
            return Vec::new();
        }

        let mut out = vec![0.0f32; to_read];
        let read_index = tail % cap;
        let first = (cap - read_index).min(to_read);

        // Safety: single consumer guarantees exclusive read of these slots during read
        unsafe {
            let src_ptr = self.buf.as_ptr();
            std::ptr::copy_nonoverlapping(src_ptr.add(read_index), out.as_mut_ptr(), first);
            if to_read > first {
                std::ptr::copy_nonoverlapping(
                    src_ptr,
                    out.as_mut_ptr().add(first),
                    to_read - first,
                );
            }
        }

        // Publish new tail
        self.tail.store(tail + to_read, Ordering::Release);
        out
    }
}
