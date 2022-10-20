use {
    anyhow::{Context, Result},
    ash::vk,
    ccthw_ash_instance::{
        LogicalDevice, PhysicalDevice, PhysicalDeviceFeatures, QueueFamilyInfo,
        VulkanHandle, VulkanInstance,
    },
    indoc::indoc,
};

/// The test device owns the Vulkan logical device and Vulkan instance for use
/// in integration tests. It's convenient to keep these values together because
/// they have similar lifetimes and are often used together.
#[derive(Debug)]
pub struct TestDevice {
    pub transfer_queue: vk::Queue,
    pub device: LogicalDevice,
    pub instance: VulkanInstance,
}

// Public API
// ----------

impl TestDevice {
    /// Create a new TestDevice which includes a Vulkan Instance and Logical
    /// Device.
    ///
    /// # Params
    ///
    /// * `features` - The physical device features required by the test.
    pub fn new(features: PhysicalDeviceFeatures) -> Result<Self> {
        let instance = unsafe {
            VulkanInstance::new(
                &[],
                &["VK_LAYER_KHRONOS_validation".to_owned()],
            )
            .context("Error creating the Vulkan Instance for the test device")?
        };
        let physical_device = Self::pick_physical_device(&instance, features)?;
        let transfer_queue_family_index =
            Self::pick_transfer_queue_family_index(&physical_device)?;

        let device = unsafe {
            let mut queue_family_info =
                QueueFamilyInfo::new(transfer_queue_family_index as u32);
            queue_family_info.add_queue_priority(1.0);

            LogicalDevice::new(
                &instance,
                physical_device,
                &[],
                &[queue_family_info],
            )
            .context("Error creating the logical device for this test")?
        };

        let transfer_queue = unsafe {
            device
                .raw()
                .get_device_queue(transfer_queue_family_index as u32, 0)
        };

        Ok(Self {
            transfer_queue,
            instance,
            device,
        })
    }
}

impl std::ops::Deref for TestDevice {
    type Target = ash::Device;

    fn deref(&self) -> &Self::Target {
        unsafe { self.device.raw() }
    }
}

impl Drop for TestDevice {
    fn drop(&mut self) {
        unsafe {
            self.device
                .raw()
                .device_wait_idle()
                .expect("Error while waiting for the device to idle!");
            self.device.destroy();
            self.instance.destroy();
        }
    }
}

impl std::fmt::Display for TestDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            indoc!(
                "
                TestDevice

                Transfer Queue {:?}

                {}

                {}"
            ),
            self.transfer_queue, self.device, self.instance,
        ))
    }
}

// Private API
// -----------

impl TestDevice {
    /// Pick a physical device which is suitable for this test.
    ///
    /// # Params
    ///
    /// * `instance` - the Vulkan instance used to acces devices.
    /// * `features` - the physical device features required by this
    ///   applicaiton.
    fn pick_physical_device(
        instance: &VulkanInstance,
        features: PhysicalDeviceFeatures,
    ) -> Result<PhysicalDevice> {
        let devices: Vec<PhysicalDevice> =
            PhysicalDevice::enumerate_supported_devices(instance, &features)?;

        let find_device_type =
            |device_type: vk::PhysicalDeviceType| -> Option<PhysicalDevice> {
                devices
                    .iter()
                    .find(|device| {
                        device.properties().properties().device_type
                            == device_type
                    })
                    .cloned()
            };

        if let Some(device) =
            find_device_type(vk::PhysicalDeviceType::DISCRETE_GPU)
        {
            return Ok(device);
        }

        if let Some(device) =
            find_device_type(vk::PhysicalDeviceType::INTEGRATED_GPU)
        {
            return Ok(device);
        }

        let device = devices
            .first()
            .cloned()
            .context("Unable to find a usable physical device!")?;
        Ok(device)
    }

    /// Pick a device queue family index which support memory transfer
    /// operations.
    ///
    /// # Params
    ///
    /// * `device` - the physical device to search for a transfer queue
    fn pick_transfer_queue_family_index(
        device: &PhysicalDevice,
    ) -> Result<usize> {
        device
            .queue_family_properties()
            .iter()
            .enumerate()
            .find(|(_index, props)| {
                props.queue_flags.contains(vk::QueueFlags::TRANSFER)
            })
            .map(|(index, _props)| index)
            .context("unable to find a suitable queue family")
    }
}
