//! Tests for creating a CPU accessible buffer, then confirming that data can
//! be written and read from the buffer.

use {
    ccthw_ash_allocator::{DeviceAllocator, MemoryAllocator, TraceAllocator},
    ccthw_ash_instance::VulkanHandle,
};

mod common;
use {anyhow::Result, ash::vk, scopeguard::defer};

unsafe fn create_allocator(
    instance: &ash::Instance,
    device: ash::Device,
    physical_device: vk::PhysicalDevice,
) -> MemoryAllocator {
    let device_allocator = DeviceAllocator::new(device.clone());
    let trace_allocator = TraceAllocator::new(
        instance,
        physical_device,
        device_allocator,
        "Device Allocator",
    );
    MemoryAllocator::new(instance, device, physical_device, trace_allocator)
}

#[repr(C, packed)]
struct ExampleData {
    pub value: i32,
}

#[test]
pub fn test_mapped_buffer() -> Result<()> {
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
            size: std::mem::size_of::<ExampleData>() as u64,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: std::ptr::null(),
            ..Default::default()
        };
        allocator.allocate_buffer(
            &create_info,
            vk::MemoryPropertyFlags::HOST_VISIBLE
                | vk::MemoryPropertyFlags::HOST_COHERENT,
            std::mem::size_of::<ExampleData>(),
        )?
    };
    defer! { unsafe { allocator.free_buffer(buffer, allocation.clone()) }; }

    // Map the memory and write a value into it. Then unmap the memory.
    {
        let ptr = unsafe { allocation.map(device.logical_device.raw())? };
        let addr = ptr as usize;

        // The other option would be to create a stack-allocated ExampleData and
        // perform an unaligned write/read
        assert_eq!(addr % std::mem::align_of::<ExampleData>(), 0);

        let sliced = unsafe {
            // SAFE because we assert that the pointer is aligned properly
            std::slice::from_raw_parts_mut(ptr as *mut ExampleData, 1)
        };

        sliced[0].value = 1337;

        unsafe {
            allocation.unmap(device.logical_device.raw())?;
        }
    }

    // Map the memory and verify that the written value is present
    {
        let ptr = unsafe { allocation.map(device.logical_device.raw())? };
        let addr = ptr as usize;

        // The other option would be to create a stack-allocated ExampleData and
        // perform an unaligned write/read
        assert_eq!(addr % std::mem::align_of::<ExampleData>(), 0);

        let sliced = unsafe {
            // SAFE because we assert that the pointer is aligned properly
            std::slice::from_raw_parts_mut(ptr as *mut ExampleData, 1)
        };

        let value = sliced[0].value;
        assert_eq!(value, 1337);

        unsafe {
            allocation.unmap(device.logical_device.raw())?;
        }
    }

    Ok(())
}

#[test]
pub fn test_repeated_mapping() -> Result<()> {
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
            size: std::mem::size_of::<ExampleData>() as u64,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: std::ptr::null(),
            ..Default::default()
        };
        allocator.allocate_buffer(
            &create_info,
            vk::MemoryPropertyFlags::HOST_VISIBLE
                | vk::MemoryPropertyFlags::HOST_COHERENT,
            std::mem::size_of::<ExampleData>(),
        )?
    };
    defer! { unsafe { allocator.free_buffer(buffer, allocation.clone()) } };

    log::info!("Allocation before mapping: {}", &allocation);
    let ptr_a = unsafe { allocation.map(device.logical_device.raw())? };
    log::info!("Allocation after one mapping: {}", &allocation);
    let ptr_b = unsafe { allocation.map(device.logical_device.raw())? };
    log::info!("Allocation after both mappings: {}", &allocation);

    assert_eq!(ptr_a, ptr_b);

    unsafe {
        allocation.unmap(device.logical_device.raw())?;
        allocation.unmap(device.logical_device.raw())?;
    }
    log::info!("Allocation after unmapping: {}", &allocation);

    Ok(())
}
