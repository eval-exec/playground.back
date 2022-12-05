use rand::Rng;
use rayon::prelude::*;
use std::ops::AddAssign;
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Duration;

fn main() {
    rayon::ThreadPoolBuilder::new()
        .thread_name(|i| format!("RayonGlobal-{}", i))
        .num_threads(3)
        .build_global()
        .expect("Init the global thread pool for rayon failed");

    let mut rng = rand::thread_rng();
    let txs_len = rng.gen_range(20..50);

    let mut txs = Vec::new();

    let mut expect_sum = 0;

    for _ in 0..txs_len {
        let mut script_groups = Vec::new();
        let script_groups_len = rng.gen_range(10..30);
        for _ in 0..script_groups_len {
            let value = rng.gen_range(1..100);
            expect_sum.add_assign(value);
            script_groups.push(value);
        }
        txs.push(script_groups);
    }

    let ret = AtomicI32::new(0);
    txs.par_iter().for_each(|script_groups| {
        script_groups.par_iter().for_each(|script_group| {
            ret.fetch_add(do_job(script_group), Ordering::SeqCst);
        });
    });

    assert_eq!(ret.load(Ordering::SeqCst), expect_sum);
}

fn do_job(t: &i32) -> i32 {
    std::thread::sleep(Duration::from_millis(*t as u64));
    *t
}
