use std::marker::PhantomData;

pub fn into_phantom_data<T>(_: T) -> PhantomData<T> {
    PhantomData
}
