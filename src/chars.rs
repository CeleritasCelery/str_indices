//! Index by chars.

use crate::byte_chunk::{ByteChunk, Chunk};

/// Counts the chars in a string slice.
///
/// Runs in O(N) time.
pub fn count(text: &str) -> usize {
    count_impl::<Chunk>(text.as_bytes())
}

#[inline]
pub fn count_inline(text: &str) -> usize {
    count_impl::<Chunk>(text.as_bytes())
}

/// Converts from byte-index to char-index in a string slice.
///
/// If the byte is in the middle of a multi-byte char, returns the index of
/// the char that the byte belongs to.
///
/// Any past-the-end index will return the one-past-the-end char index.
///
/// Runs in O(N) time.
#[inline]
pub fn from_byte_idx(text: &str, byte_idx: usize) -> usize {
    let bytes = text.as_bytes();

    // Ensure the index is either a char boundary or is off the end of
    // the text.
    let mut i = byte_idx;
    while Some(true) == bytes.get(i).map(|byte| (*byte & 0xC0) == 0x80) {
        i -= 1;
    }

    count_impl::<Chunk>(&bytes[0..i.min(bytes.len())])
}

/// Converts from char-index to byte-index in a string slice.
///
/// Any past-the-end index will return the one-past-the-end byte index.
///
/// Runs in O(N) time.
#[inline]
pub fn to_byte_idx(text: &str, char_idx: usize) -> usize {
    to_byte_idx_impl::<Chunk>(text, char_idx)
}

//-------------------------------------------------------------

#[inline(always)]
fn to_byte_idx_impl<T: ByteChunk>(text: &str, char_idx: usize) -> usize {
    // Get `middle` so we can do more efficient chunk-based counting.
    // We can't use this to get `end`, however, because the start index of
    // `end` actually depends on the accumulating char counts during the
    // counting process.
    let (start, middle, _) = unsafe { text.as_bytes().align_to::<T>() };

    let mut byte_count = 0;
    let mut char_count = 0;

    // Take care of any unaligned bytes at the beginning.
    for byte in start.iter() {
        char_count += ((*byte & 0xC0) != 0x80) as usize;
        if char_count > char_idx {
            break;
        }
        byte_count += 1;
    }

    // Process chunks in the fast path.
    let mut chunks = middle;
    let mut max_round_len = char_idx.saturating_sub(char_count) / T::MAX_ACC;
    while max_round_len > 0 && !chunks.is_empty() {
        // Choose the largest number of chunks we can do this round
        // that will neither overflow `max_acc` nor blast past the
        // char we're looking for.
        let round_len = T::MAX_ACC.min(max_round_len).min(chunks.len());
        max_round_len -= round_len;
        let round = &chunks[..round_len];
        chunks = &chunks[round_len..];

        // Process the chunks in this round.
        let mut acc = T::zero();
        for chunk in round.iter() {
            acc = acc.add(chunk.bitand(T::splat(0xc0)).cmp_eq_byte(0x80));
        }
        char_count += (T::SIZE * round_len) - acc.sum_bytes();
        byte_count += T::SIZE * round_len;
    }

    // Process chunks in the slow path.
    for chunk in chunks.iter() {
        let new_char_count =
            char_count + T::SIZE - chunk.bitand(T::splat(0xc0)).cmp_eq_byte(0x80).sum_bytes();
        if new_char_count >= char_idx {
            break;
        }
        char_count = new_char_count;
        byte_count += T::SIZE;
    }

    // Take care of any unaligned bytes at the end.
    let end = &text.as_bytes()[byte_count..];
    for byte in end.iter() {
        char_count += ((*byte & 0xC0) != 0x80) as usize;
        if char_count > char_idx {
            break;
        }
        byte_count += 1;
    }

    byte_count
}

#[inline(always)]
pub(crate) fn count_impl<T: ByteChunk>(text: &[u8]) -> usize {
    assert_eq!(16, core::mem::size_of::<T>());
    if text.len() < T::SIZE {
        // Bypass the more complex routine for short strings, where the
        // complexity hurts performance.
        text.iter()
            .map(|byte| ((byte & 0xC0) != 0x80) as usize)
            .sum()
    } else {
        // Get `middle` for more efficient chunk-based counting.
        let (start, middle, end) = unsafe { text.align_to::<T>() };

        let mut inv_count = 0;

        // Take care of unaligned bytes at the beginning.
        for byte in start.iter() {
            inv_count += ((byte & 0xC0) == 0x80) as usize;
        }

        // Take care of the middle bytes in big chunks.
        for chunks in middle.chunks(T::MAX_ACC) {
            let mut acc = T::zero();
            for chunk in chunks.iter() {
                acc = acc.add(chunk.bitand(T::splat(0xc0)).cmp_eq_byte(0x80));
            }
            inv_count += acc.sum_bytes();
        }

        // Take care of unaligned bytes at the end.
        for byte in end.iter() {
            inv_count += ((byte & 0xC0) == 0x80) as usize;
        }

        text.len() - inv_count
    }
}
