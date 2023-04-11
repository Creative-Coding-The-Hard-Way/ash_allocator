use {
    crate::{pretty_wrappers::PrettySize, AllocatorError},
    anyhow::Context,
    ash::vk,
};

/// A GPU memory allocation.
#[derive(Eq, PartialEq, Copy, Clone)]
pub struct Allocation {
    memory: vk::DeviceMemory,
    offset_in_bytes: vk::DeviceSize,
    size_in_bytes: vk::DeviceSize,
    memory_type_index: usize,
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
        self.memory
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
        let mapped_ptr = device
            .map_memory(
                self.memory,
                self.offset_in_bytes,
                self.size_in_bytes,
                vk::MemoryMapFlags::empty(),
            )
            .with_context(|| "Unable to map a memory allocation!")?;
        Ok(mapped_ptr)
    }

    /// Unmap the allocation.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    /// - The pointer returned by map() must not be used after the call to
    ///   unmap()
    /// - The application must synchronize all host access to the allocation.
    pub unsafe fn unmap(&self, device: &ash::Device) {
        device.unmap_memory(self.memory)
    }
}

impl std::fmt::Debug for Allocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Allocation")
            .field("memory", &self.memory)
            .field("offset_in_bytes", &PrettySize(self.offset_in_bytes))
            .field("size_in_bytes", &PrettySize(self.size_in_bytes))
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
        memory: vk::DeviceMemory,
        memory_type_index: usize,
        offset_in_bytes: vk::DeviceSize,
        size_in_bytes: vk::DeviceSize,
    ) -> Self {
        Self {
            memory,
            memory_type_index,
            offset_in_bytes,
            size_in_bytes,
        }
    }

    /// The index for the memory type used to allocate this chunk of memory.
    pub(crate) fn memory_type_index(&self) -> usize {
        self.memory_type_index
    }
}
