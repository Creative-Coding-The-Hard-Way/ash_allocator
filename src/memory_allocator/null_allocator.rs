use {
    crate::{
        Allocation, AllocationRequirements, AllocatorError,
        ComposableAllocator, DeviceMemory,
    },
    ash::vk,
};

/// An allocator implementation which takes no actions and simply returns null
/// memory.
///
/// This is useful in unit tests when working with allocators which defer to
/// other allocators.
pub struct NullAllocator;

impl ComposableAllocator for NullAllocator {
    unsafe fn allocate(
        &mut self,
        allocation_requirements: AllocationRequirements,
    ) -> Result<Allocation, AllocatorError> {
        Ok(Allocation::new(
            DeviceMemory::new(vk::DeviceMemory::null()),
            allocation_requirements.memory_type_index,
            0,
            allocation_requirements.size_in_bytes,
        ))
    }

    unsafe fn free(&mut self, _allocation: Allocation) { /* no-op */
    }
}
