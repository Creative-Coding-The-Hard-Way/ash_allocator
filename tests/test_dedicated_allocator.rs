//! Tests for the dedicated allocator.

use {
    anyhow::Result,
    ccthw_ash_allocator::{
        into_shared, AllocationRequirements, ComposableAllocator,
        DedicatedAllocator, FakeAllocator,
    },
};

mod common;

#[test]
fn test_non_dedicated_allocation() -> Result<()> {
    common::setup_logger();

    let shared_allocator = into_shared(FakeAllocator::default());
    let device_allocator = into_shared(FakeAllocator::default());
    let mut allocator = DedicatedAllocator::new(
        shared_allocator.clone(),
        device_allocator.clone(),
    );

    let allocation = unsafe {
        let allocation_requirements = AllocationRequirements {
            size_in_bytes: 32,
            alignment: 8,
            ..AllocationRequirements::default()
        };
        allocator.allocate(allocation_requirements)?
    };
    assert_eq!(allocation.size_in_bytes(), 32);
    assert_eq!(shared_allocator.borrow().active_allocations, 1);
    assert_eq!(device_allocator.borrow().active_allocations, 0);

    unsafe {
        allocator.free(allocation);
    }

    assert_eq!(shared_allocator.borrow().active_allocations, 0);
    assert_eq!(device_allocator.borrow().active_allocations, 0);

    Ok(())
}

#[test]
fn test_prefers_dedicated_allocation() -> Result<()> {
    common::setup_logger();

    let shared_allocator = into_shared(FakeAllocator::default());
    let device_allocator = into_shared(FakeAllocator::default());
    let mut allocator = DedicatedAllocator::new(
        shared_allocator.clone(),
        device_allocator.clone(),
    );

    let allocation = unsafe {
        let allocation_requirements = AllocationRequirements {
            size_in_bytes: 32,
            alignment: 8,
            prefers_dedicated_allocation: true,
            ..AllocationRequirements::default()
        };
        allocator.allocate(allocation_requirements)?
    };
    assert_eq!(allocation.size_in_bytes(), 32);
    assert_eq!(shared_allocator.borrow().active_allocations, 0);
    assert_eq!(device_allocator.borrow().active_allocations, 1);

    unsafe {
        allocator.free(allocation);
    }

    assert_eq!(shared_allocator.borrow().active_allocations, 0);
    assert_eq!(device_allocator.borrow().active_allocations, 0);

    Ok(())
}

#[test]
fn test_requires_dedicated_allocation() -> Result<()> {
    common::setup_logger();

    let shared_allocator = into_shared(FakeAllocator::default());
    let device_allocator = into_shared(FakeAllocator::default());
    let mut allocator = DedicatedAllocator::new(
        shared_allocator.clone(),
        device_allocator.clone(),
    );

    let allocation = unsafe {
        let allocation_requirements = AllocationRequirements {
            size_in_bytes: 32,
            alignment: 8,
            requires_dedicated_allocation: true,
            ..AllocationRequirements::default()
        };
        allocator.allocate(allocation_requirements)?
    };
    assert_eq!(allocation.size_in_bytes(), 32);
    assert_eq!(shared_allocator.borrow().active_allocations, 0);
    assert_eq!(device_allocator.borrow().active_allocations, 1);

    unsafe {
        allocator.free(allocation);
    }

    assert_eq!(shared_allocator.borrow().active_allocations, 0);
    assert_eq!(device_allocator.borrow().active_allocations, 0);

    Ok(())
}
