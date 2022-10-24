use {crate::pretty_wrappers::PrettyBitflag, ash::vk, thiserror::Error};

#[derive(Error, Debug)]
pub enum AllocatorError {
    #[error("No memory type for bits {0} and flags {1:#?}")]
    NoSupportedTypeForProperties(PrettyBitflag, vk::MemoryPropertyFlags),

    #[error(transparent)]
    RuntimeError(#[from] anyhow::Error),
}
