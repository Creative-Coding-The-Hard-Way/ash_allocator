use {
    crate::{
        pretty_wrappers::PrettySize, AllocationRequirements, AllocatorError,
        DeviceMemory,
    },
    ash::vk,
};

/// A GPU memory allocation.
#[derive(Clone)]
pub struct Allocation {
    device_memory: DeviceMemory,
    offset_in_bytes: vk::DeviceSize,
    size_in_bytes: vk::DeviceSize,
    memory_type_index: usize,
    allocation_requirements: AllocationRequirements,
}

// Public API
// ----------

impl Allocation {
    /// The underlying Vulkan memory handle.
    ///
    /// # Safety
    ///
    /// Unsafe because the allocation logically owns the device memory. It is
    /// incorrect to free the memory by any means other than to return the
    /// full allocation instance to the memory allocator.
    pub unsafe fn memory(&self) -> vk::DeviceMemory {
        self.device_memory.memory()
    }

    /// The offset where this allocation begins in device memory.
    ///
    /// This is needed because some memory allocator implementations will
    /// subdivide big regions of GPU memory into smaller allocations. Therefore
    /// the actual device memory handle can be shared by many allocations.
    pub fn offset_in_bytes(&self) -> vk::DeviceSize {
        self.offset_in_bytes
    }

    /// The size of the allocation in bytes.
    pub fn size_in_bytes(&self) -> vk::DeviceSize {
        self.size_in_bytes
    }

    /// The allocation requirements used when acquiring the device memory.
    pub fn allocation_requirements(&self) -> &AllocationRequirements {
        &self.allocation_requirements
    }

    /// Map the allocation into application address space.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    /// - The application must synchronize access to the underlying device
    ///   memory. All previously submitted GPU commands which write to the
    ///   memory owned by this alloctaion must be finished before the host reads
    ///   or writes from the mapped pointer.
    /// - Synchronization requirements vary depending on the HOST_COHERENT
    ///   memory property. See the Vulkan spec for details.
    ///
    /// For details, see the specification at:
    /// https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkMapMemory.html
    pub unsafe fn map(
        &self,
        device: &ash::Device,
    ) -> Result<*mut std::ffi::c_void, AllocatorError> {
        // Get the ptr to the start of the device memory
        let base_ptr = self.device_memory.map(device)?;
        let base_ptr_address = base_ptr as usize;

        // compute the address for this allocation
        let with_offset = base_ptr_address + self.offset_in_bytes() as usize;

        Ok(with_offset as *mut std::ffi::c_void)
    }

    /// Unmap the allocation.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    /// - The pointer returned by map() must not be used after the call to
    ///   unmap()
    /// - The application must synchronize all host access to the allocation.
    pub unsafe fn unmap(
        &self,
        device: &ash::Device,
    ) -> Result<(), AllocatorError> {
        self.device_memory.unmap(device)
    }
}

impl std::fmt::Debug for Allocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Allocation")
            .field("device_memory", &self.device_memory)
            .field("offset_in_bytes", &PrettySize(self.offset_in_bytes))
            .field("size_in_bytes", &PrettySize(self.size_in_bytes))
            .field("allocation_requirements", &self.allocation_requirements)
            .finish()
    }
}

impl std::fmt::Display for Allocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:#?}", self))
    }
}

// Private API
// -----------

impl Allocation {
    /// Create a new memory allocation.
    pub(crate) fn new(
        device_memory: DeviceMemory,
        memory_type_index: usize,
        offset_in_bytes: vk::DeviceSize,
        size_in_bytes: vk::DeviceSize,
        allocation_requirements: AllocationRequirements,
    ) -> Self {
        Self {
            device_memory,
            memory_type_index,
            offset_in_bytes,
            size_in_bytes,
            allocation_requirements,
        }
    }

    /// Create an allocation which refers to the same underlying device memory.
    ///
    /// # Params
    ///
    /// * allocation: The original memory allocation which will be subdivided.
    /// * offset: The offset relative to the original allocation's offset.
    /// * size_in_bytes: The size of the suballocation.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    /// * This constructor only checks that the suballocation fits within the
    ///   original allocation. There is nothing to prevent aliasing. The caller
    ///   must have their own strategy for tracking the original allocation's
    ///   usage and synchronizing access.
    /// * Freeing the device memory will make interacting with any Allocation
    ///   struct behave badly. The caller must have some strategy for tracking
    ///   suballocations and ensuring they're all cleaned up before allowing the
    ///   original allocation to be freed.
    pub(crate) unsafe fn suballocate(
        allocation: &Allocation,
        offset: vk::DeviceSize,
        size_in_bytes: vk::DeviceSize,
        offset_alignment: u64,
    ) -> Self {
        let full_offset = allocation.offset_in_bytes() + offset;
        assert!(
            full_offset + size_in_bytes
                <= allocation.offset_in_bytes() + allocation.size_in_bytes(),
            "Attempted to suballocate outside of an allocation's bounds!"
        );
        assert!(
            full_offset % offset_alignment == 0,
            "Attempted to suballocate with invalid alignment!"
        );
        Self {
            device_memory: allocation.device_memory.clone(),
            offset_in_bytes: full_offset,
            size_in_bytes,
            memory_type_index: allocation.memory_type_index(),
            allocation_requirements: AllocationRequirements {
                size_in_bytes,
                alignment: offset_alignment,
                ..allocation.allocation_requirements
            },
        }
    }

    /// The index for the memory type used to allocate this chunk of memory.
    pub(crate) fn memory_type_index(&self) -> usize {
        self.memory_type_index
    }
}
