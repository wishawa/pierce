use pierced::Pierced;
use std::time::{Duration, Instant};

fn bench_arc_vec() {
    use std::sync::Arc;

    const SIZE: usize = 1_000_000;

    fn normal() {
        let v = vec![42; SIZE];
        let a = Arc::new(v);
        for i in 0..SIZE {
            a.get(i).unwrap();
        }
    }

    fn pierced() {
        let v = vec![42; SIZE];
        let a = Arc::new(v);
        let p = Pierced::new(a);
        for i in 0..SIZE {
            p.get(i).unwrap();
        }
    }

    println!("Arc<Vec<_>> benchmark");

    let mut sum_normal = Duration::from_secs(0);
    let mut sum_pierced = Duration::from_secs(0);

    const RUNS: u32 = 8;

    for i in 1..=RUNS {
        let normal_start = Instant::now();
        normal();
        let normal_took = normal_start.elapsed();

        let pierced_start = Instant::now();
        pierced();
        let pierced_took = pierced_start.elapsed();

        println!(
            "Run {:02}: Normal: {:.2?}, Pierced: {:.2?}",
            i, normal_took, pierced_took
        );
        sum_normal += normal_took;
        sum_pierced += pierced_took;
    }
    println!(
        "Average of {} runs: Normal: {:.2?}, Pierced: {:.2?}",
        RUNS,
        sum_normal / RUNS,
        sum_pierced / RUNS
    );
}

fn bench_box_vec() {
    const SIZE: usize = 1_000_000;

    fn normal() {
        let v = vec![42; SIZE];
        let a = Box::new(v);
        for i in 0..SIZE {
            a.get(i).unwrap();
        }
    }

    fn pierced() {
        let v = vec![42; SIZE];
        let a = Box::new(v);
        let p = Pierced::new(a);
        for i in 0..SIZE {
            p.get(i).unwrap();
        }
    }

    println!("Box<Vec<_>> benchmark");

    let mut sum_normal = Duration::from_secs(0);
    let mut sum_pierced = Duration::from_secs(0);

    const RUNS: u32 = 8;

    for i in 1..=RUNS {
        let normal_start = Instant::now();
        normal();
        let normal_took = normal_start.elapsed();

        let pierced_start = Instant::now();
        pierced();
        let pierced_took = pierced_start.elapsed();

        println!(
            "Run {:02}: Normal: {:.2?}, Pierced: {:.2?}",
            i, normal_took, pierced_took
        );
        sum_normal += normal_took;
        sum_pierced += pierced_took;
    }
    println!(
        "Average of {} runs: Normal: {:.2?}, Pierced: {:.2?}",
        RUNS,
        sum_normal / RUNS,
        sum_pierced / RUNS
    );
}

fn bench_arc_box() {
    use std::sync::Arc;

    const NUM: u32 = 10_000_000;

    println!("Arc<Box<_>> benchmark");

    let normal = Arc::new(Box::new(42));
    let pierced = Pierced::new(Arc::new(Box::new(42)));
    let normal_start = Instant::now();
    for _ in 0..NUM {
        let _: &i32 = &*normal;
    }
    let normal_took = normal_start.elapsed();
    let pierced_start = Instant::now();
    for _ in 0..NUM {
        let _: &i32 = &pierced;
    }
    let pierced_took = pierced_start.elapsed();
    println!(
        "Average of {} access: Normal: {:.2?}, Pierced: {:.2?}",
        NUM,
        normal_took / NUM,
        pierced_took / NUM
    );
}

fn main() {
    bench_arc_vec();
    bench_box_vec();
    bench_arc_box();
}
