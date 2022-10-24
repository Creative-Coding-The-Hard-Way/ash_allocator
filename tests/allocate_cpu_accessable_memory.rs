use {
    ccthw_ash_allocator::{AllocationRequirements, MemoryProperties},
    ccthw_ash_instance::VulkanHandle,
};

mod common;
use {anyhow::Result, ash::vk, scopeguard::defer};

#[test]
pub fn allocate_some_memory() -> Result<()> {
    let device = common::setup()?;
    log::info!("{}", device);

    let buffer = unsafe {
        let create_info = vk::BufferCreateInfo {
            flags: vk::BufferCreateFlags::empty(),
            usage: vk::BufferUsageFlags::STORAGE_BUFFER,
            size: 64_000,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: std::ptr::null(),
            ..Default::default()
        };
        device.create_buffer(&create_info, None)?
    };
    defer! { unsafe { device.destroy_buffer(buffer, None); } }

    let props = MemoryProperties::new(device.instance.ash(), unsafe {
        *device.device.physical_device().raw()
    });

    log::info!("{}", props);

    let requirements = AllocationRequirements::for_buffer(
        &device,
        props.types(),
        vk::MemoryPropertyFlags::HOST_VISIBLE
            | vk::MemoryPropertyFlags::HOST_COHERENT,
        buffer,
    )?;

    log::info!("{}", requirements);

    let dedicated_info = vk::MemoryDedicatedAllocateInfo {
        buffer: vk::Buffer::null(),
        image: vk::Image::null(),
        ..Default::default()
    };
    let allocate_info = vk::MemoryAllocateInfo {
        p_next: &dedicated_info as *const _ as *const std::ffi::c_void,
        allocation_size: requirements.size_in_bytes,
        memory_type_index: requirements.memory_type_index as u32,
        ..Default::default()
    };
    let memory = unsafe { device.allocate_memory(&allocate_info, None)? };
    defer! { unsafe { device.free_memory(memory, None) }; }

    Ok(())
}
