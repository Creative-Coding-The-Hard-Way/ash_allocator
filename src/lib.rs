//! A general purpose Vulkan Memory allocator, written from scratch the hard
//! way.

mod allocation;
mod allocation_requirements;
mod device_memory;
mod error;
mod memory_allocator;
mod memory_properties;
mod pretty_wrappers;

pub use self::{
    allocation::Allocation,
    allocation_requirements::{
        AllocationRequirements, DedicatedResourceHandle,
    },
    error::AllocatorError,
    memory_allocator::{
        into_shared, ComposableAllocator, DeviceAllocator, FakeAllocator,
        MemoryAllocator, MemoryTypePoolAllocator, PageSuballocator,
        TraceAllocator,
    },
    memory_properties::MemoryProperties,
};
use self::{
    device_memory::DeviceMemory,
    pretty_wrappers::{PrettyBitflag, PrettySize},
};
