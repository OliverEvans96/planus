use crate::errors::ErrorKind;

#[doc(hidden)]
#[derive(Copy, Clone, Debug)]
pub struct SliceWithStartOffset<'buf> {
    pub buffer: &'buf [u8],
    pub offset_from_start: usize,
}

impl<'buf> SliceWithStartOffset<'buf> {
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn as_slice(&self) -> &'buf [u8] {
        self.buffer
    }

    pub fn advance(&self, amount: usize) -> core::result::Result<Self, ErrorKind> {
        let buffer = self.buffer.get(amount..).ok_or(ErrorKind::InvalidOffset)?;
        Ok(Self {
            buffer,
            offset_from_start: self.offset_from_start + amount,
        })
    }

    pub fn advance_as_array<const N: usize>(
        &self,
        amount: usize,
    ) -> core::result::Result<ArrayWithStartOffset<'buf, N>, ErrorKind> {
        let buffer = self
            .buffer
            .get(amount..amount + N)
            .ok_or(ErrorKind::InvalidOffset)?;
        Ok(ArrayWithStartOffset {
            buffer: buffer.try_into().unwrap(),
            offset_from_start: self.offset_from_start + amount,
        })
    }

    /// # Safety
    /// TODO
    pub unsafe fn unchecked_advance_as_array<const N: usize>(
        &self,
        amount: usize,
    ) -> ArrayWithStartOffset<'buf, N> {
        let buffer = self.buffer.get_unchecked(amount..amount + N);
        ArrayWithStartOffset {
            buffer: buffer.try_into().unwrap(),
            offset_from_start: self.offset_from_start + amount,
        }
    }
}

#[doc(hidden)]
#[derive(Copy, Clone, Debug)]
pub struct ArrayWithStartOffset<'buf, const N: usize> {
    pub buffer: &'buf [u8; N],
    pub offset_from_start: usize,
}

impl<'buf, const N: usize> ArrayWithStartOffset<'buf, N> {
    pub fn as_array(&self) -> &'buf [u8; N] {
        self.buffer
    }

    pub fn advance_as_array<const K: usize>(
        &self,
        amount: usize,
    ) -> core::result::Result<ArrayWithStartOffset<'buf, K>, ErrorKind> {
        let buffer = self
            .buffer
            .get(amount..amount + K)
            .ok_or(ErrorKind::InvalidOffset)?;
        Ok(ArrayWithStartOffset {
            buffer: buffer.try_into().unwrap(),
            offset_from_start: self.offset_from_start + amount,
        })
    }
}
