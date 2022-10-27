use {
    ccthw_ash_allocator::{DeviceAllocator, MemoryAllocator},
    ccthw_ash_instance::VulkanHandle,
};

mod common;
use {anyhow::Result, ash::vk, scopeguard::defer};

#[test]
pub fn allocate_some_memory() -> Result<()> {
    let device = common::setup()?;
    log::info!("{}", device);

    let mut allocator = unsafe {
        let device_allocator =
            DeviceAllocator::new(device.device.raw().clone());
        MemoryAllocator::new(
            device.instance.ash(),
            device.device.raw().clone(),
            *device.device.physical_device().raw(),
            device_allocator,
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
            create_info,
            vk::MemoryPropertyFlags::HOST_VISIBLE
                | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?
    };
    defer! { unsafe { allocator.free_buffer(buffer, allocation) }; }

    log::info!("{:#?}", allocation);

    Ok(())
}
