use crate::{
    Allocation, AllocationRequirements, AllocatorError, ComposableAllocator,
};

/// An allocator which composes over two other allocators. When a request is
/// below the trigger size, it is sent to the first alloctor, otherwise it is
/// sent to the second allocator.
pub struct SizedAllocator<
    SmallAllocator: ComposableAllocator,
    LargeAllocator: ComposableAllocator,
> {
    size_trigger: u64,
    small_allocator: SmallAllocator,
    large_allocator: LargeAllocator,
}

impl<S, L> SizedAllocator<S, L>
where
    S: ComposableAllocator,
    L: ComposableAllocator,
{
    /// Create a new allocator which routes requests based on the allocation
    /// size.
    pub fn new(
        size_trigger: u64,
        small_allocator: S,
        large_allocator: L,
    ) -> Self {
        Self {
            size_trigger,
            small_allocator,
            large_allocator,
        }
    }
}

impl<S, L> ComposableAllocator for SizedAllocator<S, L>
where
    S: ComposableAllocator,
    L: ComposableAllocator,
{
    unsafe fn allocate(
        &mut self,
        allocation_requirements: AllocationRequirements,
    ) -> Result<Allocation, AllocatorError> {
        if allocation_requirements.aligned_size() < self.size_trigger {
            self.small_allocator.allocate(allocation_requirements)
        } else {
            self.large_allocator.allocate(allocation_requirements)
        }
    }

    unsafe fn free(&mut self, allocation: Allocation) {
        if allocation.allocation_requirements().aligned_size()
            < self.size_trigger
        {
            self.small_allocator.free(allocation)
        } else {
            self.large_allocator.free(allocation)
        }
    }
}
