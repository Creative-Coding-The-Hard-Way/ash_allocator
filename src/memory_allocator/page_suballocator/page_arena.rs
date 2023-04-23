//! # Overview
//!
//! It is useful to manage memory in sets of evenly-sized pages. This module
//! defines types for manipulating a collection of contiguous pages.
//!
//! ## Terms
//!
//! * Page: A representation of a single unit of memory with a fixed size.
//! * Arena: A collection of contiguous pages.
//! * Chunk: A contiguous subset of pages which can be allocated from the arena.

/// A representation of a single unit of memory with a fixed size.
/// Pages can either be free or allocated. Pages are allocated in contiguous
/// chunks and they each keep track of where their current chunk begins.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Page {
    Free,
    Allocated {
        /// The index of the first Allocated page in the chunk containing this
        /// allocation.
        first_in_chunk: usize,
    },
}

/// A contiguous collection of Pages which can be used to allocate and free
/// chunks.
pub struct PageArena {
    pages: Vec<Page>,
    allocation_count: usize,
}

impl PageArena {
    /// Create a new arena with a fixed number of pages that are all the same
    /// size.
    ///
    /// # Params
    ///
    /// * page_count - the number of pages to manage
    pub fn new(page_count: usize) -> Self {
        Self {
            pages: vec![Page::Free; page_count],
            allocation_count: 0,
        }
    }

    /// Returns true when there are no allocated chunks.
    pub fn is_empty(&self) -> bool {
        self.allocation_count == 0
    }

    /// Allocate a chunk of contiguous pages.
    ///
    /// # Params
    ///
    /// * page_count - the number of contiguous pages to allocate.
    ///
    /// # Returns
    ///
    /// * Some(index) - the index of the first page in the allocated chunk.
    /// * None - when the chunk could not be allocated
    pub fn allocate_chunk(&mut self, page_count: usize) -> Option<usize> {
        let first_in_chunk = self.find_first_free_chunk(page_count)?;

        debug_assert!(first_in_chunk + page_count <= self.pages.len());
        for page in self.pages.iter_mut().skip(first_in_chunk).take(page_count)
        {
            debug_assert!(
                *page == Page::Free,
                "Unexpected value in chunk when setting new value!"
            );
            *page = Page::Allocated { first_in_chunk };
        }

        self.allocation_count += 1;

        Some(first_in_chunk)
    }

    /// Free a chunk of contiguous pages.
    ///
    /// # Params
    ///
    /// * index - the index of a page within the chunk to free. This doesn't
    ///   need to be the start of the page, it just needs to be somewhere in the
    ///   chunk.
    pub fn free_chunk(&mut self, index: usize) {
        debug_assert!(self.pages[index] != Page::Free);
        let first_in_chunk = {
            match self.pages[index] {
                Page::Free => {
                    return;
                }
                Page::Allocated { first_in_chunk } => first_in_chunk,
            }
        };
        for page in self
            .pages
            .iter_mut()
            .skip(first_in_chunk)
            .take_while(|p| **p == Page::Allocated { first_in_chunk })
        {
            *page = Page::Free;
        }
        self.allocation_count -= 1;
    }

    /// Find the index of the first contiguous free chunk that is large enough
    /// to fit the requested size.
    ///
    /// # Params
    ///
    /// * page_count: The number of contiguous free pages being requested.
    ///
    /// # Returns
    ///
    /// * Some(index): The index of the first free page which has at least
    ///   page_count free pages after it.
    /// * None: When there isn't enough space.
    fn find_first_free_chunk(&self, page_count: usize) -> Option<usize> {
        let mut in_region = false;
        let mut start: usize = 0;
        for (index, &value) in self.pages.iter().enumerate() {
            if value == Page::Free {
                if !in_region {
                    start = index;
                    in_region = true;
                }
                if in_region && (index - start) == (page_count - 1) {
                    return Some(start);
                }
            } else if in_region {
                in_region = false;
                start = 0;
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    use {super::*, pretty_assertions::assert_eq};

    fn page_from_str(page: &str) -> Page {
        if page == "f" {
            return Page::Free;
        }
        let first_in_chunk = str::parse(page).unwrap();
        Page::Allocated { first_in_chunk }
    }

    fn pages_from_str(pages: &str) -> Vec<Page> {
        pages.split('|').map(page_from_str).collect::<Vec<Page>>()
    }

    fn page_to_str(page: &Page) -> String {
        match *page {
            Page::Free => "f".into(),
            Page::Allocated { first_in_chunk } => format!("{first_in_chunk}"),
        }
    }

    fn pages_to_str(pages: &[Page]) -> String {
        pages.iter().map(page_to_str).collect::<String>()
    }

    fn arena_with_pages(pages: &str, allocation_count: usize) -> PageArena {
        PageArena {
            pages: pages_from_str(pages),
            allocation_count,
        }
    }

    #[test]
    fn test_page_arena_constructor() {
        let arena = PageArena::new(5);
        assert_eq!(pages_to_str(&arena.pages), "fffff");
    }

    #[test]
    fn test_find_first_free_chunk() {
        let arena = PageArena::new(5);
        assert_eq!(arena.find_first_free_chunk(1), Some(0));
        assert_eq!(arena.find_first_free_chunk(5), Some(0));
        assert_eq!(arena.find_first_free_chunk(6), None);

        let arena = arena_with_pages("f|1|1|f|f|f|6|6|6|6|f|f", 2);
        assert_eq!(arena.find_first_free_chunk(1), Some(0));
        assert_eq!(arena.find_first_free_chunk(2), Some(3));
        assert_eq!(arena.find_first_free_chunk(3), Some(3));
        assert_eq!(arena.find_first_free_chunk(4), None);
    }

    #[test]
    fn test_page_arena_allocation() {
        let mut arena = PageArena::new(10);
        assert_eq!(arena.allocate_chunk(5), Some(0));
        assert_eq!(pages_to_str(&arena.pages), "00000fffff");
        assert_eq!(arena.allocation_count, 1);

        assert_eq!(arena.allocate_chunk(2), Some(5));
        assert_eq!(pages_to_str(&arena.pages), "0000055fff");

        assert_eq!(arena.allocate_chunk(3), Some(7));
        assert_eq!(pages_to_str(&arena.pages), "0000055777");

        assert_eq!(arena.allocate_chunk(1), None);
        assert_eq!(pages_to_str(&arena.pages), "0000055777");
    }

    #[test]
    fn test_page_arena_free() {
        let mut arena = arena_with_pages("f|f|2|2|2|2", 1);
        arena.free_chunk(4);
        assert_eq!(pages_to_str(&arena.pages), "ffffff");
    }

    #[test]
    fn test_page_arena_allocate_and_free() {
        let mut arena = PageArena::new(10);
        assert_eq!(arena.allocate_chunk(5), Some(0));
        assert_eq!(arena.allocate_chunk(2), Some(5));
        assert_eq!(arena.allocate_chunk(3), Some(7));
        assert_eq!(pages_to_str(&arena.pages), "0000055777");

        arena.free_chunk(3); // somewhere in that first chunk
        assert_eq!(pages_to_str(&arena.pages), "fffff55777");

        arena.free_chunk(7); // right at the beginning of the chunk
        assert_eq!(pages_to_str(&arena.pages), "fffff55fff");

        arena.free_chunk(6); // at the very end of the chunk
        assert_eq!(pages_to_str(&arena.pages), "ffffffffff");
        assert!(arena.is_empty());
    }

    #[test]
    fn test_smoke_test() {
        let mut chunks = vec![];
        let mut arena = PageArena::new(1000);

        let count = 10_000;
        for _ in 0..count {
            if let Some(index) = arena.allocate_chunk(5) {
                chunks.push(index);
            }
        }

        for index in chunks.drain(0..) {
            arena.free_chunk(index);
        }

        assert!(arena.is_empty());
    }
}
