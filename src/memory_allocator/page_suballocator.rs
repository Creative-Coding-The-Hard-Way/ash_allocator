//! An allocator which allocates chunks from an existing allocation.

use crate::Allocation;

struct PageSuballocator {
    allocation: Allocation,
    page_size_in_bytes: u64,
}

/// Flip all of the bits in a given region.
fn set_region(bitmap: &mut [bool], value: bool, start: usize, size: usize) {
    assert!(start + size <= bitmap.len());
    for bit in bitmap.iter_mut().skip(start).take(size) {
        *bit = value;
    }
}

/// Find the index of the first contiguous region of 0s in the bitmap which is
/// at least as big as size.
///
/// # Params
///
/// * bitmap: The bitmap to search.
/// * size: The size of the region to search for.
///
/// # Returns
///
/// * Some(index): The index of the first contiguous region of at least size 0s.
/// * None: When no region with the requested size could be found.
fn linear_probe(bitmap: Vec<bool>, size: usize) -> Option<usize> {
    let mut in_region = false;
    let mut start: usize = 0;
    for (index, &value) in bitmap.iter().enumerate() {
        if !value {
            if !in_region {
                start = index;
                in_region = true;
            }

            if in_region && (index - start) == (size - 1) {
                return Some(start);
            }
        } else if in_region {
            in_region = false;
            start = 0;
        }
    }
    None
}

#[cfg(test)]
mod test {
    use super::{linear_probe, set_region};

    #[test]
    fn test_linear_probe() {
        assert_eq!(linear_probe(vec![false, false, false, false], 2), Some(0));
        assert_eq!(linear_probe(vec![true, false, false, false], 2), Some(1));
        assert_eq!(linear_probe(vec![true, true, false, false], 2), Some(2));
        assert_eq!(linear_probe(vec![true, true, true, false], 2), None);

        assert_eq!(linear_probe(vec![true, false, false, true], 2), Some(1));

        assert_eq!(
            linear_probe(
                vec![true, true, false, false, true, false, false, false, true],
                3
            ),
            Some(5)
        );
        assert_eq!(
            linear_probe(
                vec![true, true, true, false, true, false, false, false, true],
                1
            ),
            Some(3)
        );
        assert_eq!(
            linear_probe(
                vec![true, true, true, true, true, false, false, false, true],
                1,
            ),
            Some(5)
        );
    }

    #[test]
    fn test_set_region() {
        let mut bitmap = vec![false, false, false, false, false];

        set_region(&mut bitmap, true, 2, 2);
        assert_eq!(bitmap, vec![false, false, true, true, false]);

        set_region(&mut bitmap, true, 4, 1);
        assert_eq!(bitmap, vec![false, false, true, true, true]);

        set_region(&mut bitmap, false, 4, 0);
        assert_eq!(bitmap, vec![false, false, true, true, true]);

        set_region(&mut bitmap, false, 2, 1);
        assert_eq!(bitmap, vec![false, false, false, true, true]);
    }

    #[test]
    #[should_panic]
    fn test_set_region_panics_when_starts_outside_range() {
        let mut bitmap = vec![false, false, false, false, false];
        set_region(&mut bitmap, true, 8, 1);
    }

    #[test]
    #[should_panic]
    fn test_set_region_panics_when_ends_outside_range() {
        let mut bitmap = vec![false, false, false, false, false];
        set_region(&mut bitmap, true, 2, 19);
    }
}
