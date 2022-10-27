mod composable_allocator;
mod device_allocator;

use {
    crate::{
        allocation::Allocation, AllocationRequirements, AllocatorError,
        MemoryProperties,
    },
    anyhow::Context,
    ash::vk,
};

pub use self::{
    composable_allocator::ComposableAllocator,
    device_allocator::DeviceAllocator,
};

pub struct MemoryAllocator<T: ComposableAllocator> {
    internal_allocator: T,
    memory_properties: MemoryProperties,
    device: ash::Device,
}

impl<T: ComposableAllocator> MemoryAllocator<T> {
    /// Create a new memory allocator.
    ///
    /// # Safety
    ///
    /// Unsafe because the ash device must not be destroyed while the allocater
    /// still exists.
    pub unsafe fn new(
        instance: &ash::Instance,
        device: ash::Device,
        physical_device: vk::PhysicalDevice,
        internal_allocator: T,
    ) -> Self {
        let memory_properties =
            MemoryProperties::new(instance, physical_device);
        log::trace!(
            "Memory allocator for device with memory properties\n{}",
            memory_properties
        );
        Self {
            internal_allocator,
            memory_properties,
            device,
        }
    }

    /// Allocate a buffer and the memory it requires.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - the buffer and memory must be freed before the device is destroyed
    pub unsafe fn allocate_buffer(
        &mut self,
        buffer_create_info: vk::BufferCreateInfo,
        memory_property_flags: vk::MemoryPropertyFlags,
    ) -> Result<(vk::Buffer, Allocation), AllocatorError> {
        let buffer = unsafe {
            self.device
                .create_buffer(&buffer_create_info, None)
                .with_context(|| {
                    format!(
                        "Error creating a buffer with {:#?}",
                        buffer_create_info
                    )
                })?
        };

        let requirements = AllocationRequirements::for_buffer(
            &self.device,
            self.memory_properties.types(),
            memory_property_flags,
            buffer,
        )?;

        let allocation =
            unsafe { self.internal_allocator.allocate(requirements)? };

        unsafe {
            self.device
                .bind_buffer_memory(
                    buffer,
                    allocation.memory(),
                    allocation.offset_in_bytes(),
                )
                .context("Error binding buffer memory")?;
        }

        Ok((buffer, allocation))
    }

    /// Free a buffer and the associated allocated memory.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - the application must synchronize access to the buffer and its memory
    ///   - it is an error to free a buffer while ongoing GPU operations still
    ///     reference it
    ///   - it is an error to use the buffer handle after calling this method
    pub unsafe fn free_buffer(
        &mut self,
        buffer: vk::Buffer,
        allocation: Allocation,
    ) {
        self.device.destroy_buffer(buffer, None);
        self.internal_allocator.free(allocation);
    }
}
