//! Tests where memory for images and buffers is allocated and freed.

use {
    anyhow::Result,
    ash::vk,
    ccthw_ash_allocator::{
        into_shared, DedicatedAllocator, DeviceAllocator, MemoryAllocator,
        MemoryProperties, PoolAllocator, SizedAllocator, TraceAllocator,
    },
    ccthw_ash_instance::VulkanHandle,
    scopeguard::defer,
};

mod common;

unsafe fn create_allocator(
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

    let root_chunk_size = 1024 * 1024 * 1024; // kb -> mb -> gb
    let root_page_size = root_chunk_size / 128;

    let large_chunk_pool_allocator = into_shared(PoolAllocator::new(
        memory_properties.clone(),
        root_chunk_size,
        root_page_size,
        device_allocator.clone(),
    ));

    let small_chunk_size = root_page_size;
    let small_page_size = small_chunk_size / 256;

    let small_chunk_pool_allocator = PoolAllocator::new(
        memory_properties,
        small_chunk_size,
        small_page_size,
        large_chunk_pool_allocator.clone(),
    );

    let sized_allocator = SizedAllocator::new(
        root_chunk_size,
        SizedAllocator::new(
            small_chunk_size,
            small_chunk_pool_allocator,
            large_chunk_pool_allocator,
        ),
        device_allocator.clone(),
    );

    let dedicated_allocator =
        DedicatedAllocator::new(sized_allocator, device_allocator);

    MemoryAllocator::new(instance, device, physical_device, dedicated_allocator)
}

#[test]
pub fn allocate_buffer() -> Result<()> {
    let device = common::setup()?;
    log::info!("{}", device);

    let mut allocator = unsafe {
        create_allocator(
            device.instance.ash(),
            device.logical_device.raw().clone(),
            *device.logical_device.physical_device().raw(),
        )
    };

    let (buffer, allocation) = unsafe {
        let create_info = vk::BufferCreateInfo {
            flags: vk::BufferCreateFlags::empty(),
            usage: vk::BufferUsageFlags::STORAGE_BUFFER,
            size: 64_000,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: std::ptr::null(),
            ..Default::default()
        };
        allocator.allocate_buffer(
            &create_info,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?
    };
    defer! { unsafe { allocator.free_buffer(buffer, allocation.clone()) }; }

    log::info!("{:#?}", &allocation);

    Ok(())
}

#[test]
pub fn allocate_image() -> Result<()> {
    let device = common::setup()?;
    log::info!("{}", device);

    let mut allocator = unsafe {
        create_allocator(
            device.instance.ash(),
            device.logical_device.raw().clone(),
            *device.logical_device.physical_device().raw(),
        )
    };

    let (image, allocation) = unsafe {
        let create_info = vk::ImageCreateInfo {
            flags: vk::ImageCreateFlags::empty(),
            image_type: vk::ImageType::TYPE_2D,
            format: vk::Format::R8G8B8A8_UINT,
            extent: vk::Extent3D {
                width: 3840,
                height: 2160,
                depth: 1,
            },
            mip_levels: 1,
            array_layers: 1,
            samples: vk::SampleCountFlags::TYPE_1,
            tiling: vk::ImageTiling::OPTIMAL,
            usage: vk::ImageUsageFlags::TRANSFER_DST,
            initial_layout: vk::ImageLayout::PREINITIALIZED,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: std::ptr::null(),
            ..Default::default()
        };
        allocator.allocate_image(
            &create_info,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?
    };
    defer! { unsafe { allocator.free_image(image, allocation.clone()) }; }

    log::info!("Image Memory {}", &allocation);

    Ok(())
}
