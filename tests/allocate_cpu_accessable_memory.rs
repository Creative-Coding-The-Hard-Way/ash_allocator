use {
    ccthw_ash_allocator::{DeviceAllocator, MemoryAllocator},
    ccthw_ash_instance::VulkanHandle,
};

mod common;
use {anyhow::Result, ash::vk, scopeguard::defer};

unsafe fn create_allocater(
    instance: &ash::Instance,
    device: ash::Device,
    physical_device: vk::PhysicalDevice,
) -> MemoryAllocator {
    let device_allocator = DeviceAllocator::new(device.clone());
    MemoryAllocator::new(instance, device, physical_device, device_allocator)
}

#[test]
pub fn allocate_buffer() -> Result<()> {
    let device = common::setup()?;
    log::info!("{}", device);

    let mut allocator = unsafe {
        create_allocater(
            device.instance.ash(),
            device.device.raw().clone(),
            *device.device.physical_device().raw(),
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
            vk::MemoryPropertyFlags::HOST_VISIBLE
                | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?
    };
    defer! { unsafe { allocator.free_buffer(buffer, allocation) }; }

    log::info!("{:#?}", allocation);

    Ok(())
}

#[test]
pub fn allocate_image() -> Result<()> {
    let device = common::setup()?;
    log::info!("{}", device);

    let mut allocator = unsafe {
        create_allocater(
            device.instance.ash(),
            device.device.raw().clone(),
            *device.device.physical_device().raw(),
        )
    };

    let (image, allocation) = unsafe {
        let create_info = vk::ImageCreateInfo {
            flags: vk::ImageCreateFlags::empty(),
            image_type: vk::ImageType::TYPE_2D,
            format: vk::Format::R8G8B8A8_UINT,
            extent: vk::Extent3D {
                width: 1920,
                height: 1080,
                depth: 1,
            },
            mip_levels: 1,
            array_layers: 1,
            samples: vk::SampleCountFlags::TYPE_1,
            tiling: vk::ImageTiling::LINEAR,
            usage: vk::ImageUsageFlags::TRANSFER_DST,
            initial_layout: vk::ImageLayout::PREINITIALIZED,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: std::ptr::null(),
            ..Default::default()
        };
        allocator.allocate_image(
            &create_info,
            vk::MemoryPropertyFlags::HOST_VISIBLE
                | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?
    };
    defer! { unsafe { allocator.free_image(image, allocation) }; }

    log::info!("Image Memory {}", allocation);

    Ok(())
}
