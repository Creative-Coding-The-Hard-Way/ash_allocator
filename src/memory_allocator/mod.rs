mod composable_allocator;
mod device_allocator;
mod fake_allocator;
mod memory_type_pool_allocator;
mod page_suballocator;
mod trace_allocator;

use {
    crate::{
        allocation::Allocation, AllocationRequirements, AllocatorError,
        MemoryProperties,
    },
    anyhow::Context,
    ash::vk,
};

pub use self::{
    composable_allocator::{into_shared, ComposableAllocator},
    device_allocator::DeviceAllocator,
    fake_allocator::FakeAllocator,
    memory_type_pool_allocator::MemoryTypePoolAllocator,
    page_suballocator::PageSuballocator,
    trace_allocator::TraceAllocator,
};

/// The top-level interface for allocating GPU memory.
///
/// The memory allocator owns a composable allocator instance which actually
/// does the work of memory allocation. This allows the behavior to be
/// customized by composing allocators.
pub struct MemoryAllocator {
    internal_allocator: Box<dyn ComposableAllocator>,
    memory_properties: MemoryProperties,
    device: ash::Device,
}

impl MemoryAllocator {
    /// Create a new memory allocator.
    ///
    /// # Params
    ///
    /// * `instance` - the ash Instance is used te query the physical device's
    ///   memory properties
    /// * `device` - the logical device is used to create and destroy Vulkan
    ///   resources
    /// * `physical_device` - the backing physical device being controlled by
    ///   the logical device
    /// * `internal_allocator` - the actual ComposableAllocator implementation
    ///   which is responsible for allocating memory
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///  - the logical device must not be destroyed while the MemoryAllocator is
    ///    still in use
    pub unsafe fn new<T: ComposableAllocator + 'static>(
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
            internal_allocator: Box::new(internal_allocator),
            memory_properties,
            device,
        }
    }

    /// Allocate a buffer and memory.
    ///
    /// # Params
    ///
    /// - `buffer_create_info` - used to create the Buffer and determine what
    ///   memory it needs
    /// - `memory_property_flags` - used to pick the correct memory type for the
    ///   buffer's memory
    ///
    /// # Returns
    ///
    /// A tuple of `(vk::buffer, Allocation)` which contains the raw vulkan
    /// buffer and the backing memory Allocation.
    ///
    /// The buffer is already bound to the memory in the allocation so the
    /// buffer is ready to use immediately.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - the buffer and memory must be freed before the device is destroyed
    pub unsafe fn allocate_buffer(
        &mut self,
        buffer_create_info: &vk::BufferCreateInfo,
        memory_property_flags: vk::MemoryPropertyFlags,
    ) -> Result<(vk::Buffer, Allocation), AllocatorError> {
        let buffer = unsafe {
            self.device
                .create_buffer(buffer_create_info, None)
                .with_context(|| {
                    format!(
                        "Error creating a buffer with {:#?}",
                        buffer_create_info
                    )
                })?
        };

        let requirements = {
            let result = AllocationRequirements::for_buffer(
                &self.device,
                self.memory_properties.types(),
                memory_property_flags,
                buffer,
            );
            if result.is_err() {
                self.device.destroy_buffer(buffer, None);
            }
            result?
        };

        let allocation = {
            let result =
                unsafe { self.internal_allocator.allocate(requirements) };
            if result.is_err() {
                self.device.destroy_buffer(buffer, None);
            }
            result?
        };

        unsafe {
            let result = self
                .device
                .bind_buffer_memory(
                    buffer,
                    allocation.memory(),
                    allocation.offset_in_bytes(),
                )
                .context("Error binding buffer memory");
            if result.is_err() {
                self.device.destroy_buffer(buffer, None);
            }
            result?;
        }

        Ok((buffer, allocation))
    }

    /// Allocate an Image and memory.
    ///
    /// # Params
    ///
    /// - `image_create_info` - used to create the Buffer and determine what
    ///   memory it needs
    /// - `memory_property_flags` - used to pick the correct memory type for the
    ///   buffer's memory
    ///
    /// # Returns
    ///
    /// A tuple of `(vk::Image, Allocation)` which contains the raw Vulkan
    /// image and the backing memory Allocation.
    ///
    /// The image is already bound to the memory in the allocation so the
    /// image is ready to use immediately.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - the image and memory must be freed before the device is destroyed
    pub unsafe fn allocate_image(
        &mut self,
        image_create_info: &vk::ImageCreateInfo,
        memory_property_flags: vk::MemoryPropertyFlags,
    ) -> Result<(vk::Image, Allocation), AllocatorError> {
        let image = unsafe {
            self.device
                .create_image(image_create_info, None)
                .with_context(|| {
                    format!(
                        "Error creating a image with {:#?}",
                        image_create_info
                    )
                })?
        };

        let requirements = {
            let result = AllocationRequirements::for_image(
                &self.device,
                self.memory_properties.types(),
                memory_property_flags,
                image,
            );
            if result.is_err() {
                self.device.destroy_image(image, None);
            }
            result?
        };

        let allocation = {
            let result =
                unsafe { self.internal_allocator.allocate(requirements) };
            if result.is_err() {
                self.device.destroy_image(image, None);
            }
            result?
        };

        unsafe {
            let result = self
                .device
                .bind_image_memory(
                    image,
                    allocation.memory(),
                    allocation.offset_in_bytes(),
                )
                .context("Error image buffer memory");
            if result.is_err() {
                self.device.destroy_image(image, None);
            }
            result?;
        }

        Ok((image, allocation))
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

    /// Free an image and the associated allocated memory.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - the application must synchronize access to the image and its memory
    ///   - it is an error to free an image while ongoing GPU operations still
    ///     reference it
    ///   - it is an error to use the image handle after calling this method
    pub unsafe fn free_image(
        &mut self,
        image: vk::Image,
        allocation: Allocation,
    ) {
        self.device.destroy_image(image, None);
        self.internal_allocator.free(allocation);
    }
}
