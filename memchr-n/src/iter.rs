///! Copied from memchr

/// A trait for adding some helper routines to pointers.
pub(crate) trait Pointer {
    /// Returns the distance, in units of `T`, between `self` and `origin`.
    ///
    /// # Safety
    ///
    /// Same as `ptr::offset_from` in addition to `self >= origin`.
    unsafe fn distance(self, origin: Self) -> usize;

    /// Casts this pointer to `usize`.
    ///
    /// Callers should not convert the `usize` back to a pointer if at all
    /// possible. (And if you believe it's necessary, open an issue to discuss
    /// why. Otherwise, it has the potential to violate pointer provenance.)
    /// The purpose of this function is just to be able to do arithmetic, i.e.,
    /// computing offsets or alignments.
    fn as_usize(self) -> usize;
}

impl<T> Pointer for *const T {
    unsafe fn distance(self, origin: *const T) -> usize {
        // TODO: Replace with `ptr::sub_ptr` once stabilized.
        usize::try_from(self.offset_from(origin)).unwrap_unchecked()
    }

    fn as_usize(self) -> usize {
        self as usize
    }
}

impl<T> Pointer for *mut T {
    unsafe fn distance(self, origin: *mut T) -> usize {
        (self as *const T).distance(origin as *const T)
    }

    fn as_usize(self) -> usize {
        (self as *const T).as_usize()
    }
}

/// A trait for adding some helper routines to raw bytes.
#[cfg(test)]
pub(crate) trait Byte {
    /// Converts this byte to a `char` if it's ASCII. Otherwise panics.
    fn to_char(self) -> char;
}

#[cfg(test)]
impl Byte for u8 {
    fn to_char(self) -> char {
        assert!(self.is_ascii());
        char::from(self)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Iter<'h> {
    /// The original starting point into the haystack. We use this to convert
    /// pointers to offsets.
    original_start: *const u8,
    /// The current starting point into the haystack. That is, where the next
    /// search will begin.
    start: *const u8,
    /// The current ending point into the haystack. That is, where the next
    /// reverse search will begin.
    end: *const u8,
    /// A marker for tracking the lifetime of the start/cur_start/cur_end
    /// pointers above, which all point into the haystack.
    haystack: core::marker::PhantomData<&'h [u8]>,
}

// SAFETY: Iter contains no shared references to anything that performs any
// interior mutations. Also, the lifetime guarantees that Iter will not outlive
// the haystack.
unsafe impl<'h> Send for Iter<'h> {}

// SAFETY: Iter perform no interior mutations, therefore no explicit
// synchronization is necessary. Also, the lifetime guarantees that Iter will
// not outlive the haystack.
unsafe impl<'h> Sync for Iter<'h> {}

impl<'h> Iter<'h> {
    /// Create a new generic memchr iterator.
    #[inline(always)]
    pub(crate) fn new(haystack: &'h [u8]) -> Iter<'h> {
        Iter {
            original_start: haystack.as_ptr(),
            start: haystack.as_ptr(),
            end: haystack.as_ptr().wrapping_add(haystack.len()),
            haystack: core::marker::PhantomData,
        }
    }

    /// Returns the next occurrence in the forward direction.
    ///
    /// # Safety
    ///
    /// Callers must ensure that if a pointer is returned from the closure
    /// provided, then it must be greater than or equal to the start pointer
    /// and less than the end pointer.
    #[inline(always)]
    pub(crate) unsafe fn next(
        &mut self,
        mut find_raw: impl FnMut(*const u8, *const u8) -> Option<*const u8>,
    ) -> Option<usize> {
        // SAFETY: Pointers are derived directly from the same &[u8] haystack.
        // We only ever modify start/end corresponding to a matching offset
        // found between start and end. Thus all changes to start/end maintain
        // our safety requirements.
        //
        // The only other assumption we rely on is that the pointer returned
        // by `find_raw` satisfies `self.start <= found < self.end`, and that
        // safety contract is forwarded to the caller.
        let found = find_raw(self.start, self.end)?;
        let result = found.distance(self.original_start);
        self.start = found.add(1);
        Some(result)
    }

    /// Returns the number of remaining elements in this iterator.
    #[inline(always)]
    pub(crate) fn count(self, mut count_raw: impl FnMut(*const u8, *const u8) -> usize) -> usize {
        // SAFETY: Pointers are derived directly from the same &[u8] haystack.
        // We only ever modify start/end corresponding to a matching offset
        // found between start and end. Thus all changes to start/end maintain
        // our safety requirements.
        count_raw(self.start, self.end)
    }
}
