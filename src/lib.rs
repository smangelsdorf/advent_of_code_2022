pub mod parser;

pub fn nothing() {}

pub trait MoreIter: Iterator + Sized {
    fn once_if_empty(self, value: Self::Item) -> OnceIfEmpty<Self>;
}

impl<I> MoreIter for I
where
    I: Iterator + Sized,
{
    fn once_if_empty(self, value: Self::Item) -> OnceIfEmpty<Self> {
        OnceIfEmpty {
            iter: self,
            any: false,
            item: Some(value),
        }
    }
}

pub struct OnceIfEmpty<I>
where
    I: Iterator + Sized,
{
    iter: I,
    any: bool,
    item: Option<I::Item>,
}

impl<I> Iterator for OnceIfEmpty<I>
where
    I: Iterator,
    I::Item: std::fmt::Debug,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(i) => {
                self.any = true;
                Some(i)
            }
            None => {
                if self.any {
                    None
                } else {
                    self.item.take()
                }
            }
        }
    }
}
