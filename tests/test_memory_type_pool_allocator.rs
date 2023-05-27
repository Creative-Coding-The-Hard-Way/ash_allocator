use {
    anyhow::Result,
    ccthw_ash_allocator::{
        into_shared, AllocationRequirements, AllocatorError,
        ComposableAllocator, FakeAllocator, MemoryTypePoolAllocator,
    },
    pretty_assertions::assert_eq,
};

mod common;

#[test]
pub fn test_multiple_allocations() -> Result<()> {
    common::setup_logger();

    let fake = into_shared(FakeAllocator::default());
    let mut allocator = MemoryTypePoolAllocator::new(0, 512, 8, fake.clone());

    let small_allocation_requirements = AllocationRequirements {
        memory_type_index: 0,
        size_in_bytes: 64,
        alignment: 2,
        ..AllocationRequirements::default()
    };
    let allocation_1 = {
        let result =
            unsafe { allocator.allocate(small_allocation_requirements) };
        assert!(result.is_ok());
        result.unwrap()
    };
    let allocation_2 = {
        let result =
            unsafe { allocator.allocate(small_allocation_requirements) };
        assert!(result.is_ok());
        result.unwrap()
    };
    let big_allocation_requirements = AllocationRequirements {
        memory_type_index: 0,
        size_in_bytes: 512 - 64 - 32,
        alignment: 32,
        ..AllocationRequirements::default()
    };
    let allocation_3 = {
        let result = unsafe { allocator.allocate(big_allocation_requirements) };
        assert!(result.is_ok());
        result.unwrap()
    };

    assert_eq!(fake.lock().unwrap().active_allocations, 2);
    assert_eq!(
        fake.lock().unwrap().allocations,
        &[
            AllocationRequirements {
                size_in_bytes: 512,
                alignment: 1,
                ..AllocationRequirements::default()
            },
            AllocationRequirements {
                size_in_bytes: 512,
                alignment: 1,
                ..AllocationRequirements::default()
            },
        ]
    );

    unsafe {
        allocator.free(allocation_1);
        allocator.free(allocation_2);
        allocator.free(allocation_3);
    };

    assert_eq!(fake.lock().unwrap().active_allocations, 0);

    Ok(())
}

#[test]
pub fn test_allocate_and_free() -> Result<()> {
    common::setup_logger();

    let fake = into_shared(FakeAllocator::default());
    let mut allocator = MemoryTypePoolAllocator::new(0, 512, 8, fake.clone());

    let allocation_requirements = AllocationRequirements {
        memory_type_index: 0,
        size_in_bytes: 64,
        alignment: 128,
        ..AllocationRequirements::default()
    };
    let allocation = {
        let result = unsafe { allocator.allocate(allocation_requirements) };
        assert!(result.is_ok());
        result.unwrap()
    };

    assert_eq!(
        fake.lock().unwrap().allocations[0],
        AllocationRequirements {
            size_in_bytes: 512,
            alignment: 1,
            ..allocation_requirements
        }
    );
    assert_eq!(fake.lock().unwrap().active_allocations, 1);

    unsafe { allocator.free(allocation) };

    assert_eq!(fake.lock().unwrap().active_allocations, 0);

    Ok(())
}

#[test]
pub fn test_allocate_with_mismatching_type_index_should_fail() -> Result<()> {
    common::setup_logger();

    let fake = into_shared(FakeAllocator::default());
    let mut allocator = MemoryTypePoolAllocator::new(0, 32, 1, fake);

    let allocation_requirements = AllocationRequirements {
        memory_type_index: 1,
        ..AllocationRequirements::default()
    };

    let result = unsafe { allocator.allocate(allocation_requirements) };

    assert!(result.is_err());
    match result.err().unwrap() {
        AllocatorError::RuntimeError(error) => {
            assert_eq!(format!("{error}"), "Memory type index mismatch");
        }
        _ => panic!("Result must be an error!"),
    };

    Ok(())
}

#[test]
pub fn test_allocate_with_oversized_allocation_requirements() -> Result<()> {
    common::setup_logger();

    let fake = into_shared(FakeAllocator::default());
    let mut allocator = MemoryTypePoolAllocator::new(0, 64, 1, fake);

    let allocation_requirements = AllocationRequirements {
        memory_type_index: 0,
        size_in_bytes: 64,
        alignment: 2,
        ..AllocationRequirements::default()
    };

    let result = unsafe { allocator.allocate(allocation_requirements) };

    assert!(result.is_err());
    match result.err().unwrap() {
        AllocatorError::RuntimeError(error) => {
            assert_eq!(
                format!("{error}"),
                "Unable to allocate a chunk of memory with 64 bytes"
            );
        }
        _ => panic!("Result must be an error!"),
    };

    Ok(())
}
