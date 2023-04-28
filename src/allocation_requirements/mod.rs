use {
    crate::{AllocatorError, PrettyBitflag, PrettySize},
    ash::vk,
};

mod dedicated_resource_handle;

pub use self::dedicated_resource_handle::DedicatedResourceHandle;

/// All supported memory requirements.
///
/// It's convenient to keep the Memory Requirements 2 and Dedicated Requirements
/// structures together because they're populated at the same time.
#[derive(Copy, Clone, Default, PartialEq, Eq)]
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
        memory_property_flags: vk::MemoryPropertyFlags,
        buffer: vk::Buffer,
    ) -> Result<Self, AllocatorError> {
        let mut dedicated_requirements =
            vk::MemoryDedicatedRequirements::default();
        let mut memory_requirements2 = vk::MemoryRequirements2 {
            p_next: &mut dedicated_requirements
                as *mut vk::MemoryDedicatedRequirements
                as *mut std::ffi::c_void,
            ..Default::default()
        };

        unsafe {
            let requirements_info = vk::BufferMemoryRequirementsInfo2 {
                buffer,
                ..Default::default()
            };
            device.get_buffer_memory_requirements2(
                &requirements_info,
                &mut memory_requirements2,
            );
        }

        let memory_type_index = Self::pick_memory_type_index(
            memory_types,
            &memory_requirements2.memory_requirements,
            memory_property_flags,
        )?;
        Ok(Self::from_memory_requirements(
            &dedicated_requirements,
            &memory_requirements2.memory_requirements,
            memory_type_index,
            memory_property_flags,
            DedicatedResourceHandle::Buffer(buffer),
        ))
    }

    /// Get the memory requirements for a given image.
    ///
    /// # Params
    ///
    /// * `device` - the device used to create and interact with GPU resources
    /// * `memory_types` - the memory types available on the physical device
    /// * `memory_properties` - the memory properties required by the allocation
    /// * `image` - the image which needs a memory allocation
    pub fn for_image(
        device: &ash::Device,
        memory_types: &[vk::MemoryType],
        memory_property_flags: vk::MemoryPropertyFlags,
        image: vk::Image,
    ) -> Result<Self, AllocatorError> {
        let mut dedicated_requirements =
            vk::MemoryDedicatedRequirements::default();
        let mut memory_requirements2 = vk::MemoryRequirements2 {
            p_next: &mut dedicated_requirements
                as *mut vk::MemoryDedicatedRequirements
                as *mut std::ffi::c_void,
            ..Default::default()
        };

        unsafe {
            let requirements_info = vk::ImageMemoryRequirementsInfo2 {
                image,
                ..Default::default()
            };
            device.get_image_memory_requirements2(
                &requirements_info,
                &mut memory_requirements2,
            );
        }

        let memory_type_index = Self::pick_memory_type_index(
            memory_types,
            &memory_requirements2.memory_requirements,
            memory_property_flags,
        )?;
        Ok(Self::from_memory_requirements(
            &dedicated_requirements,
            &memory_requirements2.memory_requirements,
            memory_type_index,
            memory_property_flags,
            DedicatedResourceHandle::Image(image),
        ))
    }

    /// Compute the maximum size which must be allocated to ensure an aligned
    /// offset for the resulting memory.
    pub fn aligned_size(&self) -> u64 {
        self.size_in_bytes + self.alignment - 1
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

// Private API
// -----------

impl AllocationRequirements {
    /// Construct the memory requirements struct from raw requirements.
    fn from_memory_requirements(
        dedicated_requirements: &vk::MemoryDedicatedRequirements,
        memory_requirements: &vk::MemoryRequirements,
        memory_type_index: usize,
        memory_property_flags: vk::MemoryPropertyFlags,
        dedicated_resource_handle: DedicatedResourceHandle,
    ) -> Self {
        Self {
            size_in_bytes: memory_requirements.size,
            alignment: memory_requirements.alignment,
            memory_type_bits: memory_requirements.memory_type_bits,
            memory_type_index,
            memory_properties: memory_property_flags,
            prefers_dedicated_allocation: dedicated_requirements
                .prefers_dedicated_allocation
                == vk::TRUE,
            requires_dedicated_allocation: dedicated_requirements
                .requires_dedicated_allocation
                == vk::TRUE,
            dedicated_resource_handle,
        }
    }

    /// Pick the optimal memory type for the given memory requirements and
    /// property flags.
    ///
    /// # Params
    ///
    /// - `memory_types` - a slice of all avialable memory types
    /// - `memory_requirements` - the memory requirements for the resource
    /// - `memory_property_flags` - the required memory properties
    ///
    /// # Returns
    ///
    /// A result containing either the index of the suitable memory type in
    /// `memory_types`, or an [AllocatorError] indicating that no suitable
    /// memory type could be found.
    fn pick_memory_type_index(
        memory_types: &[vk::MemoryType],
        memory_requirements: &vk::MemoryRequirements,
        memory_property_flags: vk::MemoryPropertyFlags,
    ) -> Result<usize, AllocatorError> {
        memory_types
            .iter()
            .enumerate()
            .find(|(index, memory_type)| {
                let type_bits = 1 << index;
                let is_required_type =
                    type_bits & memory_requirements.memory_type_bits != 0;

                let has_required_properties =
                    memory_type.property_flags.contains(memory_property_flags);

                is_required_type && has_required_properties
            })
            .map(|(i, _memory_type)| i)
            .ok_or(AllocatorError::NoSupportedTypeForProperties(
                PrettyBitflag(memory_requirements.memory_type_bits),
                memory_property_flags,
            ))
    }
}
