use ash::vk;

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
