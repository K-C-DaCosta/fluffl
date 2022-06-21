use super::*;
use std::time::{Duration, Instant};
#[test]
fn delete_sanity() {
    let mut tree = CircularSegmentTree::<usize>::new(4, 1024);

    let intervals = [(0, 64)]
        .iter()
        .map(|&a| Interval::from(a))
        .collect::<Vec<_>>();

    for (i, &int) in intervals.iter().enumerate() {
        tree.insert(int, i);
    }

    let total_nodes_before_remove = tree.linear_tree.nodes().len();
    println!("tree before:");
    tree.print_tree("-+");

    println!("");
    for (_, &int) in intervals.iter().enumerate() {
        tree.remove(&mut TreeIterState::new(), int)
            .for_each(|item| {
                println!("removed item {:?}", item);
            });
        println!("tree now:");
        tree.print_tree("-+");
    }

    println!("\ntree after everything is removed :");
    tree.print_tree("..");

    assert_eq!(
        tree.global_pool.free_slots().len(),
        tree.global_pool.pool().len(),
        "must be the same size if final tree is to be empty"
    );

    assert_eq!(
        tree.global_pool.pool().len(),
        intervals.len(),
        "pooling failed: pool must be same length as number of inserted intervals"
    );

    assert_eq!(
        tree.bucket_pool.pool().len()-1,
        tree.bucket_pool.free_pools().len(),
        "pooling failed: free bucket list and pool list must be same size for the tree to be empty (root is ignored so i do a pool()-1 )"
    );

    assert_eq!(
        tree.bucket_pool.pool().len(),
        total_nodes_before_remove,
        "pooling failed: bucket pool length must be same size as total number of nodes"
    );

    assert_eq!(
        tree.linear_tree.nodes().len(),
        total_nodes_before_remove,
        "pooling failed: this must contain the original node count before remove() gets called"
    );
}

#[test]
fn delete_overlapping_segments() {
    let mut tree = CircularSegmentTree::<usize>::new(4, 1024);
    let interval = Interval::from((1, 100));
    let alt_interval_that_also_fits_in_same_bucket = Interval::from((101, 110));

    // for num_values in 0..=3 {
    let num_values = 3;

    //create vector of values to insert into tree
    let data_values = (1..=num_values).collect::<Vec<_>>();

    // actually insert into tree
    for &val in &data_values {
        tree.insert(interval, val);
    }
    tree.insert(alt_interval_that_also_fits_in_same_bucket, 6969);

    tree.print_tree("+-");

    // remove items from tree and collect removed items
    let mut removed_items = tree
        .remove(&mut TreeIterState::new(), interval)
        .map(|removed_interval| removed_interval.data)
        .collect::<Vec<_>>();
    removed_items.sort();

    console_log!("after removal:\n");
    tree.print_tree("+-");

    //if all operations work correctly both 'data_values' and 'removed_items' should be exactly the same
    assert_eq!(
        data_values, removed_items,
        "inserted items should exactly equal removed items"
    );
    // }
}

#[test]
fn delete_shotgun_0() {
    let mut tree = CircularSegmentTree::<usize>::new(4, 1024);

    let intervals = to_intervals(vec![
        (0, 64),
        (128 * 7, 128 * 8 - 1),
        (128 * 8, 128 * 10),
        (900, 1050),
    ]);
    println!("intervals : {:?}", intervals);

    // let mut total_clipped_intervals = 0;
    // for &x in &intervals {
    //     total_clipped_intervals = tree.clip_interval(x, &mut [Interval::default(); 2]);
    // }

    for (i, &int) in intervals.iter().enumerate() {
        tree.insert(int, i);
    }

    let total_nodes_before_remove = tree.linear_tree.nodes().len();

    println!("tree before:");
    tree.print_tree("-+");

    println!("");
    for (_, &int) in intervals.iter().enumerate() {
        tree.remove(&mut TreeIterState::new(), int)
            .for_each(|item| {
                println!("removed item {:?}", item);
            });
        println!("tree now:");
        tree.print_tree("-+");
    }

    println!("\ntree after everything is removed :");
    tree.print_tree("..");

    //check if pooling works by inserting the list and removing it over and over
    for _ in 0..50_000 {
        //insert same intervals
        for (i, &int) in intervals.iter().enumerate() {
            tree.insert(int, i);
        }
        //delete intervals
        for (_, &int) in intervals.iter().enumerate() {
            tree.remove(&mut TreeIterState::new(), int)
                .for_each(|_a| ());
        }
    }

    assert_eq!(
        tree.global_pool.free_slots().len(),
        tree.global_pool.pool().len(),
        "must be the same size if final tree is to be empty"
    );

    assert_eq!(
        tree.global_pool.pool().len(),
        intervals.len(),
        "pooling failed: pool must be same length as number of inserted intervals"
    );

    assert_eq!(
        tree.bucket_pool.pool().len()-1,
        tree.bucket_pool.free_pools().len(),
        "pooling failed: free bucket list and pool list must be same size for the tree to be empty (root is ignored so i do a pool()-1 )"
    );

    assert_eq!(
        tree.bucket_pool.pool().len(),
        total_nodes_before_remove,
        "pooling failed: bucket pool length must be same size as total number of nodes"
    );

    assert_eq!(
        tree.linear_tree.nodes().len(),
        total_nodes_before_remove,
        "pooling failed: this must contain the original node count before remove() gets called"
    );
}

#[test]
fn insert_search_by_interval_sanity() {
    let intervals = to_intervals(vec![(0, 17), (15, 18), (55, 64), (65, 102)]);

    let mut tree = CircularSegmentTree::<u32>::new(29, 1 << 31);
    for (k, &interval) in intervals.iter().enumerate() {
        tree.insert(interval, k as u32);
    }

    let search_interval_collect_sorted =
        |tree: &CircularSegmentTree<_>, interval: (_, _)| -> Vec<_> {
            let mut search_results = tree
                .search_interval(&mut TreeIterState::new(), Interval::from(interval))
                .map(|(_gi, val)| val.interval)
                .collect::<Vec<_>>();
            search_results.sort_by(sort_scheme);
            search_results
        };

    let search_results = search_interval_collect_sorted(&tree, (0, 55));
    assert_eq!(
        to_intervals(vec![(0, 17), (15, 18), (55, 64)]),
        search_results
    );

    let search_results = search_interval_collect_sorted(&tree, (65, 1200));
    assert_eq!(to_intervals(vec![(65, 102)]), search_results);
}

#[test]
fn insert_search_by_interval_shotgun_0() {
    let mut tree = CircularSegmentTree::<u32>::new(40, 1 << 40);

    let (intervals, range) = generate_sorted_test_intervals();

    intervals.iter().enumerate().for_each(|(k, &interval)| {
        tree.insert(interval, k as u32);
    });

    let search_interval_collect_sorted =
        |tree: &CircularSegmentTree<_>, interval: Interval| -> Vec<_> {
            let mut search_results = tree
                .search_interval(&mut TreeIterState::new(), interval)
                .map(|(_gi, val)| val.interval)
                .collect::<Vec<_>>();
            search_results.sort_by(sort_scheme);
            search_results
        };

    const MAX_INTERVALS: u128 = 4096;
    let mut total_tree_search_dt = 0;
    let mut total_linear_search_dt = 0;
    let mut t0 = Instant::now();

    (0..MAX_INTERVALS)
        .map(|k| range.chunk(MAX_INTERVALS as u64, k as usize))
        .for_each(|test_interval| {
            //preform and time tree search
            t0 = Instant::now();
            let tree_query = search_interval_collect_sorted(&tree, test_interval);
            total_tree_search_dt += t0.elapsed().as_micros();

            //preform and time linear search
            t0 = Instant::now();
            let mut linear_search_query = intervals
                .iter()
                .map(|&a| a)
                .filter(|interval| interval.is_overlapping(&test_interval))
                .collect::<Vec<_>>();
            linear_search_query.sort_by(sort_scheme);
            total_linear_search_dt += t0.elapsed().as_micros();

            //compare segment tree search linear search (should always be equal)
            assert_eq!(
                tree_query,
                linear_search_query,
                "tree query len = {}  , linear search len = {}",
                tree_query.len(),
                linear_search_query.len()
            );
        });

    println!(
        "linear search avg ={} micros\nTree search avg ={} micros",
        total_linear_search_dt / MAX_INTERVALS,
        total_tree_search_dt / MAX_INTERVALS
    );

    let mut total = 0;
    let single_query_range = Interval::from((0, 26));

    t0 = Instant::now();
    tree.search_interval(&mut TreeIterState::new(), single_query_range)
        .for_each(|_| total += 1);
    total_tree_search_dt = t0.elapsed().as_micros();

    // tree.print_tree("+-");

    t0 = Instant::now();
    intervals
        .iter()
        .map(|&a| a)
        .filter(|interval| interval.is_overlapping(&single_query_range))
        .for_each(|_| total += 1);
    total_linear_search_dt = t0.elapsed().as_micros();

    println!(
        "linear search = {} micros  ||  tree search = {} micros",
        total_linear_search_dt, total_tree_search_dt
    );
}

#[test]
fn insert_search_by_scalar_shotgun_0() {
    let (intervals, Interval { lo, hi }) = generate_sorted_test_intervals();

    //create tree
    let mut tree = CircularSegmentTree::<()>::new(30, 1 << 30);

    //insert intervals into tree
    for &range in &intervals {
        tree.insert(range, ());
    }

    let mut time = lo;
    let step_size = ((hi - lo) / FixedPoint::from(2000)).max(From::from(1));
    let mut tree_search_results: Vec<Interval> = Vec::with_capacity(500);
    let mut linear_search_results: Vec<Interval> = Vec::with_capacity(500);

    let mut num_times_tree_beats_linear = 0;
    let mut total_searches = 0;

    let mut tree_avg_dt = 0;
    let mut linear_avg_dt = 0;

    //start at time = lbound and step by fixed size to ubound
    while time <= hi {
        linear_search_results.clear();
        tree_search_results.clear();

        //add search results for the tree
        let t0 = std::time::Instant::now();
        for (_, i) in tree.search_scalar(time) {
            tree_search_results.push(i.interval);
        }
        let tree_dt = t0.elapsed().as_micros();

        //add search results for linear search
        let t0 = std::time::Instant::now();
        for &i in intervals.iter().filter(|i| i.is_within(time)) {
            linear_search_results.push(i);
        }
        let linear_dt = t0.elapsed().as_micros();

        if tree_dt <= linear_dt {
            num_times_tree_beats_linear += 1;
        }
        total_searches += 1;

        tree_avg_dt += tree_dt;
        linear_avg_dt += linear_dt;

        linear_search_results.sort_by(sort_scheme);
        tree_search_results.sort_by(sort_scheme);

        //compare tree results agaisnt linear search
        //both arrays should be exactly the same
        assert_eq!(
            linear_search_results,
            tree_search_results,
            "t = {} (linear_len:{}| tree_len:{})",
            time,
            linear_search_results.len(),
            tree_search_results.len()
        );

        time += step_size;
    }

    println!(
        "tree wins {}  times out of a total of {} searches\n\n",
        num_times_tree_beats_linear, total_searches,
    );

    println!(
        "linear total elapsed :{} ms , tree total elapsed: {} ms ",
        linear_avg_dt / 1000,
        tree_avg_dt / 1000
    );

    println!(
        "linear mean :{} ms , tree mean: {} ms ",
        (linear_avg_dt / 1000) / total_searches,
        (tree_avg_dt / 1000) / total_searches
    );
}

fn generate_sorted_test_intervals() -> (Vec<Interval>, Interval) {
    let mut state = 0xaaabbu128;
    //generate lots of intervals
    let mut intervals = (0..60_000)
        .map(|_| {
            let l = rand_lehmer64(&mut state) % 3_600_000;
            let u = 1 + l + rand_lehmer64(&mut state) % 60_000;
            (l, u)
        })
        .map(|a| Interval::from(a))
        .collect::<Vec<_>>();

    intervals.sort_by(sort_scheme);
    // println!("{:?}",intervals);
    let lbound = intervals.iter().min_by_key(|a| a.lo).unwrap().lo;
    let ubound = intervals.iter().max_by_key(|a| a.hi).unwrap().hi;

    (intervals, Interval::from((lbound, ubound)))
}
fn generate_sorted_test_intervals_with_len(len: usize) -> (Vec<Interval>, Interval) {
    let mut state = 0xaaabbu128;
    //generate lots of intervals
    let mut intervals = (0..len)
        .map(|_| {
            let l = rand_lehmer64(&mut state) % 3_600_000;
            let u = 1 + l + rand_lehmer64(&mut state) % 60_000;
            (l, u)
        })
        .map(|a| Interval::from(a))
        .collect::<Vec<_>>();

    intervals.sort_by(sort_scheme);
    // println!("{:?}",intervals);
    let lbound = intervals.iter().min_by_key(|a| a.lo).unwrap().lo;
    let ubound = intervals.iter().max_by_key(|a| a.hi).unwrap().hi;

    (intervals, Interval::from((lbound, ubound)))
}
fn sort_scheme(a: &Interval, b: &Interval) -> std::cmp::Ordering {
    if a.lo == b.lo {
        a.hi.cmp(&b.hi)
    } else {
        a.lo.cmp(&b.lo)
    }
}
fn to_intervals(tuples: Vec<(u64, u64)>) -> Vec<Interval> {
    tuples
        .into_iter()
        .map(|a| Interval::from(a))
        .collect::<Vec<_>>()
}
