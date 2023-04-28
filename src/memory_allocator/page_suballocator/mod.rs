//! An allocator which allocates chunks from an existing allocation.

mod page_arena;

use {
    crate::{Allocation, AllocatorError},
    anyhow::Context,
};

pub struct PageSuballocator {
    allocation: Allocation,
    page_size_in_bytes: u64,
    arena: page_arena::PageArena,
}

impl PageSuballocator {
    /// Create an allocator which takes memory from an existing allocation.
    ///
    /// # Params
    ///
    /// * allocation: The allocation to use for suballocations.
    /// * page_size_in_bytes: The size of each page in the allocation. The
    ///   trade-off is that larger pages can waste memory for small allocations
    ///   while small pages will increase allocation time.
    ///
    /// # Panic
    ///
    /// Panics if allocation.size_in_bytes is not a multiple of
    /// page_size_in_bytes.
    pub fn for_allocation(
        allocation: Allocation,
        page_size_in_bytes: u64,
    ) -> Self {
        assert!(
            allocation.size_in_bytes() % page_size_in_bytes == 0,
            "page_size_in_bytes must be a multiple of the allocation size"
        );
        let page_count = allocation.size_in_bytes() / page_size_in_bytes;
        Self {
            allocation,
            page_size_in_bytes,
            arena: page_arena::PageArena::new(page_count as usize),
        }
    }

    /// Releases ownership of the underlying allocation.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    /// - ownership is transferred, regardless of existing suballocations.
    /// - the application must ensure that no suballocations are in-use after
    ///   this call.
    pub fn release_allocation(self) -> Allocation {
        self.allocation
    }

    /// Returns true when all suballocations have been freed.
    pub fn is_empty(&self) -> bool {
        self.arena.is_empty()
    }

    /// Suballocate a region of memory.
    ///
    /// # Params
    ///
    /// * size_in_bytes: the required size of the allocation.
    /// * alignment: the required alignment of the allocation.
    ///
    /// # Safety
    ///
    /// Unsafe because
    /// * The caller must free the returned allocation
    /// * The caller is responsible for synchronizing access (CPU and GPU) to
    ///   the underlying memory
    /// * The returned memory will always be aligned to the page size relative
    ///   to the original allocation's offset.
    pub unsafe fn allocate(
        &mut self,
        size_in_bytes: u64,
        alignment: u64,
    ) -> Result<Allocation, AllocatorError> {
        if (self.allocation.offset_in_bytes() + self.page_size_in_bytes)
            % alignment
            == 0
        {
            // The page boundaries are already aligned for this request, so
            // no extra work is needed.
            return self.allocate_unaligned(size_in_bytes);
        }

        // Add enough additional size that the offset can be aligned.
        let aligned_size = size_in_bytes + (alignment - 1);
        let unaligned = self.allocate_unaligned(aligned_size)?;

        // How many bytes must the offset be advanced to reach the next aligned
        // value?
        //
        // Note that (alignment - unaligned.offset_in_bytes() % alignment) is
        // always <= alignment-1. So this correction will always leave enough
        // space for the requested size_in_bytes.
        let alignment_correction = {
            if unaligned.offset_in_bytes() % alignment == 0 {
                0
            } else {
                alignment - (unaligned.offset_in_bytes() % alignment)
            }
        };

        Ok(Allocation::suballocate(
            &unaligned,
            alignment_correction,
            size_in_bytes,
        ))
    }

    /// Suballocate a chunk of memory. The resulting allocation is always
    /// aligned to the page size relative to the original allocation's offset.
    ///
    /// # Params
    ///
    /// * size_in_bytes: the required size of the allocation.
    ///
    /// # Safety
    ///
    /// Unsafe because
    /// * The caller must free the returned allocation
    /// * The caller is responsible for synchronizing access (CPU and GPU) to
    ///   the underlying memory
    /// * The returned memory will always be aligned to the page size relative
    ///   to the original allocation's offset.
    unsafe fn allocate_unaligned(
        &mut self,
        size_in_bytes: u64,
    ) -> Result<Allocation, AllocatorError> {
        let page_count =
            div_ceil(size_in_bytes, self.page_size_in_bytes) as usize;
        let starting_index =
            self.arena.allocate_chunk(page_count).with_context(|| {
                "Unable to find a contiguous chunk of the requseted size."
            })?;
        Ok(Allocation::suballocate(
            &self.allocation,
            starting_index as u64 * self.page_size_in_bytes,
            size_in_bytes,
        ))
    }

    /// Free a previously suballocated chunk of memory.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    /// * The caller must not free the same allocation multiple times.
    /// * The caller is responsible for synchronizing access to the underlying
    ///   GPU memory.
    pub unsafe fn free(&mut self, allocation: Allocation) {
        if self.allocation.memory() != allocation.memory() {
            return;
        }
        let relative_offset =
            allocation.offset_in_bytes() - self.allocation.offset_in_bytes();

        // NOTE: it is safe to integer divide and round down here because
        // the page_index can be anywhere in the chunk. e.g. there is no need
        // to consider cases where the offset is aligned to a value larger
        // than the page size - it just works.
        let page_index = relative_offset / self.page_size_in_bytes;
        self.arena.free_chunk(page_index as usize);
    }
}

/// Divide top/bottom, rounding towards positive infinity.
fn div_ceil(top: u64, bottom: u64) -> u64 {
    (top / bottom) + u64::from(top % bottom != 0)
}

#[cfg(test)]
mod test {
    use super::div_ceil;

    #[test]
    fn div_ceil_test() {
        assert_eq!(div_ceil(1, 2), 1);
        assert_eq!(div_ceil(1, 3), 1);
        assert_eq!(div_ceil(1, 4), 1);
        assert_eq!(div_ceil(3, 2), 2);
        assert_eq!(div_ceil(7, 3), 3);
    }
}
