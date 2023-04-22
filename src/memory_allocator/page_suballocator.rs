//! An allocator which allocates chunks from an existing allocation.

use {
    crate::{Allocation, AllocatorError},
    anyhow::Context,
};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Page {
    Free,
    Allocated,
}

pub struct PageSuballocator {
    allocation: Allocation,
    page_size_in_bytes: u64,
    arena: Vec<Page>,
}

impl PageSuballocator {
    /// Create an allocator which takes memory from an existing allocation.
    ///
    /// # Params
    ///
    /// * allocation: The allocation to use for suballocations.
    /// * page_count: How many pages the allocation should be divided into. The
    ///   trade-off is that more pages increases the allocation time while fewer
    ///   pages increases the amount of wasted memory.
    ///
    /// # Panic
    ///
    /// Panics if allocation.size_in_bytes is not a multiple of page_count.
    pub fn for_allocation(allocation: Allocation, page_count: u64) -> Self {
        assert!(
            allocation.size_in_bytes() % page_count == 0,
            "PageCount must be a multiple of the allocation size"
        );
        Self {
            page_size_in_bytes: allocation.size_in_bytes() / page_count,
            arena: vec![Page::Free; page_count as usize],
            allocation,
        }
    }

    /// Suballocate a region of memory without considering alignment.
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
    pub unsafe fn allocate_unaligned(
        &mut self,
        size_in_bytes: u64,
    ) -> Result<Allocation, AllocatorError> {
        let page_count =
            div_ceil(size_in_bytes, self.page_size_in_bytes) as usize;
        let start =
            linear_probe(&self.arena, page_count).with_context(|| {
                "Unable to allocate {page_count} contiguous pages."
            })?;
        set_region(&mut self.arena, Page::Allocated, start, page_count);

        Ok(Allocation::suballocate(
            &self.allocation,
            self.page_size_in_bytes * start as u64,
            size_in_bytes,
        ))
    }

    /// Free a previously suballocated chunk of memory.
    pub unsafe fn free(&mut self, allocation: Allocation) {
        if self.allocation.memory() != allocation.memory() {
            return;
        }

        let relative_offset =
            allocation.offset_in_bytes() - self.allocation.offset_in_bytes();

        assert!(
            relative_offset % self.page_size_in_bytes == 0,
            "The relative offset must always be a multiple of page size."
        );

        let start = relative_offset / self.page_size_in_bytes;
        let page_count =
            div_ceil(allocation.size_in_bytes(), self.page_size_in_bytes)
                as usize;
        set_region(&mut self.arena, Page::Free, start as usize, page_count);
    }
}

/// Divide top/bottom, rounding towards positive infinity.
fn div_ceil(top: u64, bottom: u64) -> u64 {
    (top / bottom) + u64::from(top % bottom != 0)
}

/// Update an arena to either free or allocate a contiguous region of pages.
///
/// # Params
///
/// * arena: The set of all pages being considered.
/// * value: The value to set pages to.
/// * start: The index of the first page in the region to set.
/// * size: The size of the region to set.
fn set_region(arena: &mut [Page], value: Page, start: usize, size: usize) {
    assert!(start + size <= arena.len());
    for bit in arena.iter_mut().skip(start).take(size) {
        *bit = value;
    }
}

/// Find the index of the first contiguous free region that is large enough
/// to fit the requested size.
///
/// # Params
///
/// * arena: A set of free and allocated pages to search.
/// * page_count: The number of contiguous free pages being requested.
///
/// # Returns
///
/// * Some(index): The index of the first free page which has at least
///   page_count free pages after it.
/// * None: When there isn't enough space.
fn linear_probe(arena: &[Page], page_count: usize) -> Option<usize> {
    let mut in_region = false;
    let mut start: usize = 0;
    for (index, &value) in arena.iter().enumerate() {
        if value == Page::Free {
            if !in_region {
                start = index;
                in_region = true;
            }

            if in_region && (index - start) == (page_count - 1) {
                return Some(start);
            }
        } else if in_region {
            in_region = false;
            start = 0;
        }
    }
    None
}

#[cfg(test)]
mod test {
    use super::{div_ceil, linear_probe, set_region, Page};

    #[test]
    fn test_linear_probe() {
        use Page::{Allocated as A, Free as F};
        assert_eq!(linear_probe(&[F, F, F, F, F], 2), Some(0));
        assert_eq!(linear_probe(&[A, F, F, F, F], 2), Some(1));
        assert_eq!(linear_probe(&[A, A, F, F, F], 2), Some(2));
        assert_eq!(linear_probe(&[A, A, A, F, F], 2), Some(3));
        assert_eq!(linear_probe(&[A, A, A, A, F], 2), None);
        assert_eq!(linear_probe(&[A, F, F, A, F, F], 2), Some(1));
        assert_eq!(linear_probe(&[A, A, F, F, A, F, F, F, A], 3), Some(5));
        assert_eq!(linear_probe(&[A, A, A, F, A, F, F, F, A], 1), Some(3));
        assert_eq!(linear_probe(&[A, A, A, A, A, F, F, A], 1,), Some(5));
    }

    #[test]
    fn test_set_region() {
        use Page::{Allocated as A, Free as F};
        let mut arena = [F, F, F, F, F];

        set_region(&mut arena, A, 2, 2);
        assert_eq!(arena, [F, F, A, A, F]);

        set_region(&mut arena, A, 4, 1);
        assert_eq!(arena, [F, F, A, A, A]);

        set_region(&mut arena, F, 4, 0);
        assert_eq!(arena, [F, F, A, A, A]);

        set_region(&mut arena, F, 2, 1);
        assert_eq!(arena, [F, F, F, A, A]);
    }

    #[test]
    #[should_panic]
    fn test_set_region_panics_when_starts_outside_range() {
        use Page::{Allocated as A, Free as F};
        let mut bitmap = [F, F, F, F, F];
        set_region(&mut bitmap, A, 8, 1);
    }

    #[test]
    #[should_panic]
    fn test_set_region_panics_when_ends_outside_range() {
        use Page::{Allocated as A, Free as F};
        let mut bitmap = [F, F, F, F, F];
        set_region(&mut bitmap, A, 2, 19);
    }

    #[test]
    fn div_ceil_test() {
        assert_eq!(div_ceil(1, 2), 1);
        assert_eq!(div_ceil(1, 3), 1);
        assert_eq!(div_ceil(1, 4), 1);
        assert_eq!(div_ceil(3, 2), 2);
        assert_eq!(div_ceil(7, 3), 3);
    }
}
