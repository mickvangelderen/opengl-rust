extern crate num_traits;

use num_traits::cast::FromPrimitive;

pub struct Palette<E, I> {
    pub elements: Vec<E>,
    pub indices: Vec<I>,
}

impl<E, I> Palette<E, I>
where
    E: Clone,
    I: FromPrimitive,
{
    /// Naively find all unique elements in a slice. Could be a lot better
    /// by using sorting/hashing.
    pub fn naive<P>(values: &[E], mut predicate: P) -> Self
    where
        P: FnMut(&E, &E) -> bool,
    {

        let mut elements: Vec<E> = Vec::new();
        let mut indices: Vec<I> = Vec::with_capacity(values.len());

        for value in values.iter() {
            let index = elements
                .iter()
                .position(|elements_value| predicate(value, elements_value))
                .unwrap_or_else(|| {
                    elements.push(value.clone());
                    elements.len() - 1
                });
            indices.push(I::from_usize(index).unwrap());
        }

        Palette { elements, indices }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct Thing {
        name: String,
        ready: bool,
    }

    fn thing1() -> Thing {
        Thing {
            name: String::from("Something"),
            ready: true,
        }
    }

    fn thing2() -> Thing {
        Thing {
            name: String::from("Number two"),
            ready: true,
        }
    }

    fn thing_eq(l: &Thing, r: &Thing) -> bool {
        l == r
    }

    #[test]
    fn empty() {
        let Palette { elements, indices } = Palette::<Thing, usize>::naive(&[], thing_eq);
        assert_eq!(elements, Vec::new());
        assert_eq!(indices, Vec::new());
    }

    #[test]
    fn single_element() {
        let Palette { elements, indices } = Palette::<Thing, u16>::naive(&[ thing1() ], thing_eq);
        assert_eq!(elements, vec![thing1()]);
        assert_eq!(indices, vec![0]);
    }

    #[test]
    fn multiple_elements() {
        let Palette { elements, indices } = Palette::<Thing, u8>::naive(&[thing1(), thing2(), thing1()], thing_eq);
        assert_eq!(elements, vec![thing1(), thing2()]);
        assert_eq!(indices, vec![0, 1, 0]);
    }
}
