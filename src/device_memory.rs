use {
    crate::AllocatorError,
    anyhow::Context,
    ash::vk,
    std::{
        ffi::c_void,
        fmt::Debug,
        sync::{Arc, Mutex},
    },
};

/// A representation of Vulkan device memory which gracefully handles multiple
/// calls to vkMapMemory.
#[derive(Clone)]
pub struct DeviceMemory {
    memory: vk::DeviceMemory,
    shared_mapped_ptr: Arc<Mutex<MappedPtr>>,
}

// Public Api
// ----------

impl DeviceMemory {
    /// Create a new DeviceMemory instance.
    pub fn new(memory: vk::DeviceMemory) -> Self {
        Self {
            memory,
            shared_mapped_ptr: Arc::default(),
        }
    }

    /// The underlying Vulkan memory handle.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    /// - Ownership of the Vulkan memory is not transferred, the caller must not
    ///   retain a copy of the vk::DeviceMemory handle after this instance is
    ///   dropped.
    pub unsafe fn memory(&self) -> vk::DeviceMemory {
        self.memory
    }

    /// Get a memory-mapped ptr to the beginning of the device memory
    /// allocation. The entire region of memory is always mapped.
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
        let mut lock = self.shared_mapped_ptr.lock().unwrap();
        if lock.map_count == 0 {
            lock.host_accessible_ptr = device
                .map_memory(
                    self.memory,
                    0,
                    vk::WHOLE_SIZE,
                    vk::MemoryMapFlags::empty(),
                )
                .with_context(|| "Unable to map a memory allocation!")?;
        }
        lock.map_count += 1;
        Ok(lock.host_accessible_ptr)
    }

    /// Unmap a the device memory.
    ///
    /// This can be called multiple times until no memory is mapped anymore.
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
        let mut lock = self.shared_mapped_ptr.lock().unwrap();
        if lock.map_count == 0 {
            return Err(AllocatorError::RuntimeError(anyhow::anyhow!(
                "Attemped to unmap memory which has no mapping!"
            )));
        } else if lock.map_count == 1 {
            device.unmap_memory(self.memory);
            lock.host_accessible_ptr = std::ptr::null_mut();
        }
        lock.map_count -= 1;
        Ok(())
    }
}

impl Debug for DeviceMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lock = self.shared_mapped_ptr.lock().unwrap();
        let map_count = lock.map_count;
        let host_accessible_ptr = lock.host_accessible_ptr;

        f.debug_struct("DeviceMemory")
            .field("memory", &self.memory)
            .field("map_count", &map_count)
            .field("host_accessible_ptr", &host_accessible_ptr)
            .finish()
    }
}

/// Any given piece of Vulkan device memory can have a CPU accessable ptr -
/// assuming it was created with the HOST_VISIBLE flag.
///
/// But, multiple allocations CAN share the same underlying device memory.
/// Calling "vkMapMemory" multiple times on a single piece of Device Memory is
/// an error in Vulkan.
///
/// The Big Idea here is to have a shared object to hold a CPU accessable
/// pointer for the vk::DeviceMemory object.
struct MappedPtr {
    host_accessible_ptr: *mut c_void,
    map_count: u32,
}

impl Default for MappedPtr {
    fn default() -> Self {
        Self {
            host_accessible_ptr: std::ptr::null_mut(),
            map_count: 0,
        }
    }
}
