// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::BlockIo;
use core::fmt::{self, Debug, Display, Formatter};
use core::ops::Range;
use gpt_disk_types::{BlockSize, Lba};

/// Error type used by [`MutSliceBlockIo`].
///
/// If the `std` feature is enabled, this type implements the [`Error`]
/// trait.
///
/// [`Error`]: std::error::Error
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum SliceBlockIoError {
    /// Numeric overflow occurred.
    Overflow,

    /// Attempted to write a read-only byte slice.
    ReadOnly,

    /// A read or write is out of bounds.
    OutOfBounds {
        /// Start LBA.
        start_lba: Lba,

        /// Length in bytes.
        length_in_bytes: usize,
    },
}

impl Default for SliceBlockIoError {
    fn default() -> Self {
        SliceBlockIoError::Overflow
    }
}

impl Display for SliceBlockIoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Overflow => f.write_str("numeric overflow occurred"),
            Self::ReadOnly => {
                f.write_str("attempted to write to a read-only byte slice")
            }
            Self::OutOfBounds {
                start_lba,
                length_in_bytes,
            } => {
                write!(
                    f,
                    "out of bounds: start_lba={start_lba}, length_in_bytes={length_in_bytes}"
                )
            }
        }
    }
}

fn buffer_byte_range_opt(
    block_size: BlockSize,
    start_lba: Lba,
    buf: &[u8],
) -> Option<Range<usize>> {
    let start_lba = usize::try_from(start_lba).ok()?;
    let start_byte = start_lba.checked_mul(block_size.to_usize()?)?;
    let end_byte = start_byte.checked_add(buf.len())?;
    Some(start_byte..end_byte)
}

fn buffer_byte_range(
    block_size: BlockSize,
    start_lba: Lba,
    buf: &[u8],
) -> Result<Range<usize>, SliceBlockIoError> {
    buffer_byte_range_opt(block_size, start_lba, buf)
        .ok_or(SliceBlockIoError::Overflow)
}

/// Wrapper type that implements the [`BlockIo`] trait for immutable byte
/// slices.
#[allow(clippy::module_name_repetitions)]
pub struct SliceBlockIo<'a> {
    data: &'a [u8],
    block_size: BlockSize,
}

impl<'a> SliceBlockIo<'a> {
    /// Create a new `SliceBlockIo`.
    #[must_use]
    pub fn new(data: &'a [u8], block_size: BlockSize) -> Self {
        Self { data, block_size }
    }
}

impl<'a> BlockIo for SliceBlockIo<'a> {
    type Error = SliceBlockIoError;

    fn block_size(&self) -> BlockSize {
        self.block_size
    }

    fn num_blocks(&mut self) -> Result<u64, Self::Error> {
        let data_len = u64::try_from(self.data.len())
            .map_err(|_| SliceBlockIoError::Overflow)?;

        Ok(data_len / self.block_size().to_u64())
    }

    fn read_blocks(
        &mut self,
        start_lba: Lba,
        dst: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.assert_valid_buffer(dst);

        let src = self
            .data
            .get(buffer_byte_range(self.block_size(), start_lba, dst)?)
            .ok_or(Self::Error::OutOfBounds {
                start_lba,
                length_in_bytes: dst.len(),
            })?;
        dst.copy_from_slice(src);
        Ok(())
    }

    fn write_blocks(
        &mut self,
        _start_lba: Lba,
        _src: &[u8],
    ) -> Result<(), Self::Error> {
        Err(Self::Error::ReadOnly)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// Wrapper type that implements the [`BlockIo`] trait for mutable byte
/// slices.
#[allow(clippy::module_name_repetitions)]
pub struct MutSliceBlockIo<'a> {
    data: &'a mut [u8],
    block_size: BlockSize,
}

impl<'a> MutSliceBlockIo<'a> {
    /// Create a new `MutSliceBlockIo`.
    pub fn new(data: &'a mut [u8], block_size: BlockSize) -> Self {
        Self { data, block_size }
    }

    fn buffer_byte_range_opt(
        &self,
        start_lba: Lba,
        buf: &[u8],
    ) -> Option<Range<usize>> {
        let start_lba = usize::try_from(start_lba).ok()?;
        let start_byte =
            start_lba.checked_mul(self.block_size().to_usize()?)?;
        let end_byte = start_byte.checked_add(buf.len())?;
        Some(start_byte..end_byte)
    }

    fn buffer_byte_range(
        &self,
        start_lba: Lba,
        buf: &[u8],
    ) -> Result<Range<usize>, SliceBlockIoError> {
        self.buffer_byte_range_opt(start_lba, buf)
            .ok_or(SliceBlockIoError::Overflow)
    }
}

impl<'a> BlockIo for MutSliceBlockIo<'a> {
    type Error = SliceBlockIoError;

    fn block_size(&self) -> BlockSize {
        self.block_size
    }

    fn num_blocks(&mut self) -> Result<u64, Self::Error> {
        let data_len = u64::try_from(self.data.len())
            .map_err(|_| SliceBlockIoError::Overflow)?;

        Ok(data_len / self.block_size().to_u64())
    }

    fn read_blocks(
        &mut self,
        start_lba: Lba,
        dst: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.assert_valid_buffer(dst);

        let src = self
            .data
            .get(self.buffer_byte_range(start_lba, dst)?)
            .ok_or(Self::Error::OutOfBounds {
                start_lba,
                length_in_bytes: dst.len(),
            })?;
        dst.copy_from_slice(src);
        Ok(())
    }

    fn write_blocks(
        &mut self,
        start_lba: Lba,
        src: &[u8],
    ) -> Result<(), Self::Error> {
        self.assert_valid_buffer(src);

        let dst = self
            .data
            .get_mut(self.buffer_byte_range(start_lba, src)?)
            .ok_or(Self::Error::OutOfBounds {
                start_lba,
                length_in_bytes: src.len(),
            })?;
        dst.copy_from_slice(src);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
