/// Calls naive_palette with partial equality as the predicate.
pub fn naive_palette_eq<T>(values: &[T]) -> ( Vec<T>, Vec<usize> )
    where T: Clone + PartialEq {
    naive_palette(values, |l, r| l == r)
}

/// Naively find all unique elements in a slice. Could be a lot better
/// by using sorting/hashing.
pub fn naive_palette<T, P>(values: &[T], mut predicate: P) -> ( Vec<T>, Vec<usize> )
    where
    T: Clone,
    P: FnMut(&T, &T) -> bool,
{
    let mut palette: Vec<T> = Vec::new();
    let mut indices: Vec<usize> = Vec::with_capacity(values.len());

    for value in values.iter() {
        let index = palette.iter().position(|palette_value| predicate(value, palette_value)).unwrap_or_else(|| {
            palette.push(value.clone());
            palette.len() - 1
        });
        indices.push(index);
    }

    (palette, indices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct Thing {
        name: String,
        ready: bool
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
        let (palette, indices) = naive_palette::<Thing, _>(&[], thing_eq);
        assert_eq!(palette, Vec::new());
        assert_eq!(indices, Vec::new());
    }

    #[test]
    fn single_element() {
        let (palette, indices) = naive_palette(&[ thing1() ], thing_eq);
        assert_eq!(palette, vec![ thing1() ]);
        assert_eq!(indices, vec![ 0 ]);
    }

    #[test]
    fn multiple_elements() {
        let (palette, indices) = naive_palette(&[ thing1(), thing2(), thing1() ], thing_eq);
        assert_eq!(palette, vec![ thing1(), thing2() ]);
        assert_eq!(indices, vec![ 0, 1, 0 ]);
    }
}
