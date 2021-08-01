# Pierced

[<img alt="crates.io" src="https://img.shields.io/crates/v/pierced?style=for-the-badge" height="20">](https://crates.io/crates/pierced)
[<img alt="crates.io" src="https://img.shields.io/docsrs/pierced?style=for-the-badge" height="20">](https://docs.rs/pierced)

Avoid double indirection in nested smart pointers.

The `Pierced` stuct allows you to cache the deref result of doubly-nested smart pointers.

## Quick Example

```rust
use std::sync::Arc;
use pierced::Pierced;
let vec: Vec<i32> = vec![1, 2, 3];
let arc_vec = Arc::new(vec);
let pierced = Pierced::new(arc_vec);

// Here, the execution jumps directly to the slice. (Without Pierced it would have to jump to the Vec first, than from the Vec to the slice).
pierced.get(0).unwrap();
```

## Nested Smart Pointers

Smart Pointers can be nested to - in a way - combine their functionalities.
For example, with `Arc<Vec<i32>>`, a slice of i32 is managed by the wrapping `Vec` that is wrapped again by the `Arc`.

However, nesting comes at the cost of **double indirection**:
when we want to access the underlying data,
we must first follow the outer pointer to where the inner pointer lies,
then follow the inner pointer to where the underlying data is. Two `deref`-ings. Two jumps.

```rust
use std::sync::Arc;
let vec: Vec<i32> = vec![1, 2, 3];
let arc_vec = Arc::new(vec);

// Here, the `Arc<Vec<i32>>` is first dereferenced to the `Vec<i32>`, then the Vec is dereferenced to the underlying i32 slice.
arc_vec.get(0).unwrap();
```

## Pierced

The `Pierced` struct, provided by this crate,
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

## Usage

`Pierced<T>` can be created with `Pierced::new(...)`. `T` should be a doubly-nested pointer (e.g. `Arc<Vec<_>>`, `Box<Box<_>>`).

`deref`-ing a `Pierced<T>` returns `&<T::Target as Deref>::Target`,
i.e. the deref target of the deref target of T (the outer pointer that is wrapped by Pierced),
i.e. the deref target of the inner pointer.

You can obtain a borrow of just T (the outer pointer) using `.borrow_inner()`.

See [the quick example above](#quick-example)

See the docs at `Pierced` for more details.

### Deeper Nesting

A `Pierced` reduces two jumps to one.
If you have deeper nestings, you can wrap it multiple times.

```rust
use pierced::Pierced;
let triply_nested: Box<Box<Box<i32>>> = Box::new(Box::new(Box::new(42)));
assert_eq!(***triply_nested, 42); // <- Three jumps!
let pierced_twice = Pierced::new(Pierced::new(triply_nested));
assert_eq!(*pierced_twice, 42); // <- Just one jump!
```

## Benchmarks

These benchmarks probably won't represent your use case at all because:
* They are engineered to make Pierced look good.
* Compiler optimizations are hard to control.
* CPU caches and predictions are hard to control. (I bet the figures will be very different on your CPU.)
* Countless other reasons why you shouldn't trust synthetic benchmarks.

*Do your own benchmarks on real-world uses*.

That said, here are my results:

**Benchmark 1**: Read items from a `Box<Vec<usize>>`, with simulated memory fragmentation.

**Benchmark 2**: Read items from a `SlowBox<Vec<usize>>`. `SlowBox` deliberately slow down `deref()` call greatly.

**Benchmark 3**: Read several `Box<Box<i64>>`.

Time taken by `Pierced<T>` version compared to `T` version.

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

See the benchmarks' code [here](https://github.com/wishawa/pierced/tree/main/src/bin/benchmark/main.rs).

## Limitations

### Immutable Only

Pierced only work with immutable data.
**Mutability is not supported at all** because I'm pretty sure it would be impossible to implement soundly.
(If you have an idea please share.)

### Possibly Incorrect

Pierced is **safe, but not neccessarily correct**.
You will not run into memory safety issues (i.e. no "unsafety"),
but you may get the wrong result when deref-ing.

For Pierced to always deref to the correct result,
it must be true for **both** the outer and inner pointer that
**an immutable version of the pointer derefs to the same target every time**.

This condition is met by most common smart pointers, including (but not limited to) `Box`, `Vec`, `String`, `Arc`, `Rc`.
In fact, I have never seen any real-world pointer that doesn't meet this condition. If you know one, please do share.

Here's an example where this invariant is **not** upheld:

```rust
// THIS DOESN'T WORK

use pierced::Pierced;
use std::ops::Deref;
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

For Pierced to function optimally, **the final deref target must not be inside the outer pointer**,
(it should be e.g. somehwere else on the heap or in the static region).

This condition is met by most common smart pointers, including (but not limited to) `Box`, `Vec`, `String`, `Arc`, `Rc`.

For pointers that don't meet this condition,
Pierced pin it to the heap using `Box` to give it a stable address,
so that the cache would not be left dangling if the Pierced (and the outer pointer in it) is moved.

You should avoid using Pierced if your doubly-nested pointer points to itself anyway.
