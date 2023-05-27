//! Tests for the pool allocator.

use {
    anyhow::Result,
    ash::vk,
    ccthw_ash_allocator::{
        into_shared, AllocationRequirements, ComposableAllocator,
        FakeAllocator, MemoryProperties, PoolAllocator,
    },
};

mod common;

#[test]
fn test_allocate_and_free() -> Result<()> {
    common::setup_logger();

    let fake_allocator = into_shared(FakeAllocator::default());
    let memory_properties = unsafe {
        // Safe because the fake_allocater will never actually attempt to
        // allocate real memory.
        MemoryProperties::from_raw(
            &[
                vk::MemoryType {
                    property_flags: vk::MemoryPropertyFlags::empty(),
                    heap_index: 0,
                },
                vk::MemoryType {
                    property_flags: vk::MemoryPropertyFlags::empty(),
                    heap_index: 0,
                },
            ],
            &[vk::MemoryHeap {
                size: 128_000,
                flags: vk::MemoryHeapFlags::empty(),
            }],
        )
    };
    let mut allocator =
        PoolAllocator::new(memory_properties, 64, 1, fake_allocator.clone());

    let a1 = unsafe {
        allocator.allocate(AllocationRequirements {
            memory_type_index: 0,
            alignment: 1,
            size_in_bytes: 32,
            ..AllocationRequirements::default()
        })?
    };
    let a2 = unsafe {
        allocator.allocate(AllocationRequirements {
            memory_type_index: 0,
            alignment: 1,
            size_in_bytes: 32,
            ..AllocationRequirements::default()
        })?
    };

    assert_eq!(a1.size_in_bytes(), 32);
    assert_eq!(a2.size_in_bytes(), 32);
    assert_eq!(fake_allocator.lock().unwrap().active_allocations, 1);

    let a3 = unsafe {
        allocator.allocate(AllocationRequirements {
            memory_type_index: 1,
            alignment: 1,
            size_in_bytes: 32,
            ..AllocationRequirements::default()
        })?
    };

    assert_eq!(a3.size_in_bytes(), 32);
    assert_eq!(fake_allocator.lock().unwrap().active_allocations, 2);

    unsafe {
        allocator.free(a1);
        allocator.free(a2);
        allocator.free(a3);
    }

    assert_eq!(fake_allocator.lock().unwrap().active_allocations, 0);

    Ok(())
}

#[test]
fn test_allocation_should_fail_when_too_big() {
    common::setup_logger();

    let fake_allocator = into_shared(FakeAllocator::default());
    let memory_properties = unsafe {
        // Safe because the fake_allocater will never actually attempt to
        // allocate real memory.
        MemoryProperties::from_raw(
            &[vk::MemoryType {
                property_flags: vk::MemoryPropertyFlags::empty(),
                heap_index: 0,
            }],
            &[vk::MemoryHeap {
                size: 1,
                flags: vk::MemoryHeapFlags::empty(),
            }],
        )
    };
    let chunk_size = 64;
    let mut allocator =
        PoolAllocator::new(memory_properties, chunk_size, 1, fake_allocator);

    unsafe {
        // Attempt to allocate a piece of memory that's as large as one of the
        // pool's entire chunks.
        let result = allocator.allocate(AllocationRequirements {
            memory_type_index: 0,
            size_in_bytes: chunk_size,
            alignment: 1,
            ..AllocationRequirements::default()
        });
        assert!(result.is_err());
    }

    unsafe {
        // Attempt to allocate a piece of memory that's bigger than one of the
        // pool's entire chunks.
        let result = allocator.allocate(AllocationRequirements {
            memory_type_index: 0,
            size_in_bytes: chunk_size * 2,
            alignment: 1,
            ..AllocationRequirements::default()
        });
        assert!(result.is_err());
    }
}

#[test]
#[should_panic]
fn test_allocation_should_fail_when_using_an_invalid_memory_type_index() {
    common::setup_logger();

    let fake_allocator = into_shared(FakeAllocator::default());
    let memory_properties = unsafe {
        // Safe because the fake_allocater will never actually attempt to
        // allocate real memory.
        MemoryProperties::from_raw(
            &[vk::MemoryType {
                property_flags: vk::MemoryPropertyFlags::empty(),
                heap_index: 0,
            }],
            &[vk::MemoryHeap {
                size: 1,
                flags: vk::MemoryHeapFlags::empty(),
            }],
        )
    };
    let mut allocator =
        PoolAllocator::new(memory_properties, 64, 1, fake_allocator);

    unsafe {
        let _result = allocator.allocate(AllocationRequirements {
            memory_type_index: 1,
            size_in_bytes: 20,
            alignment: 1,
            ..AllocationRequirements::default()
        });
    }
}
