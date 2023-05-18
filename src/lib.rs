//! A general purpose Vulkan Memory allocator, written from scratch the hard
//! way.

mod allocation;
mod allocation_requirements;
mod device_memory;
mod error;
mod memory_allocator;
mod memory_properties;
mod pretty_wrappers;

use {
    self::{
        allocation::AllocationId,
        device_memory::DeviceMemory,
        pretty_wrappers::{PrettyBitflag, PrettySize},
    },
    ash::vk,
};

pub use self::{
    allocation::Allocation,
    allocation_requirements::{
        AllocationRequirements, DedicatedResourceHandle,
    },
    error::AllocatorError,
    memory_allocator::{
        into_shared, ComposableAllocator, DedicatedAllocator, DeviceAllocator,
        FakeAllocator, MemoryAllocator, MemoryTypePoolAllocator,
        PageSuballocator, PoolAllocator, SizedAllocator, TraceAllocator,
    },
    memory_properties::MemoryProperties,
};

/// Create an opinionated system allocator for GPU memoy.
///
/// # Safety
///
/// Unsafe because:
/// - The application must keep the device alive for as long as the allocator is
///   alive.
/// - The application must free any memory it allocates prior to dropping the
///   memory allocator or device.
pub unsafe fn create_system_allocator(
    instance: &ash::Instance,
    device: ash::Device,
    physical_device: vk::PhysicalDevice,
) -> MemoryAllocator {
    let memory_properties = MemoryProperties::new(instance, physical_device);

    let device_allocator = into_shared(TraceAllocator::new(
        instance,
        physical_device,
        DeviceAllocator::new(device.clone()),
        "Device Allocator",
    ));

    let small_page_size = 1024; // 1kb
    let small_chunk_size = small_page_size * 64; // 64kb
    let medium_page_size = small_chunk_size; // 64kb
    let medium_chunk_size = medium_page_size * 64; // 4mb
    let root_page_size = medium_chunk_size; // 4mb
    let root_chunk_size = medium_chunk_size * 128; // 0.5gb

    let large_chunk_pool_allocator = into_shared(SizedAllocator::new(
        root_chunk_size,
        PoolAllocator::new(
            memory_properties.clone(),
            root_chunk_size,
            root_page_size,
            device_allocator.clone(),
        ),
        device_allocator.clone(),
    ));

    let medium_chunk_pool_allocator = into_shared(SizedAllocator::new(
        medium_chunk_size,
        PoolAllocator::new(
            memory_properties.clone(),
            medium_chunk_size,
            medium_page_size,
            large_chunk_pool_allocator.clone(),
        ),
        large_chunk_pool_allocator,
    ));

    let small_chunk_pool_allocator = SizedAllocator::new(
        small_chunk_size,
        PoolAllocator::new(
            memory_properties,
            small_chunk_size,
            small_page_size,
            medium_chunk_pool_allocator.clone(),
        ),
        medium_chunk_pool_allocator,
    );

    let dedicated_allocator =
        DedicatedAllocator::new(small_chunk_pool_allocator, device_allocator);

    let system_allocator = TraceAllocator::new(
        instance,
        physical_device,
        dedicated_allocator,
        "Application Allocator",
    );

    MemoryAllocator::new(instance, device, physical_device, system_allocator)
}
