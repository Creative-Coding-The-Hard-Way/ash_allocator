use {
    crate::{
        pretty_wrappers::PrettySize, Allocation, AllocationRequirements,
        AllocatorError, ComposableAllocator, MemoryProperties,
    },
    ash::vk,
    indoc::indoc,
    std::collections::HashMap,
};

#[derive(Default)]
struct Metrics {
    total_allocations: u32,
    leaked_allocations: u32,
    max_size: u64,
    min_size: u64,
    avg_size: u64,
}

impl Metrics {
    fn record_allocation(&mut self, size: u64) {
        self.total_allocations += 1;
        self.leaked_allocations += 1;
        self.max_size = self.max_size.max(size);
        self.min_size = self.min_size.min(size);

        let n = self.total_allocations as u64;
        self.avg_size = (size / n) + ((n - 1) / n) * self.avg_size;
    }

    fn record_free(&mut self) {
        self.leaked_allocations -= 1;
    }
}

/// An allocator decorator which tracks metrics and generates a report for
/// all allocations made to the wrapped allocator.
pub struct TraceAllocator<T: ComposableAllocator> {
    wrapped_allocator: T,
    name: String,
    total: Metrics,
    per_type: HashMap<usize, Metrics>,
    properties: MemoryProperties,
}

impl<T: ComposableAllocator> TraceAllocator<T> {
    pub fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        wrapped_allocator: T,
        name: impl Into<String>,
    ) -> Self {
        let properties = MemoryProperties::new(instance, physical_device);
        Self {
            wrapped_allocator,
            name: name.into(),
            total: Metrics::default(),
            per_type: HashMap::new(),
            properties,
        }
    }
}

impl<T: ComposableAllocator> Drop for TraceAllocator<T> {
    fn drop(&mut self) {
        let mut report = format!(
            indoc!(
                "
                # {} Allocation Trace

                ## Total Allocations

                total allocations: {}
                leaked allocations: {}
                min_size: {}
                max_size: {}
                avg_size: {}

                ## Allocations Per Memory Type

                "
            ),
            self.name,
            self.total.total_allocations,
            self.total.leaked_allocations,
            PrettySize(self.total.min_size),
            PrettySize(self.total.max_size),
            PrettySize(self.total.avg_size),
        );

        for (memory_type_index, metrics) in self.per_type.iter() {
            report.push_str(&format!(
                indoc!(
                    "
                    ### Memory Type {}
                    Properties: {:#?}

                    total allocations: {}
                    leaked allocations: {}
                    min_size: {}
                    max_size: {}
                    avg_size: {}

                    "
                ),
                memory_type_index,
                self.properties.types()[*memory_type_index].property_flags,
                metrics.total_allocations,
                metrics.leaked_allocations,
                PrettySize(self.total.min_size),
                PrettySize(self.total.max_size),
                PrettySize(self.total.avg_size),
            ));
        }

        log::debug!("{}", report);
    }
}

impl<T: ComposableAllocator> ComposableAllocator for TraceAllocator<T> {
    unsafe fn allocate(
        &mut self,
        allocation_requirements: AllocationRequirements,
    ) -> Result<Allocation, AllocatorError> {
        self.total
            .record_allocation(allocation_requirements.size_in_bytes);
        self.per_type
            .entry(allocation_requirements.memory_type_index)
            .or_default()
            .record_allocation(allocation_requirements.size_in_bytes);
        self.wrapped_allocator.allocate(allocation_requirements)
    }

    unsafe fn free(&mut self, allocation: Allocation) {
        self.total.record_free();
        self.per_type
            .entry(allocation.memory_type_index())
            .or_default()
            .record_free();
        self.wrapped_allocator.free(allocation)
    }
}
