use {
    crate::{
        Allocation, AllocationId, AllocationRequirements, AllocatorError,
        ComposableAllocator, PageSuballocator,
    },
    anyhow::anyhow,
    std::collections::HashMap,
};

pub struct MemoryTypePoolAllocator<Allocator: ComposableAllocator> {
    memory_type_index: usize,
    allocator: Allocator,
    chunk_size: u64,
    page_size: u64,
    pool: HashMap<AllocationId, PageSuballocator>,
}

impl<Allocator: ComposableAllocator> MemoryTypePoolAllocator<Allocator> {
    /// Create a new pool for a particular memory type index.
    ///
    /// # Params
    ///
    /// * memory_type_index: the index of the specific memory type this pool can
    ///   allocate from.
    /// * chunk_size: the size of each chunk of memory to be divided into pages.
    /// * page_size: chunks are divided into pages with this size for
    ///   allocation.
    /// * allocator: the backing allocator which provides device memory.
    pub fn new(
        memory_type_index: usize,
        chunk_size: u64,
        page_size: u64,
        allocator: Allocator,
    ) -> Self {
        debug_assert!(
            chunk_size % page_size == 0,
            "Chunks must be evenly divisible into pages."
        );
        Self {
            memory_type_index,
            allocator,
            chunk_size,
            page_size,
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

        if allocation_requirements.aligned_size() >= self.chunk_size {
            return Err(AllocatorError::RuntimeError(anyhow!(
                "Unable to allocate a chunk of memory with {} bytes",
                allocation_requirements.size_in_bytes
            )));
        }

        // Attempt to allocate from an existing chunk
        for suballocator in self.pool.values_mut() {
            if let Ok(allocation) = suballocator.allocate(
                allocation_requirements.size_in_bytes,
                allocation_requirements.alignment,
            ) {
                return Ok(allocation);
            }
        }

        // Unable to allocate from an existing chunk, so create a new chunk
        // and allocate from it.
        let chunk_requirements = AllocationRequirements {
            alignment: 1,
            size_in_bytes: self.chunk_size,
            memory_type_index: self.memory_type_index,
            ..allocation_requirements
        };
        let chunk_allocation = self.allocator.allocate(chunk_requirements)?;
        let chunk_allocation_id = chunk_allocation.id();
        let mut suballocator =
            PageSuballocator::for_allocation(chunk_allocation, self.page_size);

        // Allocate using the newly created suballocator. Remember to
        // free the chunk if something goes wrong at this point.
        let allocation = match suballocator.allocate(
            allocation_requirements.size_in_bytes,
            allocation_requirements.alignment,
        ) {
            Ok(allocation) => allocation,
            Err(err) => {
                self.allocator.free(suballocator.release_allocation());
                return Err(err);
            }
        };

        debug_assert!(allocation.parent_id().unwrap() == chunk_allocation_id);
        debug_assert!(!self.pool.contains_key(&chunk_allocation_id));
        self.pool.insert(chunk_allocation_id, suballocator);

        Ok(allocation)
    }

    unsafe fn free(&mut self, allocation: Allocation) {
        debug_assert!(
            allocation.parent_id().is_some(),
            "MemoryTypePoolAllocator can only free suballocated allocations!"
        );
        debug_assert!(
            self.pool.contains_key(&allocation.parent_id().unwrap()),
            "The allocation does not come from this MemoryTypePoolAllocator!"
        );

        let key = allocation.parent_id().unwrap();
        let suballocator = self.pool.get_mut(&key).unwrap();
        suballocator.free(allocation);

        if suballocator.is_empty() {
            let chunk_mem =
                self.pool.remove(&key).unwrap().release_allocation();
            self.allocator.free(chunk_mem);
        }
    }
}
