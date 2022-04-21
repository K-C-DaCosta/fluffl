use macros::*;

#[fluffl(Debug)]
pub async fn main() {
    println!("Hello, world!");
}

#[test]
pub fn fr_knapsack_test() {
    let value: Vec<f32> = vec![60.0, 100.0, 120.0];
    let mut weight: Vec<f32> = vec![20.0, 50.0, 30.0];

    let best_value = frknap(50.0, &value, &mut weight);
    println!("max cost is = {}", best_value);
    println!("value:{:?}", value);
    println!("cost:{:?}", weight);

    let value: Vec<f32> = vec![500.0];
    let mut weight: Vec<f32> = vec![30.0];
    let best_value = frknap(10.0, &value, &mut weight);
    println!("max cost is = {}", best_value);
    println!("value:{:?}", value);
    println!("cost:{:?}", weight);
}
fn frknap(mut capacity: f32, value: &[f32], weight: &mut [f32]) -> f32 {
    let mut best_value = 0.0f32;
    
    for _ in 0..value.len() {
        if capacity.abs() < 0.001 {
            break;
        }
        //find most valueable item
        let mut max_item = None;
        let mut max_ratio = 0.0;
        for k in 0..value.len() {
            if weight[k].abs() < 0.001 {
                continue;
            }
            let ratio = value[k] / weight[k];
            if ratio > max_ratio {
                max_item = Some(k);
                max_ratio = ratio;
            }
        }
        //add valuable item to bag if exists
        if let Some(item_idx) = max_item {
            //take as much as my sack can hold
            let maximal_take_weight = capacity.min(weight[item_idx]);

            // include value of take_weight
            best_value += max_ratio * maximal_take_weight;
            // decrease capacity of bag since i just took something
            capacity -= maximal_take_weight;

            // compute remaining weight of current item
            weight[item_idx] -= maximal_take_weight;
        }
    }

    best_value
}
