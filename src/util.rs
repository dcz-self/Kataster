#[macro_export]
macro_rules! assert_matches {
    ($expression:expr, $($pattern:tt)+) => {
        match $expression {
            $($pattern)+ => (),
            ref e => panic!("assertion failed: `{:?}` does not match `{}`", e, stringify!($($pattern)+)),
        }
    }
}

// Based on https://github.com/rust-lang/rust/issues/24000
/// Box<dyn Foo> also cannot be cloned,
/// so it must be wrapped in some type that can
pub struct PredicateContainer<T: 'static> {
    function: Box<dyn Predicate<T>>,
}

impl<T: 'static> Clone for PredicateContainer<T> {
    fn clone(&self) -> Self {
        PredicateContainer {
            function: self.function.clone_boxed(),
        }
    }
}

impl<T> PredicateContainer<T> {
    pub fn new<F: 'static + Send + Sync + Clone + Fn(&T) -> bool>(f: F) -> Self {
        PredicateContainer {
            function: Box::new(f),
        }
    }
    pub fn apply<'a, 'b>(&'a self, val: &'b T) -> bool {
        (self.function)(val)
    }
}

pub trait Predicate<T>: Fn(&T) -> bool + Send + Sync {
    fn clone_boxed(&self) -> Box<dyn Predicate<T>>;
}

impl<F, T: 'static> Predicate<T> for F
where
    F: 'static + Send + Sync + Clone + Fn(&T) -> bool,
{
    fn clone_boxed(&self) -> Box<dyn Predicate<T>> {
        Box::new(F::clone(self))
    }
}

/*
impl<T: 'static> Clone for Box<dyn Predicate<T>> {
    fn clone(&self) -> Self {
        self.clone_boxed()
    }
}*/


#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn clone_boxed() {
        let c = |t: &bool| *t;
        let _ = c.clone_boxed();
    }
    
    #[test]
    fn clone() {
        let c = |t: &bool| *t;
        let _ = c.clone();
    }
    
    #[test]
    fn clone_box() {
        let c = |t: &bool| *t;
        let _ = Box::new(c).clone();
    }

    #[test]
    fn clone_cont_dyn() {
        let c = |t: &bool| *t;
        let _ = PredicateContainer::new(c).clone();
    }

    #[test]
    fn clone_pred_boxed_dyn() {
        let c = |t: &bool| *t;
        let _ = <dyn Predicate<_>>::clone_boxed(&c);
    }
}
