use crate::{
    Allocation, AllocationRequirements, AllocatorError, ComposableAllocator,
};

/// An allocator which correctly handles allocations which prefer or require
/// dedicated allocations.
pub struct DedicatedAllocator<A: ComposableAllocator, B: ComposableAllocator> {
    allocator: A,
    device_allocator: B,
}

impl<A, B> DedicatedAllocator<A, B>
where
    A: ComposableAllocator,
    B: ComposableAllocator,
{
    /// Create a new dedicated allocator which decorates another allocator
    /// implementation to properly handle dedicated resources.
    ///
    /// # Param
    ///
    /// - allocator: The allocator to decorate.
    /// - device_allocator: An allocator which directly returns memory from the
    ///   device itself.
    pub fn new(allocator: A, device_allocator: B) -> Self {
        Self {
            allocator,
            device_allocator,
        }
    }
}

impl<A, B> ComposableAllocator for DedicatedAllocator<A, B>
where
    A: ComposableAllocator,
    B: ComposableAllocator,
{
    unsafe fn allocate(
        &mut self,
        allocation_requirements: AllocationRequirements,
    ) -> Result<Allocation, AllocatorError> {
        if allocation_requirements.prefers_dedicated_allocation
            || allocation_requirements.requires_dedicated_allocation
        {
            self.device_allocator.allocate(allocation_requirements)
        } else {
            self.allocator.allocate(allocation_requirements)
        }
    }

    unsafe fn free(&mut self, allocation: Allocation) {
        let allocation_requirements = allocation.allocation_requirements();
        if allocation_requirements.prefers_dedicated_allocation
            || allocation_requirements.requires_dedicated_allocation
        {
            self.device_allocator.free(allocation)
        } else {
            self.allocator.free(allocation)
        }
    }
}
