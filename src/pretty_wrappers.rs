/// A pretty-printer wrapper for how big something is in bytes.
///
/// The wrapper automatically rounds to the nearest macro unit (kilobytes,
/// megabytes, gigabytes, etc..) so that large-sizes are easier to
/// reason about at a glance.
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct PrettySize(pub u64);

/// Used when pretty-printing units.
const UNIT_NAMES: [&str; 5] = ["b", "kb", "mb", "gb", "pb"];

impl std::fmt::Debug for PrettySize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            let size = self.0 as f32;
            let unit_pow = size.log(1024.0).clamp(0.0, 4.0).floor();
            let unit_size_in_bytes = 1024.0_f32.powf(unit_pow);
            let size_in_units = size / unit_size_in_bytes;
            let unit_name = UNIT_NAMES[unit_pow as usize];

            f.write_fmt(format_args!("{} {}", size_in_units, unit_name))
        } else {
            f.write_fmt(format_args!("{}", self.0))
        }
    }
}

impl std::fmt::Display for PrettySize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:#?}", self))
    }
}

/// A pretty-printer wrapper for a bitflag. All it does is make the binary
/// representation of the value get printed rather than the base-10 version.
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct PrettyBitflag(pub u32);

impl std::fmt::Debug for PrettyBitflag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:b}", self.0))
    }
}

impl std::fmt::Display for PrettyBitflag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:#?}", self))
    }
}
