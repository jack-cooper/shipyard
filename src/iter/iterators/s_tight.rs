use super::{AbstractMut, Chunk1, ChunkExact1, InnerShiperator, IntoAbstract};
use crate::storage::Key;
#[cfg(feature = "parallel")]
use rayon::iter::plumbing::Producer;

/// Tight iterator over a single component.
pub struct Tight1<T: IntoAbstract> {
    pub(super) data: T::AbsView,
    pub(super) current: usize,
    pub(super) end: usize,
}

impl<T: IntoAbstract> Tight1<T> {
    /// Transform the iterator into a chunk iterator, returning `size` items at a time.
    ///
    /// Chunk will return a smaller slice at the end if `size` does not divide exactly the length.
    pub fn into_chunk(self, size: usize) -> Chunk1<T> {
        Chunk1 {
            data: self.data,
            current: self.current,
            end: self.end,
            step: size,
        }
    }
    /// Transform the iterator into a chunk exact iterator, returning `size` items at a time.
    ///
    /// ChunkExact will always return a slice with the same length.
    ///
    /// To get the remaining items (if any) use the `remainder` method.
    pub fn into_chunk_exact(self, size: usize) -> ChunkExact1<T> {
        ChunkExact1 {
            data: self.data,
            current: self.current,
            end: self.end,
            step: size,
        }
    }
}

impl<T: IntoAbstract> Iterator for Tight1<T> {
    type Item = <T::AbsView as AbstractMut>::Out;
    fn next(&mut self) -> Option<Self::Item> {
        let first = self.first_pass()?;
        self.post_process(first)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<T: IntoAbstract> DoubleEndedIterator for Tight1<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.end > self.current {
            self.end -= 1;
            // SAFE the index is valid and the iterator can only be created where the lifetime is valid
            Some(unsafe { self.data.get_data(self.end) })
        } else {
            None
        }
    }
}

impl<T: IntoAbstract> ExactSizeIterator for Tight1<T> {
    fn len(&self) -> usize {
        self.end - self.current
    }
}

#[cfg(feature = "parallel")]
impl<T: IntoAbstract> Producer for Tight1<T>
where
    <T::AbsView as AbstractMut>::Out: Send,
{
    type Item = <T::AbsView as AbstractMut>::Out;
    type IntoIter = Self;
    fn into_iter(self) -> Self::IntoIter {
        self
    }
    fn split_at(mut self, index: usize) -> (Self, Self) {
        let clone = Tight1 {
            data: self.data.clone(),
            current: self.current + index,
            end: self.end,
        };
        self.end = clone.current;
        (self, clone)
    }
}

impl<T: IntoAbstract> InnerShiperator for Tight1<T> {
    type Item = <T::AbsView as AbstractMut>::Out;
    type Index = usize;
    fn first_pass(&mut self) -> Option<(Self::Index, Self::Item)> {
        let current = self.current;
        if current < self.end {
            self.current += 1;
            let data = unsafe { self.data.get_data(current) };
            Some((current, data))
        } else {
            None
        }
    }
    #[inline]
    fn post_process(&mut self, (_, item): (usize, Self::Item)) -> Option<Self::Item> {
        Some(item)
    }
    #[inline]
    fn last_id(&self) -> Key {
        unsafe { self.data.id_at(self.current - 1) }
    }
}
