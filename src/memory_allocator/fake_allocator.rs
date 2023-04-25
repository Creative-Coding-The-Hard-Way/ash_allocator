use {
    crate::{
        device_memory::DeviceMemory, Allocation, AllocationRequirements,
        AllocatorError, ComposableAllocator,
    },
    ash::vk,
};

/// A fake implementation of a composable memory allocator which keeps track of
/// all requested memory allocations.
#[derive(Default)]
pub struct FakeAllocator {
    /// An ordered collection of every allocation made with this allocator.
    pub allocations: Vec<AllocationRequirements>,

    /// The number of allocations which have yet to be freed.
    pub active_allocations: u32,
}

impl ComposableAllocator for FakeAllocator {
    unsafe fn allocate(
        &mut self,
        allocation_requirements: AllocationRequirements,
    ) -> Result<Allocation, AllocatorError> {
        self.active_allocations += 1;
        self.allocations.push(allocation_requirements);

        Ok(Allocation::new(
            DeviceMemory::new(vk::DeviceMemory::null()),
            allocation_requirements.memory_type_index,
            0,
            allocation_requirements.size_in_bytes,
        ))
    }

    unsafe fn free(&mut self, _allocation: Allocation) {
        self.active_allocations -= 1;
    }
}
