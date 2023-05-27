use {
    crate::{Allocation, AllocationRequirements, AllocatorError},
    std::sync::{Arc, Mutex},
};

/// Move an composable allocator into a Rc RefCell.
pub fn into_shared<T: ComposableAllocator>(allocator: T) -> Arc<Mutex<T>> {
    Arc::new(Mutex::new(allocator))
}

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

impl ComposableAllocator for Box<dyn ComposableAllocator> {
    unsafe fn allocate(
        &mut self,
        allocation_requirements: AllocationRequirements,
    ) -> Result<Allocation, AllocatorError> {
        self.as_mut().allocate(allocation_requirements)
    }

    unsafe fn free(&mut self, allocation: Allocation) {
        self.as_mut().free(allocation)
    }
}

impl<T> ComposableAllocator for Box<T>
where
    T: ComposableAllocator,
{
    unsafe fn allocate(
        &mut self,
        allocation_requirements: AllocationRequirements,
    ) -> Result<Allocation, AllocatorError> {
        self.as_mut().allocate(allocation_requirements)
    }

    unsafe fn free(&mut self, allocation: Allocation) {
        self.as_mut().free(allocation)
    }
}

impl<T> ComposableAllocator for Arc<Mutex<T>>
where
    T: ComposableAllocator,
{
    unsafe fn allocate(
        &mut self,
        allocation_requirements: AllocationRequirements,
    ) -> Result<Allocation, AllocatorError> {
        self.lock().unwrap().allocate(allocation_requirements)
    }

    unsafe fn free(&mut self, allocation: Allocation) {
        self.lock().unwrap().free(allocation)
    }
}
