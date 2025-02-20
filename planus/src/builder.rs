use core::{marker::PhantomData, mem::MaybeUninit};

use crate::{backvec::BackVec, Offset, Primitive, WriteAsOffset};

#[derive(Debug)]
pub struct Builder {
    inner: BackVec,
    // This is a bit complicated. The buffer has support for guaranteeing a
    // specific write gets a specific alignment. It has many writes and thus
    // many promises, so how does keep track of this this across those promises, even
    // when writing from the back?
    //
    // The algorithm works by aggregating all of the promises into one big promise.
    // Specifically, we promise that the remaining part of the buffer will always
    // be of size `self.delayed_bytes + self.alignment() * K` where we are free to
    // choose K as we want.
    //
    // Initially we set `delayed_bytes` to 0 and `alignment` to 1, i.e. we have
    // only promised to write `0 + 1 * K` bytes, for any `K` we choose, which will
    // be true no matter how many bytes we write.
    //
    // Whenever we get a new request `(req_size, req_alignment)`, then that
    // `req_size` will be counted towards the previous promises, i.e. we need
    // to decrease `self.delayed_bytes()` by `req_bytes` and calculate the new value
    // of `req_size` modulo `alignment`. However we also need to fulfil this new
    // promise.
    //
    // To do this, we do two things. 1) We insert sufficient padding, before the
    // current request, to make sure that the current request is compatible with
    // the previous ones. 2) We set `alignment = alignment.max(req_alignment)`.
    //
    // One small wrinkle is that we do not store `alignment` directly for performance
    // reasons. Instead we store `alignment_mask = alignment - 1`, so we can do use
    // binary and (`&`) instead of modulo (`%`).
    delayed_bytes: usize,
    alignment_mask: usize,

    #[cfg(debug_assertions)]
    // Bytes missing to be written by a call to prepare_write
    missing_bytes: usize,
}

impl Default for Builder {
    fn default() -> Self {
        Self::with_capacity(0)
    }
}

impl Builder {
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: BackVec::with_capacity(capacity),

            delayed_bytes: 0,
            alignment_mask: 0,

            #[cfg(debug_assertions)]
            missing_bytes: 0,
        }
    }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.delayed_bytes = 0;
        self.alignment_mask = 0;
        #[cfg(debug_assertions)]
        {
            self.missing_bytes = 0;
        }
    }

    #[doc(hidden)]
    pub fn prepare_write(&mut self, size: usize, alignment_mask: usize) {
        debug_assert!((alignment_mask + 1) & alignment_mask == 0); // Check that the alignment is a power of two
        #[cfg(debug_assertions)]
        debug_assert_eq!(self.missing_bytes, 0);

        let delayed_bytes = self.delayed_bytes.wrapping_sub(size) & self.alignment_mask;
        let needed_padding = delayed_bytes & alignment_mask;
        self.delayed_bytes = delayed_bytes.wrapping_sub(needed_padding);
        self.alignment_mask |= alignment_mask;
        self.inner.reserve(size.wrapping_add(needed_padding));
        // TODO: investigate if it makes sense to use an extend_with_zeros_unchecked for performance, given
        // that we know we have enough space
        self.inner.extend_with_zeros(needed_padding);

        debug_assert_eq!(self.delayed_bytes & alignment_mask, 0);

        #[cfg(debug_assertions)]
        {
            self.missing_bytes = size;
        }
    }

    #[doc(hidden)]
    pub fn current_offset<T: ?Sized>(&self) -> Offset<T> {
        Offset {
            offset: self.inner.len() as u32,
            phantom: PhantomData,
        }
    }

    #[doc(hidden)]
    pub fn write(&mut self, buffer: &[u8]) {
        #[cfg(debug_assertions)]
        {
            self.missing_bytes = self.missing_bytes.checked_sub(buffer.len()).unwrap();
        }
        // TODO: investigate if it makes sense to use an extend_from_slice_unchecked for performance, given
        // that we know we have enough space
        self.inner.extend_from_slice(buffer);
    }

    #[doc(hidden)]
    pub unsafe fn write_with(
        &mut self,
        size: usize,
        alignment_mask: usize,
        f: impl FnOnce(u32, &mut [MaybeUninit<u8>]),
    ) {
        self.prepare_write(size, alignment_mask);
        let offset = (self.inner.len() + size) as u32;
        self.inner.extend_write(size, |bytes| f(offset, bytes));
        #[cfg(debug_assertions)]
        {
            self.missing_bytes = self.missing_bytes.checked_sub(size).unwrap();
        }
    }

    #[doc(hidden)]
    pub fn get_buffer_position_and_prepare_write(
        &mut self,
        vtable_size: usize,
        object_size: usize,
        object_alignment_mask: usize,
    ) -> usize {
        debug_assert!((object_alignment_mask + 1) & object_alignment_mask == 0); // Check that the alignment is a power of two

        const VTABLE_ALIGNMENT: usize = 2;
        const VTABLE_ALIGNMENT_MASK: usize = VTABLE_ALIGNMENT - 1;
        self.prepare_write(vtable_size + 4, VTABLE_ALIGNMENT_MASK);

        let delayed_bytes = self.delayed_bytes.wrapping_sub(object_size) & self.alignment_mask;
        let needed_padding = delayed_bytes & object_alignment_mask;

        self.inner.len() + vtable_size + 4 + needed_padding + object_size + 4
    }

    pub fn finish<T>(
        &mut self,
        root: impl WriteAsOffset<T>,
        file_identifier: Option<[u8; 4]>,
    ) -> &[u8] {
        let root = root.prepare(self);

        if let Some(file_identifier) = file_identifier {
            // TODO: how does alignment interact with file identifiers? Is the alignment with out without the header?
            self.prepare_write(
                8,
                <Offset<T> as Primitive>::ALIGNMENT_MASK.max(self.alignment_mask),
            );
            self.write(&(4 + self.inner.len() as u32 - root.offset).to_le_bytes());
            self.write(&file_identifier);
        } else {
            self.prepare_write(
                4,
                <Offset<T> as Primitive>::ALIGNMENT_MASK.max(self.alignment_mask),
            );
            self.write(&(4 + self.inner.len() as u32 - root.offset).to_le_bytes());
        }
        debug_assert_eq!(self.delayed_bytes, 0);
        self.inner.as_slice()
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;
    use rand::{thread_rng, Rng};

    use super::*;

    #[test]
    fn test_buffer_random() {
        let mut slice = [0; 128];
        let mut rng = thread_rng();
        let mut back_offsets: Vec<(usize, usize, usize)> = Vec::new();

        for _ in 0..50 {
            let mut builder = Builder::new();
            back_offsets.clear();

            for byte in 1..50 {
                let size: usize = rng.gen::<usize>() % slice.len();
                let slice = &mut slice[..size];
                for p in &mut *slice {
                    *p = byte;
                }
                let alignment: usize = 1 << (rng.gen::<u32>() % 5);
                let alignment_mask = alignment - 1;
                builder.prepare_write(size, alignment_mask);
                let len_before = builder.inner.len();
                builder.write(slice);
                assert!(builder.inner.len() < len_before + slice.len() + alignment);
                back_offsets.push((builder.inner.len(), size, alignment));
            }
            let random_padding: usize = rng.gen::<usize>() % slice.len();
            let slice = &mut slice[..random_padding];
            for p in &mut *slice {
                *p = rng.gen();
            }
            builder.prepare_write(random_padding, 1);
            builder.write(slice);
            let buffer = builder.finish(builder.current_offset::<()>(), None);

            for (i, (back_offset, size, alignment)) in back_offsets.iter().enumerate() {
                let byte = (i + 1) as u8;
                let offset = buffer.len() - back_offset;
                assert_eq!(offset % alignment, 0);
                assert!(buffer[offset..offset + size].iter().all(|&b| b == byte));
            }
        }
    }

    #[test]
    fn test_buffer_align() {
        let mut builder = Builder::new();
        builder.prepare_write(3, 0);
        builder.write(b"MNO");
        assert_eq!(builder.delayed_bytes, 0);
        builder.prepare_write(4, 1);
        builder.write(b"IJKL");
        assert_eq!(builder.delayed_bytes, 0);
        builder.prepare_write(8, 3);
        builder.write(b"ABCDEFGH");
        assert_eq!(builder.delayed_bytes, 0);
        builder.prepare_write(7, 0);
        builder.write(b"0123456");
        assert_eq!(
            builder.finish(builder.current_offset::<()>(), None),
            b"\x05\x00\x00\x00\x000123456ABCDEFGHIJKLMNO"
        );

        builder.clear();
        builder.prepare_write(4, 3);
        builder.write(b"IJKL");
        assert_eq!(builder.delayed_bytes, 0);
        builder.prepare_write(1, 0);
        builder.write(b"X");
        assert_eq!(builder.delayed_bytes, 3);
        builder.prepare_write(1, 0);
        builder.write(b"Y");
        assert_eq!(builder.delayed_bytes, 2);
        builder.prepare_write(8, 7);
        builder.write(b"ABCDEFGH");
        assert_eq!(builder.delayed_bytes, 0);
        assert_eq!(
            builder.finish(builder.current_offset::<()>(), None),
            b"\x08\x00\x00\x00\x00\x00\x00\x00ABCDEFGH\x00\x00YXIJKL"
        );
    }
}
