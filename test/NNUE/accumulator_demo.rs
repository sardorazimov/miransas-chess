// Standalone NNUE accumulator demo.
// This file is intentionally tiny and not connected to the engine.

const FEATURES: usize = 8;
const HIDDEN: usize = 4;

type Accumulator = [i32; HIDDEN];

const BIAS: Accumulator = [10, -3, 2, 7];

const WEIGHTS: [[i32; HIDDEN]; FEATURES] = [
    [1, 0, 2, -1],
    [0, 3, -1, 1],
    [2, 2, 0, 0],
    [-1, 1, 3, 2],
    [4, -2, 1, 0],
    [0, 1, 0, 5],
    [3, 0, -2, 1],
    [1, 1, 1, 1],
];

fn refresh(features: &[usize]) -> Accumulator {
    let mut acc = BIAS;
    for &feature in features {
        add_feature(&mut acc, feature);
    }
    acc
}

fn add_feature(acc: &mut Accumulator, feature: usize) {
    for i in 0..HIDDEN {
        acc[i] += WEIGHTS[feature][i];
    }
}

fn remove_feature(acc: &mut Accumulator, feature: usize) {
    for i in 0..HIDDEN {
        acc[i] -= WEIGHTS[feature][i];
    }
}

fn clipped_relu(x: i32) -> i32 {
    x.clamp(0, 127)
}

fn output(acc: &Accumulator) -> i32 {
    let output_weights = [2, -1, 3, 1];
    acc.iter()
        .zip(output_weights)
        .map(|(&value, weight)| clipped_relu(value) * weight)
        .sum()
}

fn main() {
    let before_features = [0, 2, 5];
    let after_features = [0, 4, 5];

    let mut incremental = refresh(&before_features);
    remove_feature(&mut incremental, 2);
    add_feature(&mut incremental, 4);

    let full_refresh = refresh(&after_features);

    println!("incremental accumulator: {incremental:?}");
    println!("full refresh accumulator: {full_refresh:?}");
    println!("score: {}", output(&full_refresh));

    assert_eq!(incremental, full_refresh);
}
