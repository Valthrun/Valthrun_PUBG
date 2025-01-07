use std::cell::{
    Ref,
    RefMut,
};

pub(crate) fn transpose_ref_opt<T>(x: Ref<'_, Option<T>>) -> Option<Ref<'_, T>> {
    if x.is_none() {
        None
    } else {
        Some(Ref::map(x, |x| x.as_ref().unwrap()))
    }
}

pub(crate) fn transpose_ref_mut_opt<T>(x: RefMut<'_, Option<T>>) -> Option<RefMut<'_, T>> {
    if x.is_none() {
        None
    } else {
        Some(RefMut::map(x, |x| x.as_mut().unwrap()))
    }
}
