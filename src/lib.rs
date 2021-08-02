/*! Avoid double indirection in nested smart pointers.

The [`Pierce`] stuct allows you to cache the deref result of doubly-nested smart pointers.

# Quick Example

```
# use std::sync::Arc;
# use pierce::Pierce;
let vec: Vec<i32> = vec![1, 2, 3];
let arc_vec = Arc::new(vec);
let pierce = Pierce::new(arc_vec);

// Here, the execution jumps directly to the slice to call `.get(...)`.
// Without Pierce it would have to jump to the Vec first,
// than from the Vec to the slice.
pierce.get(0).unwrap();
```

# Nested Smart Pointers

Smart Pointers can be nested to - in a way - combine their functionalities.
For example, with `Arc<Vec<i32>>`, a slice of i32 is managed by the wrapping [`Vec`] that is wrapped again by the [`Arc`][std::sync::Arc].

However, nesting comes at the cost of **double indirection**:
when we want to access the underlying data,
we must first follow the outer pointer to where the inner pointer lies,
then follow the inner pointer to where the underlying data is. Two [Deref]-ings. Two jumps.

```
# use std::sync::Arc;
let vec: Vec<i32> = vec![1, 2, 3];
let arc_vec = Arc::new(vec);

// Here, the `Arc<Vec<i32>>` is first dereferenced to the `Vec<i32>`,
// then the Vec is dereferenced to the underlying i32 slice,
// on which `.get(...)` is called.
arc_vec.get(0).unwrap();
```

# Pierce

The [`Pierce`] struct, provided by this crate,
can help reduce the performance cost of nesting smart pointers by **caching the deref result**.
We double-deref the nested smart pointer at the start, storing the address where the inner pointer points to.
We can then access the underlying data by just jumping to the stored address. One jump.

Here's a diagram of what it *might* look like.

```text
             ┌───────────────────────────┬───────────────────────────────┬──────────────────────────────────────────┐
             │ Stack                     │ Heap                          │ Heap                                     │
┌────────────┼───────────────────────────┼───────────────────────────────┼──────────────────────────────────────────┤
│ T          │                           │                               │                                          │
│            │  ┌──────────────────┐     │     ┌───────────────────┐     │    ┌──────────────────────────────────┐  │
│            │  │Outer Pointer     │     │     │Inner Pointer      │     │    │Target                            │  │
│            │  │                  │     │     │                   │     │    │                                  │  │
│            │  │        T ────────────────────────► T::Target ─────────────────► <T::Target as Deref>::Target   │  │
│            │  │                  │     │     │                   │     │    │                                  │  │
│            │  └──────────────────┘     │     └───────────────────┘     │    └──────────────────────────────────┘  │
│            │                           │                               │                                          │
├────────────┼───────────────────────────┼───────────────────────────────┼──────────────────────────────────────────┤
│ Pierce<T>  │                           │                               │                                          │
│            │  ┌──────────────────┐     │     ┌───────────────────┐     │    ┌──────────────────────────────────┐  │
│            │  │Outer Pointer     │     │     │Inner Pointer      │     │    │Target                            │  │
│            │  │                  │     │     │                   │     │    │                                  │  │
│            │  │        T ────────────────────────► T::Target ─────────────────► <T::Target as Deref>::Target   │  │
│            │  │                  │     │     │                   │     │    │                ▲                 │  │
│            │  ├──────────────────┤     │     └───────────────────┘     │    └────────────────│─────────────────┘  │
│            │  │Cache             │     │                               │                     │                    │
│            │  │                  │     │                               │                     │                    │
│            │  │       ptr ───────────────────────────────────────────────────────────────────┘                    │
│            │  │                  │     │                               │                                          │
│            │  └──────────────────┘     │                               │                                          │
│            │                           │                               │                                          │
└────────────┴───────────────────────────┴───────────────────────────────┴──────────────────────────────────────────┘
```

# Usage

`Pierce<T>` can be created with `Pierce::new(...)`. `T` should be a doubly-nested pointer (e.g. `Arc<Vec<_>>`, `Box<Box<_>>`).

[deref][Deref::deref]-ing a `Pierce<T>` returns `&<T::Target as Deref>::Target`,
i.e. the deref target of the deref target of `T` (the outer pointer that is wrapped by Pierce),
i.e. the deref target of the inner pointer.

You can also obtain a borrow of just `T` (the outer pointer) using `.borrow_inner()`.

See the docs at [`Pierce`] for more details.

## Deeper Nesting

A `Pierce` reduces two jumps to one.
If you have deeper nestings, you can wrap it multiple times.

```
# use pierce::Pierce;
let triply_nested: Box<Box<Box<i32>>> = Box::new(Box::new(Box::new(42)));
assert_eq!(***triply_nested, 42); // <- Three jumps!
let pierce_twice = Pierce::new(Pierce::new(triply_nested));
assert_eq!(*pierce_twice, 42); // <- Just one jump!
```

# Benchmarks

These benchmarks probably won't represent your use case at all because:
* They are engineered to make Pierce look good.
* Compiler optimizations are hard to control.
* CPU caches and predictions are hard to control. (I bet the figures will be very different on your CPU.)
* Countless other reasons why you shouldn't trust synthetic benchmarks.

*Do your own benchmarks on real-world uses*.

That said, here are my results:

**Benchmark 1**: Read items from a `Box<Vec<usize>>`, with simulated memory fragmentation.

**Benchmark 2**: Read items from a `SlowBox<Vec<usize>>`. `SlowBox` deliberately slow down `deref()` call greatly.

**Benchmark 3**: Read several `Box<Box<i64>>`.

Time taken by `Pierce<T>` version compared to `T` version.

| Run		| Benchmark 1		| Benchmark 2	 	| Benchmark 3       |
|-----------|-------------------|-------------------|-------------------|
| 1			| -40.23%			| -99.69%			| -5.68%            |
| 2			| -40.59%			| -99.69%			| -5.16%            |
| 3			| -40.70%			| -99.68%			| +2.69%            |
| 4			| -39.85%			| -99.68%			| -5.35%            |
| 5			| -38.90%			| -99.71%			| -5.02%            |
| 6			| -39.12%			| -99.69%			| -5.53%            |
| 7			| -40.51%			| -99.69%			| -6.09%            |
| 8			| -26.99%			| -99.71%			| -6.43%            |

See the benchmarks' code [here](https://github.com/wishawa/pierce/tree/main/src/bin/benchmark/main.rs).

# Limitations

## Immutable Only

Pierce only work with immutable data.
**Mutability is not supported at all** because I'm pretty sure it would be impossible to implement soundly.
(If you have an idea please share.)

## Requires `StableDeref`

Pointer wrapped by Pierce must be [`StableDeref`].
If your pointer type meets the conditions required, you can `unsafe impl StableDeref for T {}` on it.
The trait is re-exported at `pierce::StableDeref`.

The vast majority of pointers are `StableDeref`,
including [Box], [Vec], [String], [Rc][std::rc::Rc], [Arc][std::sync::Arc].
*/

use std::{ops::Deref, ptr::NonNull};

pub use stable_deref_trait::StableDeref;

/** Cache doubly-nested pointers.

A `Pierce<T>` stores `T` along with a cached pointer to `<T::Target as Deref>::Target`.
*/
pub struct Pierce<T>
where
    T: StableDeref,
    T::Target: StableDeref,
{
    outer: T,
    target: NonNull<<T::Target as Deref>::Target>,
}

impl<T> Pierce<T>
where
    T: StableDeref,
    T::Target: StableDeref,
{
    /** Create a new Pierce.

    Create a Pierce out of the given nested pointer.
    This method derefs `T` twice and cache the address where the inner pointer points to.

    Deref-ing the created Pierce returns the cached reference directly. `deref` is not called on `T`.
     */
    #[inline]
    pub fn new(outer: T) -> Self {
        let inner: &T::Target = outer.deref();
        let target: &<T::Target as Deref>::Target = inner.deref();
        let target = NonNull::from(target);
        Self { outer, target }
    }

    /** Borrow the outer pointer `T`.

    You can then call the methods on `&T`.

    You can even call `deref` twice on `&T` yourself to bypass Pierce's cache:
    ```
    # use pierce::Pierce;
    use std::ops::Deref;
    let pierce = Pierce::new(Box::new(Box::new(5)));
    let outer: &Box<Box<i32>> = pierce.borrow_outer();
    let inner: &Box<i32> = outer.deref();
    let target: &i32 = inner.deref();
    assert_eq!(*target, 5);
    ```

    */
    #[inline]
    pub fn borrow_outer(&self) -> &T {
        &self.outer
    }

    /** Get the outer pointer `T` out.

    Like `into_inner()` elsewhere, this consumes the Pierce and return the wrapped pointer.
     */
    #[inline]
    pub fn into_outer(self) -> T {
        self.outer
    }
}

unsafe impl<T> Send for Pierce<T>
where
    T: StableDeref + Send,
    T::Target: StableDeref,
    <T::Target as Deref>::Target: Sync,
{
}

unsafe impl<T> Sync for Pierce<T>
where
    T: StableDeref + Sync,
    T::Target: StableDeref,
    <T::Target as Deref>::Target: Sync,
{
}

unsafe impl<T> StableDeref for Pierce<T>
where
    T: StableDeref,
    T::Target: StableDeref,
{
}

impl<T> Clone for Pierce<T>
where
    T: StableDeref + Clone,
    T::Target: StableDeref,
{
    #[inline]
    fn clone(&self) -> Self {
        Self::new(self.outer.clone())
    }
}

impl<T> Deref for Pierce<T>
where
    T: StableDeref,
    T::Target: StableDeref,
{
    type Target = <T::Target as Deref>::Target;
    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { self.target.as_ref() }
        /* SAFETY:
        The Pierce must still be alive (not dropped) when this is called,
        and thus the outer pointer must still be alive.

        The Pierce might be moved, but it must be StableDeref so moving is ok.

        The inner pointer (which is the deref result of the outer pointer) must last as long as the outer pointer,
        so it must still be alive too.

        The target (which is the deref result of the inner pointer) must last as long as the inner pointer,
        so it must still be alive too.

        It might seem that interior mutability can cause an issue,
        but it actually is impossible to get long-living reference out of a RefCell or Mutex,
        so you can't deref to anything inside an interior-mutable cell anyway.
        */
    }
}

impl<T> AsRef<<T::Target as Deref>::Target> for Pierce<T>
where
    T: StableDeref,
    T::Target: StableDeref,
{
    #[inline]
    fn as_ref(&self) -> &<T::Target as Deref>::Target {
        &**self
    }
}

impl<T> Default for Pierce<T>
where
    T: StableDeref + Default,
    T::Target: StableDeref,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arc_vec() {
        use std::cell::RefCell;
        use std::ops::AddAssign;
        use std::sync::Arc;

        let v = vec![RefCell::new(1), RefCell::new(2)];
        let a = Arc::new(v);
        let p1 = Pierce::new(a);
        let p2 = p1.clone();
        p1.get(0).unwrap().borrow_mut().add_assign(5);
        assert_eq!(*p2.get(0).unwrap().borrow(), 6);
    }

    #[test]
    fn test_rc_string() {
        use std::rc::Rc;

        let v = String::from("hello world");
        let a = Rc::new(v);
        let pierce = Pierce::new(a);
        assert_eq!(&*pierce, "hello world");
    }

    #[test]
    fn test_box_vec() {
        let v = vec![1, 2, 3];
        let a = Box::new(v);
        let pierce = Pierce::new(a);
        assert_eq!(*pierce.get(2).unwrap(), 3);
    }

    #[test]
    fn test_triply_nested() {
        let b: Box<Box<Box<i32>>> = Box::new(Box::new(Box::new(42)));
        let pierce_once = Pierce::new(b);
        assert_eq!(*Box::deref(Pierce::deref(&pierce_once)), 42);
        let pierce_twice = Pierce::new(pierce_once);
        assert_eq!(*Pierce::deref(&pierce_twice), 42);
    }

    #[test]
    fn test_send() {
        use std::sync::Arc;
        let p1 = Pierce::new(Arc::new(String::from("asdf")));
        let p2 = p1.clone();
        let h1 = std::thread::spawn(move || {
            assert_eq!(&*p1, "asdf");
        });
        let h2 = std::thread::spawn(move || {
            assert_eq!(&*p2, "asdf");
        });
        h1.join().unwrap();
        h2.join().unwrap();
    }
    #[test]
    fn test_sync() {
        let p: Pierce<Box<String>> = Pierce::new(Box::new(String::from("hello world")));
        let p1: &'static Pierce<Box<String>> = Box::leak(Box::new(p));
        let p2: &'static Pierce<Box<String>> = p1;
        let h1 = std::thread::spawn(move || {
            assert_eq!(&**p1, "hello world");
        });
        let h2 = std::thread::spawn(move || {
            assert_eq!(&**p2, "hello world");
        });
        h1.join().unwrap();
        h2.join().unwrap();
    }

    #[test]
    fn test_size_of() {
        use std::mem::size_of;
        use std::sync::Arc;
        fn inner_test<T>()
        where
            T: StableDeref,
            T::Target: StableDeref,
        {
            assert_eq!(
                size_of::<T>() + size_of::<&<T::Target as Deref>::Target>(),
                size_of::<Pierce<T>>()
            );
        }
        inner_test::<Box<Vec<i32>>>();
        inner_test::<Box<Box<i32>>>();
        inner_test::<Arc<Vec<i32>>>();
        inner_test::<Box<Arc<i32>>>();
    }
}
