//! Tests where memory for images and buffers is allocated and freed.

use {
    anyhow::Result, ash::vk, ccthw_ash_allocator::create_system_allocator,
    ccthw_ash_instance::VulkanHandle, scopeguard::defer,
};

mod common;

#[test]
pub fn allocate_buffer() -> Result<()> {
    let device = common::setup()?;
    log::info!("{}", device);

    let mut allocator = unsafe {
        create_system_allocator(
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
        create_system_allocator(
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
