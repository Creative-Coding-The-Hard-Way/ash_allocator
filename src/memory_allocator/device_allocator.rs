use {
    crate::{
        Allocation, AllocationRequirements, AllocatorError,
        ComposableAllocator, DeviceMemory,
    },
    anyhow::Context,
    ash::vk,
};

/// A GPU memory allocator which always allocates memory directly from the
/// device.
pub struct DeviceAllocator {
    device: ash::Device,
}

impl DeviceAllocator {
    /// Create a new device allocator.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///  - the device must not be destroyed while this allocater still exists
    ///  - all memory allocated by this allocator must be freed before
    ///    destroying the device
    pub unsafe fn new(device: ash::Device) -> Self {
        Self { device }
    }
}

impl ComposableAllocator for DeviceAllocator {
    unsafe fn allocate(
        &mut self,
        allocation_requirements: AllocationRequirements,
    ) -> Result<Allocation, AllocatorError> {
        let dedicated_info = allocation_requirements
            .dedicated_resource_handle
            .as_dedicated_allocation_info();
        let create_info = vk::MemoryAllocateInfo {
            p_next: &dedicated_info as *const vk::MemoryDedicatedAllocateInfo
                as *const std::ffi::c_void,
            allocation_size: allocation_requirements.size_in_bytes,
            memory_type_index: allocation_requirements.memory_type_index as u32,
            ..Default::default()
        };
        let memory = self
            .device
            .allocate_memory(&create_info, None)
            .with_context(|| {
                format!(
                    "Error allocating memory with requirements {}",
                    allocation_requirements,
                )
            })?;
        let allocation = Allocation::new(
            DeviceMemory::new(memory),
            allocation_requirements.memory_type_index,
            0,
            allocation_requirements.size_in_bytes,
            allocation_requirements,
        );
        Ok(allocation)
    }

    unsafe fn free(&mut self, allocation: Allocation) {
        self.device.free_memory(allocation.memory(), None)
    }
}
