use crate::{Allocation, AllocationRequirements, AllocatorError};

pub trait ComposableAllocator {
    /// Allocate GPU memory based on the given requirements.
    ///
    /// # Safety
    ///
    /// Unsafe because memory must be freed before the device is destroyed.
    unsafe fn allocate(
        &mut self,
        allocation_requirements: AllocationRequirements,
    ) -> Result<Allocation, AllocatorError>;

    /// Return a GPU memory allocation to the device.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///  - memory must be freed by the application before the device is
    ///    destroyed
    ///  - the application is responsible for synchronizing access to device
    ///    memory. It is an error to free memory while ongoing GPU operations
    ///    are still referencing it.
    unsafe fn free(&mut self, allocation: Allocation);
}
