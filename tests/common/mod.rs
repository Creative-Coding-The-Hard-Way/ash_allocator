mod logging;
mod test_device;

pub use {self::logging::setup_logger, test_device::TestDevice};
use {anyhow::Result, ccthw_ash_instance::PhysicalDeviceFeatures};

/// Setup logging and create the Vulkan test device.
pub fn setup() -> Result<TestDevice> {
    setup_logger();
    TestDevice::new(PhysicalDeviceFeatures::default())
}
