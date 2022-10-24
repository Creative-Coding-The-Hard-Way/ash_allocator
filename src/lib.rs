//! A general purpose Vulkan Memory allocator, written from scratch the hard
//! way.

mod allocation_requirements;
mod error;
mod memory_allocator;
mod memory_properties;
mod pretty_wrappers;

use self::pretty_wrappers::{PrettyBitflag, PrettySize};
pub use self::{
    allocation_requirements::{
        AllocationRequirements, DedicatedResourceHandle,
    },
    error::AllocatorError,
    memory_properties::MemoryProperties,
};
