use pierced::Pierced;
use std::time::{Duration, Instant};

fn bench_arc_vec() {
    use std::sync::Arc;

    const SIZE: usize = 100_000_000;

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

    let normal_start = Instant::now();
    normal();
    let normal_took = normal_start.elapsed();

    let pierced_start = Instant::now();
    pierced();
    let pierced_took = pierced_start.elapsed();
    println!(
        "Normal: {:.2?}, Pierced: {:.2?}",
        normal_took.as_micros(),
        pierced_took.as_micros()
    );
}

fn bench_box_vec() {
    const SIZE: usize = 100_000_000;

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

    let normal_start = Instant::now();
    normal();
    let normal_took = normal_start.elapsed();

    let pierced_start = Instant::now();
    pierced();
    let pierced_took = pierced_start.elapsed();
    println!(
        "Normal: {:.2?}, Pierced: {:.2?}",
        normal_took.as_micros(),
        pierced_took.as_micros()
    );
}

fn bench_arc_box() {
    use std::sync::Arc;

    const NUM: u32 = 100_000_000;

    println!("Arc<Box<_>> benchmark");


    let normal = Arc::new(Box::new(42));
    let pierced = Pierced::new(Arc::new(Box::new(42)));
    let mut normal_took = Duration::from_secs(0);
    let mut pierced_took = Duration::from_secs(0);
    for i in 0..NUM {
        if i % 2 == 0 {
            let start = Instant::now();
            let _ = **normal;
            normal_took += start.elapsed();
        }
        else {
            let start = Instant::now();
            let _ = *pierced;
            pierced_took += start.elapsed();
        }
    }

    println!(
        "Normal: {:.2?}, Pierced: {:.2?}",
        normal_took.as_micros(),
        pierced_took.as_micros()
    );
}

fn bench_box_box() {
    const NUM: u32 = 100_000_000;

    println!("Box<Box<_>> benchmark");

    let normal = Box::new(Box::new(42));
    let pierced = Pierced::new(Box::new(Box::new(42)));
    let mut normal_took = Duration::from_secs(0);
    let mut pierced_took = Duration::from_secs(0);
    for i in 0..NUM {
        if i % 2 == 0 {
            let start = Instant::now();
            let _ = **normal;
            normal_took += start.elapsed();
        }
        else {
            let start = Instant::now();
            let _ = *pierced;
            pierced_took += start.elapsed();
        }
    }

    println!(
        "Normal: {:.2?}, Pierced: {:.2?}",
        normal_took.as_micros(),
        pierced_took.as_micros()
    );
}

fn main() {
    bench_arc_vec();
    bench_box_vec();
    bench_arc_box();
    bench_box_box();
}
