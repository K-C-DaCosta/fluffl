use super::*;

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
        tree.remove(&mut CircularIterState::new(), int)
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
        .remove(&mut CircularIterState::new(), interval)
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

    let intervals = [
        (0, 64),
        (128 * 7, 128 * 8 - 1),
        (128 * 8, 128 * 10),
        (900, 1050),
    ]
    .iter()
    .map(|&a| Interval::from(a))
    .collect::<Vec<_>>();

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
        tree.remove(&mut CircularIterState::new(), int)
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
            tree.remove(&mut CircularIterState::new(), int)
                .for_each(|a| ());
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
fn insert_search_by_scalar_shotgun_0() {
    let (intervals, Interval { lo, hi }) = generate_sorted_test_intervals();

    //create tree
    let mut tree = CircularSegmentTree::<()>::new(30, 1 << 30);

    //insert intervals into tree
    for &range in &intervals {
        tree.insert(range, ());
    }

    let mut time = lo;
    let step_size = ((hi - lo) / 2000).max(1);
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

#[test]
fn insert_search_by_interval_shotgun_0() {
    let (intervals,_range) = generate_sorted_test_intervals();//to_intervals(vec![(0, 17), (15, 18), (55, 102)]);

    let mut tree = CircularSegmentTree::<u32>::new(29, 1 << 31);
    for (k, &interval) in intervals.iter().enumerate() {
        tree.insert(interval, k as u32);
    }
    // println!("{:?}", &intervals[0..16]);

    tree.search_interval(&mut CircularIterState::new(), Interval::from((0, 209)))
        .for_each(|(a, b)| {
            println!("{}", b.data);
        });
}

fn generate_sorted_test_intervals() -> (Vec<Interval>, Interval) {
    let mut state = 0xaaabbu128;
    //generate lots of intervals
    let mut intervals = (0..60_000)
        .map(|_| {
            let l = rand_lehmer64(&mut state) as u128 % 3_600_000;
            let u = 1 + l + rand_lehmer64(&mut state) as u128 % 60_000;
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
fn to_intervals(tuples: Vec<(u128, u128)>) -> Vec<Interval> {
    tuples
        .into_iter()
        .map(|a| Interval::from(a))
        .collect::<Vec<_>>()
}
