# Pierce

[<img alt="crates.io" src="https://img.shields.io/crates/v/pierce?style=for-the-badge" height="20">](https://crates.io/crates/pierce)
[<img alt="crates.io" src="https://img.shields.io/docsrs/pierce?style=for-the-badge" height="20">](https://docs.rs/pierce)

Avoid double indirection in nested smart pointers.

The `Pierce` stuct allows you to cache the deref result of doubly-nested smart pointers.

## Quick Example

```rust
use std::sync::Arc;
use pierce::Pierce;
let vec: Vec<i32> = vec![1, 2, 3];
let arc_vec = Arc::new(vec);
let pierce = Pierce::new(arc_vec);

// Here, the execution jumps directly to the slice to call `.get(...)`.
// Without Pierce it would have to jump to the Vec first,
// than from the Vec to the slice.
pierce.get(0).unwrap();
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

// Here, the `Arc<Vec<i32>>` is first dereferenced to the `Vec<i32>`,
// then the Vec is dereferenced to the underlying i32 slice,
// on which `.get(...)` is called.
arc_vec.get(0).unwrap();
```

## Pierce

The `Pierce` struct, provided by this crate,
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

## Usage

`Pierce<T>` can be created with `Pierce::new(...)`. `T` should be a doubly-nested pointer (e.g. `Arc<Vec<_>>`, `Box<Box<_>>`).

`deref`-ing a `Pierce<T>` returns `&<T::Target as Deref>::Target`,
i.e. the deref target of the deref target of T (the outer pointer that is wrapped by Pierce),
i.e. the deref target of the inner pointer.

You can also obtain a borrow of just T (the outer pointer) using `.borrow_inner()`.

See the docs at `Pierce` for more details.

### Deeper Nesting

A `Pierce` reduces two jumps to one.
If you have deeper nestings, you can wrap it multiple times.

```rust
use pierce::Pierce;
let triply_nested: Box<Box<Box<i32>>> = Box::new(Box::new(Box::new(42)));
assert_eq!(***triply_nested, 42); // <- Three jumps!
let pierce_twice = Pierce::new(Pierce::new(triply_nested));
assert_eq!(*pierce_twice, 42); // <- Just one jump!
```

## Benchmarks

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

Pointer wrapped by Pierce must be [`StableDeref`](https://docs.rs/stable_deref_trait/).
If your pointer type meets the conditions required, you can `unsafe impl StableDeref for T {}` on it.
The trait is re-exported at `pierce::StableDeref`.

The vast majority of pointers are `StableDeref`,
including `Box`, `Vec`, `String`, `Rc`, `Arc`.