use pierce::Pierce;
use std::time::{Duration, Instant};

const SMALL_NUM: usize = 65536;
const MEDIUM_NUM: usize = 1_000_000;
const BIG_NUM: usize = 16_000_000;
const HUGE_NUM: usize = 640_000_000;

#[inline(never)]
fn bench_fragmented_box_vec() {
    #[inline(never)]
    fn normal() -> Duration {
        // Create the vec we will read.
        let v: Vec<usize> = (0..SMALL_NUM).collect();

        // Confuse the optimizer and kinda simulate memory fragmentation by creating a lot of empty vecs.
        let mut boxes: Vec<Box<Vec<usize>>> = (0..BIG_NUM).map(|_| Box::new(vec![])).collect();
        *boxes[BIG_NUM / 2] = v;
        let b = std::mem::replace(&mut boxes[BIG_NUM / 2], Default::default());

        let mut _sum = 0;

        // Measure read time
        let start = Instant::now();
        for i in 0..HUGE_NUM {
            _sum += b.get(i % SMALL_NUM).unwrap();
        }

        start.elapsed()
    }

    #[inline(never)]
    fn pierce() -> Duration {
        let v: Vec<usize> = (0..SMALL_NUM).collect();

        let mut boxes: Vec<Box<Vec<usize>>> = (0..BIG_NUM).map(|_| Box::new(vec![])).collect();
        *boxes[BIG_NUM / 2] = v;
        let b = std::mem::replace(&mut boxes[BIG_NUM / 2], Default::default());

        let mut _sum = 0;
        let start = Instant::now();
        let p = Pierce::new(b);
        for i in 0..HUGE_NUM {
            _sum += p.get(i % SMALL_NUM).unwrap();
        }

        start.elapsed()
    }

    println!("Fragmented Box<Vec<_>> benchmark");

    let mut normal_took = Duration::from_secs(0);
    let mut pierce_took = Duration::from_secs(0);

    // Warm up a bit.
    normal();
    pierce();

    // Actual runs.
    normal_took += normal();
    pierce_took += pierce();
    normal_took += normal();
    pierce_took += pierce();

    println!("Normal: {:.2?}, Pierce: {:.2?}", normal_took, pierce_took);
}

#[inline(never)]
fn bench_slow_box() {
    // SlowBox: like Box but computes Collatz(63) every time you want to deref it.
    struct SlowBox<T>(Box<T>);
    impl<T> std::ops::Deref for SlowBox<T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            let mut n = 31;
            loop {
                match n {
                    1 => break self.0.deref(),
                    _ if n % 2 == 0 => n /= 2,
                    _ => n = n * 3 + 1,
                }
            }
        }
    }
    impl<T> SlowBox<T> {
        fn new(inner: T) -> Self {
            Self(Box::new(inner))
        }
    }

    #[inline(never)]
    fn normal() -> Duration {
        let a: SlowBox<Vec<usize>> = SlowBox::new((0..SMALL_NUM).collect());
        let start = Instant::now();
        for i in 0..MEDIUM_NUM {
            a.get(i % SMALL_NUM).unwrap();
        }
        start.elapsed()
    }

    #[inline(never)]
    fn pierce() -> Duration {
        let a: SlowBox<Vec<usize>> = SlowBox::new((0..SMALL_NUM).collect());
        let start = Instant::now();
        let p = Pierce::new(a);
        for i in 0..MEDIUM_NUM {
            p.get(i % SMALL_NUM).unwrap();
        }
        start.elapsed()
    }

    println!("SlowBox<_> benchmark");

    let mut normal_took = Duration::from_secs(0);
    let mut pierce_took = Duration::from_secs(0);

    // Warm up a bit.
    normal();
    pierce();

    // Actual runs.
    normal_took += normal();
    pierce_took += pierce();
    normal_took += normal();
    pierce_took += pierce();

    println!("Normal: {:.2?}, Pierce: {:.2?}", normal_took, pierce_took);
}

#[inline(never)]
fn bench_vec_box_box() {
    #[inline(never)]
    fn normal() -> Duration {
        let start = Instant::now();
        let v: Vec<Box<Box<i64>>> = (0..MEDIUM_NUM)
            .map(|i| Box::new(Box::new(i as i64)))
            .collect();
        let mut sum = 0i64;
        for _ in 0..MEDIUM_NUM {
            let mut i: usize = 65535;
            loop {
                match i {
                    1 => break,
                    v if v % 2 == 1 => i = v * 3 + 1,
                    v => i = v / 2,
                }
                sum += ***v.get(i % MEDIUM_NUM).unwrap();
            }
        }
        assert!(sum > 4000i64);
        start.elapsed()
    }
    #[inline(never)]
    fn pierce() -> Duration {
        let start = Instant::now();
        let v: Vec<Pierce<Box<Box<i64>>>> = (0..MEDIUM_NUM)
            .map(|i| Pierce::new(Box::new(Box::new(i as i64))))
            .collect();
        let mut sum = 0i64;
        for _ in 0..MEDIUM_NUM {
            let mut i: usize = 65535;
            loop {
                match i {
                    1 => break,
                    v if v % 2 == 1 => i = v * 3 + 1,
                    v => i = v / 2,
                }
                sum += **v.get(i % MEDIUM_NUM).unwrap();
            }
        }
        assert!(sum > 4000i64);
        start.elapsed()
    }

    let mut normal_took = Duration::from_secs(0);
    let mut pierce_took = Duration::from_secs(0);

    println!("Vec<Box<Box<_>>> benchmark");

    // Warm up a bit.
    normal();
    pierce();

    // Actual runs.
    normal_took += normal();
    pierce_took += pierce();
    normal_took += normal();
    pierce_took += pierce();

    println!("Normal: {:.2?}, Pierce: {:.2?}", normal_took, pierce_took);
}

#[inline(never)]
fn bench_fragmented_arc_string() {
    #[inline(never)]
    fn normal() -> Duration {
        let mut strings: Vec<Box<String>> = (0..BIG_NUM)
            .map(|idx| Box::new((idx * idx).to_string()))
            .collect();
        let (l, r) = strings.split_at_mut(BIG_NUM / 2);
        for i in 0..(BIG_NUM / 2) {
            let l = &mut *l[i];
            let r = &mut *r[i];
            std::mem::swap(l, r);
        }
        let t: u64 = strings[14620135].parse().unwrap();
        let u = t.to_string();
        let start = Instant::now();
        for (idx, s) in strings.iter().enumerate() {
            if (**s).partial_cmp(&u) == Some(std::cmp::Ordering::Equal) {
                assert_eq!(idx, 14620135);
                break;
            }
        }
        start.elapsed()
    }

    #[inline(never)]
    fn pierce() -> Duration {
        let mut strings: Vec<Box<String>> = (0..BIG_NUM)
            .map(|idx| Box::new((idx * idx).to_string()))
            .collect();
        let (l, r) = strings.split_at_mut(BIG_NUM / 2);
        for i in 0..(BIG_NUM / 2) {
            let l = &mut *l[i];
            let r = &mut *r[i];
            std::mem::swap(l, r);
        }
        let strings: Vec<Pierce<Box<String>>> = strings.into_iter().map(Pierce::new).collect();
        let t: u64 = strings[14620135].parse().unwrap();
        let u = t.to_string();
        let start = Instant::now();
        for (idx, s) in strings.iter().enumerate() {
            if (*s).partial_cmp(&u) == Some(std::cmp::Ordering::Equal) {
                assert_eq!(idx, 14620135);
                break;
            }
        }
        start.elapsed()
    }
    let mut normal_took = Duration::from_secs(0);
    let mut pierce_took = Duration::from_secs(0);

    println!("Vec<Arc<String>> benchmark");

    // Warm up a bit.
    normal();
    pierce();

    // Actual runs.
    normal_took += normal();
    pierce_took += pierce();
    normal_took += normal();
    pierce_took += pierce();

    println!("Normal: {:.2?}, Pierce: {:.2?}", normal_took, pierce_took);
}

fn main() {
    bench_fragmented_box_vec();
    bench_slow_box();
    bench_vec_box_box();
    bench_fragmented_arc_string();
}
