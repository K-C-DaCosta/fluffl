use super::*;

pub fn quick_co_sort<const N: usize, T>(list: &mut [T], mut co_lists: [&mut dyn CanSwap; N])
where
    T: PartialOrd + Copy + Debug,
{
    // let list_ptr = list as *mut [T];
    let co_lists_ptr = &mut co_lists as *mut [&mut dyn CanSwap; N];

    quick_sort(0, list.len(), list, move |(a, b)| {
        let co_lists = unsafe { &mut *co_lists_ptr };
        // let list = unsafe { &*list_ptr };
        for colist in co_lists.iter_mut() {
            colist.ext_swap(a, b);
            // println!("Swappable {{ array: {:?}", list);
            // colist.print();
        }
    });
}

fn quick_sort<T, CB>(lbound: usize, ubound: usize, list: &mut [T], my_swap: CB)
where
    T: PartialOrd + Copy,
    CB: FnMut((usize, usize)) + Copy,
{
    if (ubound - lbound) < 2 {
        return;
    }

    let split_idx = partition(lbound, ubound, list, my_swap);
    quick_sort(lbound, split_idx, list, my_swap);
    quick_sort(split_idx + 1, ubound, list, my_swap);
}

fn partition<T, CB>(lbound: usize, ubound: usize, list: &mut [T], mut my_swap: CB) -> usize
where
    T: PartialOrd + Copy,
    CB: FnMut((usize, usize)),
{
    let mut wall_pos = lbound + 1;
    let wall_val = list[lbound];

    for idx in lbound + 1..ubound {
        let val = list[idx];
        if val < wall_val {
            list.swap(wall_pos, idx);
            my_swap((wall_pos, idx));

            wall_pos += 1;
        }
    }

    list.swap(wall_pos - 1, lbound);
    my_swap((wall_pos - 1, lbound));

    wall_pos - 1
}

#[test]
fn partition_sanity() {
    let mut test = vec![3, 1, 2, 6];
    let split_idx = partition(0, test.len(), &mut test, |_| ());
    assert_eq!(split_idx, 2);
    assert_eq!(vec![2, 1, 3, 6], test);

    let mut test = vec![-10, 2, 5, 11];
    let split_idx = partition(0, test.len(), &mut test, |_| ());
    assert_eq!(split_idx, 0);
    assert_eq!(vec![-10, 2, 5, 11], test);
}

#[test]
fn partition_complex() {
    let mut test = vec![9, -3, 100, 2, 1, -10, 15];
    let pivot = partition(0, test.len(), &mut test, |_| ());
    println!("{:?}", test);
    println!("pivot = {}", pivot);
}

#[test]
fn quick_sort_sanity() {
    // let mut test = vec![9, -3, 100, 2, 1, -10, 15];
    // quick_sort(&mut test, |_| ());

    let mut test = vec![9, 3, 100, 2, 1, 10, 15];
    let mut co_test = vec![-9, -3, -100, -2, -1, -10, -15];
    quick_co_sort(&mut test, [&mut Swappable::new(&mut co_test)]);
    println!("ord_test:{:?}", test);
    println!("co__test:{:?}", co_test);
}
