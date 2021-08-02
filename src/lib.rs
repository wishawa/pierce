/*! Avoid double indirection in nested smart pointers.

The [`Pierce`] stuct allows you to cache the deref result of doubly-nested smart pointers.

# Quick Example

```
# use std::sync::Arc;
# use pierce::Pierce;
let vec: Vec<i32> = vec![1, 2, 3];
let arc_vec = Arc::new(vec);
let pierce = Pierce::new(arc_vec);

// Here, the execution jumps directly to the slice to call `.get(...)`. (Without Pierce it would have to jump to the Vec first, than from the Vec to the slice).
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

// Here, the `Arc<Vec<i32>>` is first dereferenced to the `Vec<i32>`, then the Vec is dereferenced to the underlying i32 slice, on which `.get(...)` is called.
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
i.e. the deref target of the deref target of T (the outer pointer that is wrapped by Pierce),
i.e. the deref target of the inner pointer.

You can also obtain a borrow of just T (the outer pointer) using `.borrow_inner()`.

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

## Possibly Incorrect

Pierce is **safe, but not neccessarily correct**.
You will not run into memory safety issues (i.e. no "unsafety"),
but you may get the wrong result when deref-ing.

For Pierce to always deref to the correct result,
it must be true for **both** the outer and inner pointer that
**an immutable version of the pointer derefs to the same target every time**.

This condition is met by most common smart pointers, including (but not limited to) [Box], [Vec], [String], [Arc][std::sync::Arc], [Rc][std::rc::Rc].
In fact, I have never seen any real-world pointer that doesn't meet this condition. If you know one, please do share.

Here's an example where this invariant is **not** upheld:

```should_panic
# use pierce::Pierce;
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
let weird_pierce = Pierce::new(
    Box::new(WeirdPointer)
);

let first = &*weird_pierce;
std::thread::sleep(Duration::from_secs(1));

// Having slept for 1 second we now expect the WeirdPointer to dereference to another str.
// But no. The next line will fail because Pierce will still return the same cached target, unaware that WeirdPointer now deref to a different address.
assert_ne!(&*weird_pierce, first);
```

## Fallback

For Pierce to function optimally, **the final deref target must not be inside the outer pointer**,
(it should be e.g. somehwere else on the heap or in the static region).

This condition is met by most common smart pointers, including (but not limited to) [Box], [Vec], [String], [Arc][std::sync::Arc], [Rc][std::rc::Rc].

For pointers that don't meet this condition,
Pierce pin it to the heap using `Box` to give it a stable address,
so that the cache would not be left dangling if the Pierce (and the outer pointer in it) is moved.

You should avoid using Pierce if your doubly-nested pointer points to itself anyway.
*/

use std::{mem::size_of, ops::Deref, ptr::NonNull};

pub struct Pierce<T>
where
    T: Deref,
    T::Target: Deref,
{
    outer: PierceOuter<T>,
    target: NonNull<<T::Target as Deref>::Target>,
}

pub enum PierceOuter<T>
where
    T: Deref,
    T::Target: Deref,
{
    Normal(T),
    Fallback(Box<T>),
}

fn needs_pinning<T>(outer: &T, target: &<T::Target as Deref>::Target) -> bool
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

impl<T> Pierce<T>
where
    T: Deref,
    T::Target: Deref,
{
    /** Create a new Pierce

    Create a Pierce out of the given nested pointer.
    This method derefs T twice and cache the address where the inner pointer points to.

    Deref-ing the create Pierce returns the cached reference directly. `deref` is not called on T.
     */
    #[inline]
    pub fn new(outer: T) -> Self {
        let inner: &T::Target = outer.deref();
        let target: &<T::Target as Deref>::Target = inner.deref();

        if needs_pinning(&outer, target) {
            let target = NonNull::from(target);
            Self {
                outer: PierceOuter::Normal(outer),
                target,
            }
        } else {
            let boxed = Box::new(outer);
            let target = NonNull::from(&***boxed);
            Self {
                outer: PierceOuter::Fallback(boxed),
                target,
            }
        }
    }

    /** Borrow the outer pointer T

    You can then call the methods on &T.

    You can even call `deref` twice on &T directly to bypass Pierce's cache:
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
        match &self.outer {
            PierceOuter::Normal(ptr) => ptr,
            PierceOuter::Fallback(boxed) => &boxed,
        }
    }

    /** Get the outer pointer T out.

    Like `into_inner()` elsewhere, this consumes the Pierce and return the wrapped pointer.
     */
    #[inline]
    pub fn into_outer(self) -> T {
        match self.outer {
            PierceOuter::Normal(ptr) => ptr,
            PierceOuter::Fallback(boxed) => *boxed,
        }
    }

    /** Whether or not Pierce has cached the target

    Pierce only cache the target when it is safe to do so. See the "Limitations" section at the [crate docs][crate].

    This method returns true if the target is cached, false if Pierce is falling back to double-derefing every time.
    */
    pub fn is_cached(&self) -> bool {
        match self.outer {
            PierceOuter::Normal(..) => true,
            PierceOuter::Fallback(..) => false,
        }
    }
}

unsafe impl<T> Send for Pierce<T>
where
    T: Deref + Send,
    T::Target: Deref,
    <T::Target as Deref>::Target: Sync,
{
}

unsafe impl<T> Sync for Pierce<T>
where
    T: Deref + Sync,
    T::Target: Deref,
    <T::Target as Deref>::Target: Sync,
{
}

impl<T> Clone for Pierce<T>
where
    T: Deref + Clone,
    T::Target: Deref,
{
    #[inline]
    fn clone(&self) -> Self {
        match &self.outer {
            PierceOuter::Normal(ptr) => Self::new(ptr.clone()),
            PierceOuter::Fallback(boxed) => Self::new((&**boxed).clone()),
        }
    }
}

impl<T> Deref for Pierce<T>
where
    T: Deref,
    T::Target: Deref,
{
    type Target = <T::Target as Deref>::Target;
    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { self.target.as_ref() }
        /* SAFETY:
        The Pierce must still be alive (not dropped) when this is called,
        and thus the outer pointer must still be alive.

        The Pierce might be moved, but moving the Pierce only moves the outer pointer.
        And if the target points to somewhere in the outer pointer,
        we would have pinned the outer pointer by boxing it anyway.

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
    T: Deref,
    T::Target: Deref,
{
    #[inline]
    fn as_ref(&self) -> &<T::Target as Deref>::Target {
        &**self
    }
}

impl<T> Default for Pierce<T>
where
    T: Deref + Default,
    T::Target: Deref,
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
        assert_eq!(p1.is_cached(), true);
    }

    #[test]
    fn test_rc_string() {
        use std::rc::Rc;

        let v = String::from("hello world");
        let a = Rc::new(v);
        let pierce = Pierce::new(a);
        assert_eq!(&*pierce, "hello world");
        assert_eq!(pierce.is_cached(), true);
    }

    #[test]
    fn test_box_vec() {
        let v = vec![1, 2, 3];
        let a = Box::new(v);
        let pierce = Pierce::new(a);
        assert_eq!(*pierce.get(2).unwrap(), 3);
        assert_eq!(pierce.is_cached(), true);
    }

    #[test]
    fn test_triply_nested() {
        let b: Box<Box<Box<i32>>> = Box::new(Box::new(Box::new(42)));
        let pierce_once = Pierce::new(b);
        assert_eq!(*Box::deref(Pierce::deref(&pierce_once)), 42);
        let pierce_twice = Pierce::new(pierce_once);
        assert_eq!(*Pierce::deref(&pierce_twice), 42);
        assert_eq!(pierce_twice.is_cached(), true);
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
        let weird_pierce = Pierce::new(Box::new(WeirdPointer {
            inner: RefCell::new(true),
        }));
        assert_eq!(weird_pierce.is_cached(), true);
        assert_eq!(&**weird_normal, "hello");
        assert_eq!(&*weird_pierce, "hello");
        assert_eq!(&**weird_normal, "world");
        assert_eq!(&*weird_pierce, "hello");
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
        let p = Pierce::new(c);

        assert_eq!(p.is_cached(), false);
    }
    #[test]
    fn test_box_stack() {
        let a = 41;
        let b = StackPtr(a);
        let c = Box::new(b);
        let p = Pierce::new(c);

        assert_eq!(p.is_cached(), true);
    }
    #[test]
    fn test_stack_box() {
        let a = 41;
        let b = Box::new(a);
        let c = StackPtr(b);
        let p = Pierce::new(c);

        assert_eq!(p.is_cached(), true);
    }

    #[test]
    fn test_box_box() {
        let a = 41;
        let b = Box::new(a);
        let c = Box::new(b);
        let p = Pierce::new(c);

        assert_eq!(p.is_cached(), true);
    }
}
