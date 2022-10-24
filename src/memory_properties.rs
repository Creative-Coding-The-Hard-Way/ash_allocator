use {crate::PrettySize, ash::vk, indoc::indoc};

#[derive(Debug, Clone)]
pub struct MemoryProperties {
    types: Vec<vk::MemoryType>,
    heaps: Vec<vk::MemoryHeap>,
}

impl MemoryProperties {
    /// Get the memory properties for the given physical device.
    pub fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
    ) -> Self {
        let properties = unsafe {
            instance.get_physical_device_memory_properties(physical_device)
        };
        let mut types =
            Vec::with_capacity(properties.memory_type_count as usize);
        types.extend_from_slice(
            &properties.memory_types[0..properties.memory_type_count as usize],
        );
        let mut heaps =
            Vec::with_capacity(properties.memory_heap_count as usize);
        heaps.extend_from_slice(
            &properties.memory_heaps[0..properties.memory_heap_count as usize],
        );
        Self { types, heaps }
    }

    /// All of the currently usable memory heaps on this system.
    pub fn heaps(&self) -> &[vk::MemoryHeap] {
        &self.heaps
    }

    /// All of the currently usable memory types on this system.
    pub fn types(&self) -> &[vk::MemoryType] {
        &self.types
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

        for (index, heap) in self.heaps.iter().enumerate() {
            f.write_fmt(format_args!(
                indoc!(
                    "
                        [{}] flags: {:#?}
                             size: {}

                        "
                ),
                index,
                heap.flags,
                PrettySize(heap.size),
            ))?;
        }

        Ok(())
    }
}
