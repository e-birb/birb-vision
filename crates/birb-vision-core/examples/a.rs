

struct A<'a> {
    n: &'a i32,
}

struct Callback<F> {
    f: Box<F>,
}

impl<F> Callback<F> {
    fn foo<T>(&self, t: T)
    where
        F: Fn(T),
    {
        (self.f)(t);
    }

    fn bar<T, R>(&self, f: impl FnMut(T) -> R)
    where
        F: Fn(T),
    {

    }
}

fn main() {
    let n = 42;

    let cb = Callback {
        f: Box::new(|_: A| {}),
    };

    let a = A { n: &n };
    cb.foo(a);
    cb.bar(|_| 42);
}