//! Tests for the paged_suballocator. The big idea is to allocate a big chunk
//! of device memory, then suballocate it, write to the suballocations, then
//! verify the results.

use {
    anyhow::Result,
    ash::vk,
    ccthw_ash_allocator::{
        Allocation, DeviceAllocator, MemoryAllocator, PageSuballocator,
        TraceAllocator,
    },
    ccthw_ash_instance::VulkanHandle,
    scopeguard::defer,
    std::mem::align_of,
};

mod common;

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

fn mapped_slice<'a, T>(
    allocation: &'a Allocation,
    device: &common::TestDevice,
) -> Result<&'a mut [T]>
where
    T: Sized,
{
    let ptr = unsafe { allocation.map(device.logical_device.raw())? };
    let addr = ptr as usize;

    // The other option would be to create a stack-allocated ExampleData and
    // perform an unaligned write/read
    assert_eq!(addr % std::mem::align_of::<T>(), 0);

    let slice_length =
        allocation.size_in_bytes() as usize / std::mem::size_of::<T>();

    let sliced = unsafe {
        // SAFE because we assert that the pointer is aligned properly
        std::slice::from_raw_parts_mut(ptr as *mut T, slice_length)
    };
    Ok(sliced)
}

#[test]
pub fn test_paged_suballocator() -> Result<()> {
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
            size: std::mem::size_of::<u32>() as u64 * 100,
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

    {
        // Fill the entire allocation with 0s.
        let slice = mapped_slice::<u32>(&allocation, &device)?;
        for item in slice {
            *item = 0;
        }
    }

    let mut suballocator =
        PageSuballocator::for_allocation(allocation.clone(), 1);

    // Allocate memory from the original allocation
    // --------------------------------------------

    let u32_alignment = align_of::<u32>() as u64;
    let suballocation_1 = unsafe {
        suballocator
            .allocate(std::mem::size_of::<u32>() as u64 * 20, u32_alignment)?
    };
    assert_eq!(
        suballocation_1.size_in_bytes(),
        std::mem::size_of::<u32>() as u64 * 20
    );

    let suballocation_2 = unsafe {
        suballocator
            .allocate(std::mem::size_of::<u32>() as u64 * 60, u32_alignment)?
    };
    assert_eq!(
        suballocation_2.size_in_bytes(),
        std::mem::size_of::<u32>() as u64 * 60
    );

    let suballocation_3 = unsafe {
        suballocator
            .allocate(std::mem::size_of::<u32>() as u64 * 17, u32_alignment)?
    };
    assert_eq!(
        suballocation_3.size_in_bytes(),
        std::mem::size_of::<u32>() as u64 * 17
    );

    let try_4 = unsafe { suballocator.allocate(10, u32_alignment) };
    assert!(try_4.is_err());

    // Map the suballocations and write to them
    // ----------------------------------------

    {
        let slice = mapped_slice(&suballocation_1, &device)?;
        for item in slice {
            *item = 1;
        }
    }

    {
        let slice = mapped_slice(&suballocation_2, &device)?;
        for item in slice {
            *item = 2;
        }
    }

    {
        let slice = mapped_slice(&suballocation_3, &device)?;
        for item in slice {
            *item = 3;
        }
    }

    // Verify that the correct regions in the original allocation were
    // written.
    // ---------------------------------------------------------------

    {
        let slice = mapped_slice::<u32>(&allocation, &device)?;
        for (i, &v) in slice.iter().enumerate() {
            if i < 20 {
                assert_eq!(v, 1, "slice at {i}");
            } else if i == 20 {
                assert_eq!(v, 0, "slice at {i}");
            } else if (21..81).contains(&i) {
                assert_eq!(v, 2, "slice at {i}");
            } else if i == 81 {
                assert_eq!(v, 0, "slice at {i}");
            } else if (82..99).contains(&i) {
                assert_eq!(v, 3, "slice at {i}");
            } else if i == 99 {
                assert_eq!(v, 0, "slice at {i}");
            }
        }
    }

    unsafe {
        suballocator.free(suballocation_1);
        suballocator.free(suballocation_2);
        suballocator.free(suballocation_3);
    }

    assert!(suballocator.is_empty());

    Ok(())
}
