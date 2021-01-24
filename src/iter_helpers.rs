use std::ptr::replace;
use serde::export::fmt::Debug;

pub struct ChainOrdered<A, B>
    where
        A: Iterator,
        A::Item: Ord,
        B: Iterator<Item = A::Item>,
{
    a: A,
    b: B,

    last_a: Option<A::Item>,
    last_b: Option<B::Item>,

    first_run: bool,
}

impl<A, B> Iterator for ChainOrdered<A, B>
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
            if self.first_run
            {
                replace((&mut self.last_a as *mut _), self.a.next());
                replace((&mut self.last_b as *mut _), self.b.next());
                self.first_run = false;
            }

            let a_ref = self.last_a.as_ref();
            let b_ref = self.last_b.as_ref();

            match a_ref
            {
                None => {
                    replace((&mut self.last_a as *mut _), self.a.next());

                    return replace((&mut self.last_b as *mut _), self.b.next());
                },
                Some(a) => match b_ref {
                    None => {
                        replace((&mut self.last_b as *mut _), self.b.next());

                        return replace((&mut self.last_a as *mut _), self.a.next());
                    },
                    Some(b) => {
                        if a <= b
                        {
                            return replace((&mut self.last_a as *mut _), self.a.next());
                        }
                        else
                        {
                            println!("Return b (replace with next).");
                            return replace((&mut self.last_b as *mut _), self.b.next());
                        }
                    },
                },
            }
        }
    }

}

pub trait ChainOrderedTrait: Sized + Iterator
    where Self::Item: Ord
{
    fn chain_ordered<U>(self, b: U) -> ChainOrdered<Self, U>
        where U: Iterator<Item = Self::Item>;

}

impl<T> ChainOrderedTrait for T
    where
        T: Iterator,
        T::Item: Ord
{
    fn chain_ordered<U>(self, b: U) -> ChainOrdered<T, U>
        where U: Iterator<Item = T::Item>
    {
        ChainOrdered {
            a: self,
            b,
            last_a: None,
            last_b: None,
            first_run: true,
        }
    }
}

#[cfg(test)]
mod test
{
    use crate::iter_helpers::ChainOrderedTrait;
    use itertools::Itertools;

    #[test]
    fn test_chain_ordered()
    {
        let a = vec![1, 3, 5].into_iter();
        let b = vec![2, 4, 6].into_iter();

        let result = a.chain_ordered(b).collect_vec();

        assert_eq!(result, [1, 2, 3, 4, 5, 6]);
    }
}