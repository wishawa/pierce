/*! Avoid double indirection in nested smart pointers.

The [`Pierced`] stuct allows you to cache the deref result of doubly-nested smart pointers.

# Quick Example

```
# use std::sync::Arc;
# use pierced::Pierced;
let vec: Vec<i32> = vec![1, 2, 3];
let arc_vec = Arc::new(vec);
let pierced = Pierced::new(arc_vec);

// Here, the execution jumps directly to the slice. (Without Pierced it would have to jump to the Vec first, than from the Vec to the slice).
pierced.get(0).unwrap();
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

// Here, the `Arc<Vec<i32>>` is first dereferenced to the `Vec<i32>`, then the Vec is dereferenced to the underlying i32 slice.
arc_vec.get(0).unwrap();
```

# Pierced

The [`Pierced`] struct, provided by this crate,
can help reduce the performance cost of nesting smart pointers by **caching the deref result**.
We double-deref the nested smart pointer at the start, storing the address where the inner pointer points to.
We can then access the underlying data by just jumping to the stored address. One jump.

Here's a diagram of what it *might* look like.

```text
             ┌───────────────────────────┬───────────────────────────────┬──────────────────────────────────────────┐
             │ Stack                     │ Heap                          │ Heap                                     │
             │ (Probably)                │ (Probably)                    │ (Probably)                               │
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
│ Pierced<T> │                           │                               │                                          │
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

`Pierced<T>` can be created with `Pierced::new(...)`. `T` should be a doubly-nested pointer (e.g. `Arc<Vec<_>>`, `Box<Box<_>>`).

[deref][Deref::deref]-ing a `Pierced<T>` returns `&<T::Target as Deref>::Target`,
i.e. the deref target of the deref target of T (the outer pointer that is wrapped by Pierced),
i.e. the deref target of the inner pointer.

You can obtain a borrow of just T (the outer pointer) using `.borrow_inner()`.

See [the quick example above](#quick_example)

See the docs at [`Pierced`] for more details.

## Deeper Nesting

A `Pierced` reduces two jumps to one.
If you have deeper nestings, you can wrap it multiple times.

```
# use pierced::Pierced;
let triply_nested: Box<Box<Box<i32>>> = Box::new(Box::new(Box::new(42)));
assert_eq!(***triply_nested, 42); // <- Three jumps!
let pierced_twice = Pierced::new(Pierced::new(triply_nested));
assert_eq!(*pierced_twice, 42); // <- Just one jump!
```

# Performance

Double indirection is probably not so bad for most use cases.
But in some cases, using Pierced can provide a significant performance improvement.

In our benchmark reading every value inside an `Arc<Vec<i32>>`,
the Pierced vesion (`Pierced<Arc<Vec<i32>>>`) **took 10-15% less time** than just `Arc<Vec<i32>>.

In our benchmark reading every value inside a `Box<Vec<i32>>`,
the Pierced vesion (`Pierced<Box<Vec<i32>>>`) **took 2-3% less time** than just `Box<Vec<i32>>.

In our benchmark repeatedly reading value from an `Arc<Box<i32>>`,
the Pierced version (`Pierced<Arc<Box<i32>>>`) **is slower, taking around 4 more nanoseconds each read** than just `Arc<Box<i32>>`.

You should try and benchmark your own use case to decide if you should use `Pierced`.

See the benchmarks' code [here](https://github.com/wishawa/pierced/tree/src/bin/benchmark/main.rs).

# Limitations

## Immutable Only

Pierced only work with immutable data.
**Mutability is not supported at all** because I'm pretty sure it would be impossible to implement soundly.
(If you have an idea please share.)

## Possibly Incorrect

Pierced is **safe, but not neccessarily correct**.
You will not run into memory safety issues (i.e. no "unsafety"),
but you may get the wrong result when deref-ing.

For Pierced to always deref to the correct result,
it must be true for **both** the outer and inner pointer that
**an immutable version of the pointer derefs to the same target every time**.

This condition is met by most common smart pointers, including (but not limited to) [Box], [Vec], [String], [Arc][std::sync::Arc], [Rc][std::rc::Rc].
In fact, I have never seen any real-world pointer that doesn't meet this condition. If you know one, please do share.

Here's an example where this invariant is **not** upheld:

```should_panic
# use pierced::Pierced;
# use std::ops::Deref;
use std::time::{SystemTime, Duration, UNIX_EPOCH};

// A really strange smart pointer that points to different strs based on the current time.
struct WeirdPointer;
impl Deref for WeirdPointer {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        if SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() % 2 == 0 {
            "even unix timestamp"
        }
        else {
            "odd unix timestamp"
        }
    }
}
let weird_pierced = Pierced::new(
    Box::new(WeirdPointer)
);

let first = &*weird_pierced;
std::thread::sleep(Duration::from_secs(1));

// Having slept for 1 second we now expect the WeirdPointer to dereference to another str.
// But no. The next line will fail because Pierced will still return the same cached target, unaware that WeirdPointer now deref to a different address.
assert_ne!(&*weird_pierced, first);
```

## Fallback
Pierced only cache the target address when it is possible to do so safely.
For that to be true, **the inner pointer must points somewhere outside the outer pointer**, (e.g. somehwere else on the heap or in the static region).

This condition is met by most common smart pointers, including (but not limited to) [Box], [Vec], [String], [Arc][std::sync::Arc], [Rc][std::rc::Rc].

If Pierced is unable to cache the target safely, it falls back to calling deref twice every time. You can use `.is_cacached()` to check.
*/

use std::{mem::size_of, ops::Deref, ptr::NonNull};

#[derive(Debug)]
pub struct Pierced<T>
where
    T: Deref,
    T::Target: Deref,
{
    outer: T,
    target: Option<NonNull<<T::Target as Deref>::Target>>,
}

fn is_cachable<T>(outer: &T, target: &<T::Target as Deref>::Target) -> bool
where
    T: Deref,
    T::Target: Deref,
{
    fn points_outside(start: usize, size: usize, ptr: usize) -> bool {
        ptr < start || ptr >= start + size
    }

    let outer_casted = outer as *const T as usize;
    points_outside(
        outer_casted,
        size_of::<T>(),
        target as *const <T::Target as Deref>::Target as *const () as usize,
    )
}

impl<T> Pierced<T>
where
    T: Deref,
    T::Target: Deref,
{
    /** Create a new Pierced

    Create a Pierced out of the given nested pointer.
    This method derefs T twice and cache the address where the inner pointer points to.

    Deref-ing the create Pierced returns the cached reference directly. `deref` is not called on T.
     */
    #[inline]
    pub fn new(outer: T) -> Self {
        let inner: &T::Target = outer.deref();
        let target: &<T::Target as Deref>::Target = inner.deref();

        let target = if is_cachable(&outer, target) {
            Some(NonNull::from(target))
        } else {
            None
        };

        Self { outer, target }
    }

    /** Borrow the outer pointer T

    You can then call the methods on &T.

    You can even call `deref` twice on &T directly to bypass Pierced's cache:
    ```
    # use pierced::Pierced;
    use std::ops::Deref;
    let pierced = Pierced::new(Box::new(Box::new(5)));
    let outer: &Box<Box<i32>> = pierced.borrow_outer();
    let inner: &Box<i32> = outer.deref();
    let target: &i32 = inner.deref();
    assert_eq!(*target, 5);
    ```

    */
    #[inline]
    pub fn borrow_outer(&self) -> &T {
        &self.outer
    }

    /** Get the outer pointer T out.

    Like `into_inner()` elsewhere, this consumes the Pierced and return the wrapped pointer.
     */
    #[inline]
    pub fn into_outer(self) -> T {
        self.outer
    }

    /** Whether or not Pierced has cached the target

    Pierced only cache the target when it is safe to do so. See the "Limitations" section at the [crate docs][crate].

    This method returns true if the target is cached, false if Pierced is falling back to double-derefing every time.
    */
    #[inline]
    pub fn is_cached(&self) -> bool {
        self.target.is_some()
    }
}

impl<T> Clone for Pierced<T>
where
    T: Deref + Clone,
    T::Target: Sized + Deref,
{
    #[inline]
    fn clone(&self) -> Self {
        Self::new(self.outer.clone())
    }
}

impl<T> Deref for Pierced<T>
where
    T: Deref,
    T::Target: Sized + Deref,
{
    type Target = <T::Target as Deref>::Target;
    #[inline]
    fn deref<'a>(&'a self) -> &'a Self::Target {
        match self.target.as_ref() {
            Some(ptr) => {
                unsafe { ptr.as_ref() }
                /* SAFETY:
                The Pierced must still be alive (not dropped) when this is called,
                and thus the outer pointer must still be alive.

                The Pierced might be moved, but moving the Pierced only moves the outer pointer.
                And if the target points to somewhere in the outer pointer, we wouldn't have cached it (None case below).

                The inner pointer (which is the deref result of the outer pointer) must last as long as the outer pointer,
                so it must still be alive too.

                The target (which is the deref result of the inner pointer) must last as long as the inner pointer,
                so it must still be alive too.

                It might seem that interior mutability can cause an issue,
                but it actually is impossible to get long-living reference out of a RefCell or Mutex,
                so you can't deref to anything inside an interior-mutable cell anyway.
                */
            }
            None => &self.outer,
        }
    }
}

impl<T> AsRef<<T::Target as Deref>::Target> for Pierced<T>
where
    T: Deref,
    T::Target: Sized + Deref,
{
    #[inline]
    fn as_ref(&self) -> &<T::Target as Deref>::Target {
        &**self
    }
}

impl<T> Default for Pierced<T>
where
    T: Deref + Default,
    T::Target: Sized + Deref,
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
        let p1 = Pierced::new(a);
        let p2 = p1.clone();
        p1.get(0).unwrap().borrow_mut().add_assign(5);
        assert_eq!(*p2.get(0).unwrap().borrow(), 6);
        assert_eq!(p1.is_cached(), true);
    }

    #[test]
    fn test_rc_string() {
        use std::rc::Rc;

        let v = String::from("hello world");
        let a = Rc::new(v);
        let pierced = Pierced::new(a);
        assert_eq!(&*pierced, "hello world");
        assert_eq!(pierced.is_cached(), true);
    }

    #[test]
    fn test_box_vec() {
        let v = vec![1, 2, 3];
        let a = Box::new(v);
        let pierced = Pierced::new(a);
        assert_eq!(*pierced.get(2).unwrap(), 3);
        assert_eq!(pierced.is_cached(), true);
    }

    #[test]
    fn test_triply_nested() {
        let b: Box<Box<Box<i32>>> = Box::new(Box::new(Box::new(42)));
        let pierced_once = Pierced::new(b);
        assert_eq!(*Box::deref(Pierced::deref(&pierced_once)), 42);
        let pierced_twice = Pierced::new(pierced_once);
        assert_eq!(*Pierced::deref(&pierced_twice), 42);
        assert_eq!(pierced_twice.is_cached(), true);
    }

    #[test]
    fn test_weird_pointer() {
        use std::cell::RefCell;

        struct WeirdPointer {
            inner: RefCell<bool>,
        }
        impl Deref for WeirdPointer {
            type Target = str;
            fn deref(&self) -> &Self::Target {
                let mut b = self.inner.borrow_mut();
                if *b {
                    *b = false;
                    "hello"
                } else {
                    *b = true;
                    "world"
                }
            }
        }
        let weird_normal = Box::new(WeirdPointer {
            inner: RefCell::new(true),
        });
        let weird_pierced = Pierced::new(Box::new(WeirdPointer {
            inner: RefCell::new(true),
        }));
        assert_eq!(weird_pierced.is_cached(), true);
        assert_eq!(&**weird_normal, "hello");
        assert_eq!(&*weird_pierced, "hello");
        assert_eq!(&**weird_normal, "world");
        assert_eq!(&*weird_pierced, "hello");
    }

    struct StackPtr<T>(T);
    impl<T> Deref for StackPtr<T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    #[test]
    fn test_stack_stack() {
        let a = 41;
        let b = StackPtr(a);
        let c = StackPtr(b);
        let p = Pierced::new(c);

        assert_eq!(p.is_cached(), false);
    }
    #[test]
    fn test_box_stack() {
        let a = 41;
        let b = StackPtr(a);
        let c = Box::new(b);
        let p = Pierced::new(c);

        assert_eq!(p.is_cached(), true);
    }
    #[test]
    fn test_stack_box() {
        let a = 41;
        let b = Box::new(a);
        let c = StackPtr(b);
        let p = Pierced::new(c);

        assert_eq!(p.is_cached(), true);
    }

    #[test]
    fn test_box_box() {
        let a = 41;
        let b = Box::new(a);
        let c = Box::new(b);
        let p = Pierced::new(c);

        assert_eq!(p.is_cached(), true);
    }
}
