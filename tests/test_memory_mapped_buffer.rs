//! Tests for creating a CPU accessible buffer, then confirming that data can
//! be written and read from the buffer.

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

    let system_allocator = TraceAllocator::new(
        instance,
        physical_device,
        dedicated_allocator,
        "Application Allocator",
    );

    MemoryAllocator::new(instance, device, physical_device, system_allocator)
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
