use {ccthw_ash_instance::VulkanHandle, std::collections::HashMap};

mod common;
use {anyhow::Result, ash::vk, indoc::indoc, scopeguard::defer};

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

    let requirements_info = vk::BufferMemoryRequirementsInfo2 {
        buffer,
        ..Default::default()
    };
    let mut dedicated_requirements = vk::MemoryDedicatedRequirements::default();
    let mut memory_requirements = vk::MemoryRequirements2 {
        p_next: &mut dedicated_requirements
            as *mut vk::MemoryDedicatedRequirements
            as *mut std::ffi::c_void,
        memory_requirements: vk::MemoryRequirements::default(),
        ..Default::default()
    };
    unsafe {
        device.get_buffer_memory_requirements2(
            &requirements_info,
            &mut memory_requirements,
        );
    }

    log::info!(
        indoc!(
            "
            {:#?}
            [{:#?}] {:#?}"
        ),
        memory_requirements,
        &dedicated_requirements as *const _,
        dedicated_requirements,
    );

    #[derive(Debug, Clone)]
    struct MemoryProperties {
        types: Vec<vk::MemoryType>,
        heaps: Vec<vk::MemoryHeap>,
    }

    impl MemoryProperties {
        fn new(
            instance: &ash::Instance,
            physical_device: vk::PhysicalDevice,
        ) -> Self {
            let properties = unsafe {
                instance.get_physical_device_memory_properties(physical_device)
            };
            let mut types =
                Vec::with_capacity(properties.memory_type_count as usize);
            types.extend_from_slice(
                &properties.memory_types
                    [0..properties.memory_type_count as usize],
            );
            let mut heaps =
                Vec::with_capacity(properties.memory_heap_count as usize);
            heaps.extend_from_slice(
                &properties.memory_heaps
                    [0..properties.memory_heap_count as usize],
            );
            Self { types, heaps }
        }
    }

    impl std::fmt::Display for MemoryProperties {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("# Memory Properties\n\n")?;
            f.write_str("## Memory Types\n\n")?;

            for (index, memory_type) in self.types.iter().enumerate() {
                f.write_fmt(format_args!(
                    indoc!(
                        "
                        [{}] property_flags: {:#?}
                                heap_index: {}

                        "
                    ),
                    index, memory_type.property_flags, memory_type.heap_index,
                ))?;
            }

            f.write_str("\n## Memory Heaps\n\n")?;

            let mut table = HashMap::new();
            table.insert(0, "b");
            table.insert(1, "kb");
            table.insert(2, "mb");
            table.insert(3, "gb");
            table.insert(4, "pb");

            for (index, heap) in self.heaps.iter().enumerate() {
                let pow = (heap.size as f32).log(1024.0).floor() as u32;
                let div = heap.size as f32 / 1024.0_f32.powf(pow as f32);
                let str = table.get(&pow).unwrap();
                f.write_fmt(format_args!(
                    indoc!(
                        "
                        [{}] flags: {:#?}
                             size: {:.2}{}

                        "
                    ),
                    index, heap.flags, div, str
                ))?;
            }

            Ok(())
        }
    }

    let props = MemoryProperties::new(device.instance.ash(), unsafe {
        *device.device.physical_device().raw()
    });

    log::info!("{}", props);

    //let mut table = HashMap::new();
    //table.insert(0, "b");
    //table.insert(1, "kb");
    //table.insert(2, "mb");
    //table.insert(3, "gb");
    //table.insert(4, "pb");
    //for i in 0..properties.memory_heap_count {
    //    let size = properties.memory_heaps[i as usize].size;
    //    let pow = (size as f32).log(1024.0).floor() as u32;
    //    let div = size as f32 / 1024.0_f32.powf(pow as f32);
    //    let str = table.get(&pow).unwrap();
    //    log::info!("Heap {} size {:.2} {}", i, div, str);
    //    log::info!("{:#?}", properties.memory_types[i as usize]);
    //}

    //device.instance.ash().
    // get_physical_device_memory_properties2(physical_device, out)

    Ok(())
}
