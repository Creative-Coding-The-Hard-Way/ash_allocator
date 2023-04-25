use {
    ccthw_ash_allocator::{
        into_shared, AllocationRequirements, AllocatorError,
        ComposableAllocator, MemoryTypePoolAllocator,
    },
    pretty_assertions::assert_eq,
};

mod common;
use {anyhow::Result, ccthw_ash_allocator::FakeAllocator};

#[test]
pub fn test_allocate_with_mismatching_type_index_should_fail() -> Result<()> {
    common::setup_logger();

    let fake = into_shared(FakeAllocator::default());
    let mut allocator = MemoryTypePoolAllocator::new(0, fake);

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
