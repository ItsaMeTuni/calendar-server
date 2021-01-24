use std::ptr::replace;
use serde::export::fmt::Debug;

/// An iterator that merges two sorted iterators
/// in order. The output is undefined if any of
/// the supplied iterators is not ordered.
///
/// # Example
/// ```
/// let a = vec![1, 3, 5].into_iter();
/// let b = vec![2, 4, 6].into_iter();
///
/// let result = a.chain_ordered(b).collect::<Vec<_>>();
///
/// assert_eq!(result, [1, 2, 3, 4, 5, 6]);
/// ```
pub struct MergeOrdered<A, B>
    where
        A: Iterator,
        A::Item: Ord,
        B: Iterator<Item = A::Item>,
{
    a: A,
    b: B,

    cache_a: Option<A::Item>,
    cache_b: Option<B::Item>,

    first_run: bool,
}

impl<A, B> Iterator for MergeOrdered<A, B>
    where
        A: Iterator,
        A::Item: Ord + Debug,
        B: Iterator<Item = A::Item>,
{
    type Item = A::Item;

    fn next(&mut self) -> Option<Self::Item>
    {
        unsafe
        {
            // Get first elements in the first iteration.
            if self.first_run
            {
                replace((&mut self.cache_a as *mut _), self.a.next());
                replace((&mut self.cache_b as *mut _), self.b.next());
                self.first_run = false;
            }

            let cache_ref_a = self.cache_a.as_ref();
            let cache_ref_b = self.cache_b.as_ref();

            // Return the greater of the elements. If one of
            // them is None, return the other. If both of them
            // are None, return None.
            // Whenever we return one element, we already cache it's
            // iterator's next element.
            match cache_ref_a
            {
                None => {
                    return replace((&mut self.cache_b as *mut _), self.b.next());
                },
                Some(a) => match cache_ref_b {
                    None => {
                        return replace((&mut self.cache_a as *mut _), self.a.next());
                    },
                    Some(b) => {
                        if a <= b
                        {
                            return replace((&mut self.cache_a as *mut _), self.a.next());
                        }
                        else
                        {
                            return replace((&mut self.cache_b as *mut _), self.b.next());
                        }
                    },
                },
            }
        }
    }

}

pub trait MergeOrderedTrait: Sized + Iterator
    where Self::Item: Ord
{
    fn merge_ordered<U>(self, b: U) -> MergeOrdered<Self, U>
        where U: Iterator<Item = Self::Item>;

}

impl<T> MergeOrderedTrait for T
    where
        T: Iterator,
        T::Item: Ord
{
    /// Merge two ordered iterators in order.
    ///
    /// The output is undefined if any of the
    /// supplied iterators is not ordered.
    fn merge_ordered<U>(self, b: U) -> MergeOrdered<T, U>
        where U: Iterator<Item = T::Item>
    {
        MergeOrdered {
            a: self,
            b,
            cache_a: None,
            cache_b: None,
            first_run: true,
        }
    }
}

#[cfg(test)]
mod test
{
    use crate::iter_helpers::MergeOrderedTrait;
    use itertools::Itertools;

    #[test]
    fn test_interspaced()
    {
        let a = vec![1, 3, 5].into_iter();
        let b = vec![2, 4, 6].into_iter();

        let result = a.merge_ordered(b).collect_vec();

        assert_eq!(result, [1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_asymmetric()
    {
        let a = vec![1, 3, 5, 7, 9].into_iter();
        let b = vec![2, 4, 6].into_iter();

        let result = a.merge_ordered(b).collect_vec();

        assert_eq!(result, [1, 2, 3, 4, 5, 6, 7, 9]);
    }

    #[test]
    fn test_with_duplicates()
    {
        let a = vec![1, 3, 5, 7, 9, 9].into_iter();
        let b = vec![2, 4, 5, 5, 6].into_iter();

        let result = a.merge_ordered(b).collect_vec();

        assert_eq!(result, [1, 2, 3, 4, 5, 5, 5, 6, 7, 9, 9]);
    }

    #[test]
    fn test_with_empty_a()
    {
        let a = vec![].into_iter();
        let b = vec![2, 4, 5, 5, 6].into_iter();

        let result = a.merge_ordered(b).collect_vec();

        assert_eq!(result, [2, 4, 5, 5, 6]);
    }

    #[test]
    fn test_with_empty_b()
    {
        let a = vec![1, 3, 5, 7, 9, 9].into_iter();
        let b = vec![].into_iter();

        let result = a.merge_ordered(b).collect_vec();

        assert_eq!(result, [1, 3, 5, 7, 9, 9]);
    }
}