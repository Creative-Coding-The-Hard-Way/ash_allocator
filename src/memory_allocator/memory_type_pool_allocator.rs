use {
    crate::{
        Allocation, AllocationRequirements, AllocatorError,
        ComposableAllocator, PageSuballocator,
    },
    anyhow::anyhow,
    ash::vk,
    std::collections::HashMap,
};

pub struct MemoryTypePoolAllocator<Allocator: ComposableAllocator> {
    memory_type_index: usize,
    allocator: Allocator,
    pool: HashMap<vk::DeviceMemory, PageSuballocator>,
}

impl<Allocator: ComposableAllocator> MemoryTypePoolAllocator<Allocator> {
    /// Create a new pool for a particular memory type index.
    ///
    /// # Params
    ///
    /// * memory_type_index: the index of the specific memory type this pool can
    ///   allocate from.
    /// * allocator: the backing allocator which provides device memory.
    pub fn new(memory_type_index: usize, allocator: Allocator) -> Self {
        Self {
            memory_type_index,
            allocator,
            pool: HashMap::new(),
        }
    }
}

impl<Allocator: ComposableAllocator> ComposableAllocator
    for MemoryTypePoolAllocator<Allocator>
{
    unsafe fn allocate(
        &mut self,
        allocation_requirements: AllocationRequirements,
    ) -> Result<Allocation, AllocatorError> {
        if self.memory_type_index != allocation_requirements.memory_type_index {
            return Err(AllocatorError::RuntimeError(anyhow!(
                "Memory type index mismatch"
            )));
        }

        todo!()
    }

    unsafe fn free(&mut self, allocation: Allocation) {
        todo!()
    }
}
