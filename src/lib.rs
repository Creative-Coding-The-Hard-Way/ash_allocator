//! A general purpose Vulkan Memory allocator, written from scratch the hard
//! way.
//!

use ::{ash::vk, thiserror::Error};

#[derive(Debug, Copy, Clone, Error)]
pub enum VulkanAllocatorError {}

type VulkanAllocatorResult<T> = Result<T, VulkanAllocatorError>;

/// An allocated chunk of GPU memory.
#[allow(dead_code)]
pub struct Allocation {
    device_memory: vk::DeviceMemory,
    offset_in_bytes: vk::DeviceSize,
    size_in_bytes: vk::DeviceSize,
    memory_type_index: u32,
    cpu_mapped_ptr: Option<*mut std::ffi::c_void>,
}

/// The interface for composable GPU Memory Allocators.
pub trait VulkanAllocator {
    /// Allocate a block of device memory.
    ///
    /// # Safety
    ///
    /// Unsafe because the caller is responsible for calling free when the
    /// memory is no longer needed.
    unsafe fn allocate(
        &mut self,
        allocate_info: vk::MemoryAllocateInfo,
        alignment: u64,
    ) -> VulkanAllocatorResult<Allocation>;

    /// Free an allocated piece of device memory.
    ///
    /// # Safety
    ///
    /// Unsafe because the caller must ensure that no GPU operations refer to
    /// the allocation.
    unsafe fn free(
        &mut self,
        allocation: &Allocation,
    ) -> VulkanAllocatorResult<()>;
}
