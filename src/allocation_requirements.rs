use {
    crate::{AllocatorError, PrettyBitflag, PrettySize},
    ash::vk,
};

/// A copy of the resource handle associated with an allocation.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DedicatedResourceHandle {
    Buffer(vk::Buffer),
    Image(vk::Image),
    None,
}

impl DedicatedResourceHandle {
    /// Get a memory dedicated allocation info struct based on the current
    /// resource. Nulls are used for missing values and the generated result
    /// can be used for allocation.
    pub fn as_dedicated_allocation_info(
        &self,
    ) -> vk::MemoryDedicatedAllocateInfo {
        let mut dedicated_allocate_info =
            vk::MemoryDedicatedAllocateInfo::default();
        match self {
            DedicatedResourceHandle::Buffer(buffer) => {
                dedicated_allocate_info.buffer = *buffer;
            }
            DedicatedResourceHandle::Image(image) => {
                dedicated_allocate_info.image = *image;
            }
            DedicatedResourceHandle::None => (),
        }
        dedicated_allocate_info
    }
}

impl Default for DedicatedResourceHandle {
    fn default() -> Self {
        Self::None
    }
}

/// All supported memory requirements.
///
/// It's convenient to keep the Memory Requirements 2 and Dedicated Requirements
/// structures together because they're populated at the same time.
#[derive(Copy, Clone, Default)]
pub struct AllocationRequirements {
    pub size_in_bytes: u64,
    pub alignment: u64,
    pub memory_type_bits: u32,
    pub memory_type_index: usize,
    pub memory_properties: vk::MemoryPropertyFlags,
    pub prefers_dedicated_allocation: bool,
    pub requires_dedicated_allocation: bool,
    pub dedicated_resource_handle: DedicatedResourceHandle,
}

// Public API
// ----------

impl AllocationRequirements {
    /// Get the memory requirements for a given buffer.
    ///
    /// # Params
    ///
    /// * `device` - the device used to create and interact with GPU resources
    /// * `memory_types` - the memory types available on the physical device
    /// * `memory_properties` - the memory properties required by the allocation
    /// * `buffer` - the buffer which needs a memory allocation
    pub fn for_buffer(
        device: &ash::Device,
        memory_types: &[vk::MemoryType],
        memory_properties: vk::MemoryPropertyFlags,
        buffer: vk::Buffer,
    ) -> Result<Self, AllocatorError> {
        unsafe {
            let mut dedicated_requirements =
                vk::MemoryDedicatedRequirements::default();
            let mut memory_requirements2 = vk::MemoryRequirements2 {
                p_next: &mut dedicated_requirements
                    as *mut vk::MemoryDedicatedRequirements
                    as *mut std::ffi::c_void,
                ..Default::default()
            };

            let requirements_info = vk::BufferMemoryRequirementsInfo2 {
                buffer,
                ..Default::default()
            };
            device.get_buffer_memory_requirements2(
                &requirements_info,
                &mut memory_requirements2,
            );

            let index = memory_types
                .iter()
                .enumerate()
                .find(|(index, memory_type)| {
                    let type_bits = 1 << index;
                    let is_required_type = type_bits
                        & memory_requirements2
                            .memory_requirements
                            .memory_type_bits
                        != 0;

                    let has_required_properties =
                        memory_type.property_flags.contains(memory_properties);

                    is_required_type && has_required_properties
                })
                .map(|(i, _memory_type)| i)
                .ok_or(AllocatorError::NoSupportedTypeForProperties(
                    PrettyBitflag(
                        memory_requirements2
                            .memory_requirements
                            .memory_type_bits,
                    ),
                    memory_properties,
                ))?;

            Ok(Self {
                size_in_bytes: memory_requirements2.memory_requirements.size,
                alignment: memory_requirements2.memory_requirements.alignment,
                memory_type_bits: memory_requirements2
                    .memory_requirements
                    .memory_type_bits,
                memory_type_index: index,
                memory_properties,
                prefers_dedicated_allocation: dedicated_requirements
                    .prefers_dedicated_allocation
                    == vk::TRUE,
                requires_dedicated_allocation: dedicated_requirements
                    .requires_dedicated_allocation
                    == vk::TRUE,
                dedicated_resource_handle: DedicatedResourceHandle::Buffer(
                    buffer,
                ),
            })
        }
    }
}

impl std::fmt::Debug for AllocationRequirements {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AllocationRequirements")
            .field("size_in_bytes", &PrettySize(self.size_in_bytes))
            .field("alignment", &self.alignment)
            .field("memory_type_bits", &PrettyBitflag(self.memory_type_bits))
            .field("memory_type_index", &self.memory_type_index)
            .field(
                "prefers_dedicated_allocation",
                &self.prefers_dedicated_allocation,
            )
            .field(
                "requires_dedicated_allocation",
                &self.requires_dedicated_allocation,
            )
            .finish()
    }
}

impl std::fmt::Display for AllocationRequirements {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:#?}", self))
    }
}
