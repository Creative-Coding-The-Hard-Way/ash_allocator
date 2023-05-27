use {
    crate::{
        Allocation, AllocationRequirements, AllocatorError,
        ComposableAllocator, MemoryProperties, MemoryTypePoolAllocator,
    },
    std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    },
};

type SharedAllocator<T> = Arc<Mutex<T>>;

pub struct PoolAllocator<A: ComposableAllocator> {
    typed_pools: HashMap<usize, MemoryTypePoolAllocator<SharedAllocator<A>>>,
}

impl<A: ComposableAllocator> PoolAllocator<A> {
    pub fn new(
        memory_properties: MemoryProperties,
        chunk_size: u64,
        page_size: u64,
        allocator: A,
    ) -> Self {
        let allocator = SharedAllocator::new(Mutex::new(allocator));
        let typed_pools = memory_properties
            .types()
            .iter()
            .enumerate()
            .map(|(memory_type_index, _memory_type)| {
                (
                    memory_type_index,
                    MemoryTypePoolAllocator::new(
                        memory_type_index,
                        chunk_size,
                        page_size,
                        allocator.clone(),
                    ),
                )
            })
            .collect::<HashMap<_, _>>();
        Self { typed_pools }
    }
}

impl<A: ComposableAllocator> ComposableAllocator for PoolAllocator<A> {
    unsafe fn allocate(
        &mut self,
        allocation_requirements: AllocationRequirements,
    ) -> Result<Allocation, AllocatorError> {
        let pool = self
            .typed_pools
            .get_mut(&allocation_requirements.memory_type_index)
            .unwrap();
        pool.allocate(allocation_requirements)
    }

    unsafe fn free(&mut self, allocation: Allocation) {
        let pool = self
            .typed_pools
            .get_mut(&allocation.memory_type_index())
            .unwrap();
        pool.free(allocation)
    }
}
