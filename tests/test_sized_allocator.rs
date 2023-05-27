//! Tests for the sized allocator.

use {
    anyhow::Result,
    ccthw_ash_allocator::{
        into_shared, AllocationRequirements, ComposableAllocator,
        FakeAllocator, SizedAllocator,
    },
};

mod common;

#[test]
fn test_small_allocation() -> Result<()> {
    common::setup_logger();

    let small_allocator = into_shared(FakeAllocator::default());
    let large_allocator = into_shared(FakeAllocator::default());
    let mut allocator = SizedAllocator::new(
        64,
        small_allocator.clone(),
        large_allocator.clone(),
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
    assert_eq!(small_allocator.lock().unwrap().active_allocations, 1);
    assert_eq!(large_allocator.lock().unwrap().active_allocations, 0);

    unsafe {
        allocator.free(allocation);
    }

    assert_eq!(small_allocator.lock().unwrap().active_allocations, 0);
    assert_eq!(large_allocator.lock().unwrap().active_allocations, 0);

    Ok(())
}

#[test]
fn test_large_allocation() -> Result<()> {
    common::setup_logger();

    let small_allocator = into_shared(FakeAllocator::default());
    let large_allocator = into_shared(FakeAllocator::default());
    let mut allocator = SizedAllocator::new(
        64,
        small_allocator.clone(),
        large_allocator.clone(),
    );

    let allocation = unsafe {
        let allocation_requirements = AllocationRequirements {
            size_in_bytes: 62,
            alignment: 8,
            ..AllocationRequirements::default()
        };
        allocator.allocate(allocation_requirements)?
    };
    assert_eq!(allocation.size_in_bytes(), 62);
    assert_eq!(small_allocator.lock().unwrap().active_allocations, 0);
    assert_eq!(large_allocator.lock().unwrap().active_allocations, 1);

    unsafe {
        allocator.free(allocation);
    }

    assert_eq!(small_allocator.lock().unwrap().active_allocations, 0);
    assert_eq!(large_allocator.lock().unwrap().active_allocations, 0);

    Ok(())
}
