// use mycrate::fibonacci;
use planner::planning::planning::plan_all;
use planner::predicates::binary_predicates;
use planner::workflow::graph;


use std::io::Cursor;
use std::fs::File;
use std::io::{self, BufReader, BufRead};
use std::path::Path;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_combined_approach(c: &mut Criterion) {
    // let step_size = 20;
    // let content = format!("#Steps: {}\n\
    // #Users: 3\n\
    // #Constraints: 6\n\
    // Authorizations:\n\
    // user 1: 1 0 1 1 0 1 0 1 1 0 1 1 0 1 0 0 1 0 1 1\n\
    // user 2: 0 1 0 1 1 0 1 0 1 1 1 0 1 1 0 1 0 1 1 0\n\
    // user 3: 1 1 0 1 0 0 1 0 1 1 0 1 0 1 1 0 1 0 1 1\n\
    // Constraints:\n\
    // sod scope 1 2\n\
    // sod scope 0 1\n\
    // bod scope 2 3\n\
    // sod scope 6 7\n\
    // bod scope 8 9\n", step_size);

    let step_size = 20;
    let content = format!("#Steps: {}\n\
    #Users: 7\n\
    #Constraints: 12\n\
    Authorizations:\n\
    user 1: 1 0 1 1 0 1 0 1 1 0 1 1 1 1 1 0 1 1 0 1\n\
    user 2: 0 1 0 1 1 0 1 0 1 1 1 0 1 0 1 1 0 0 1 0\n\
    user 3: 1 1 0 1 0 0 1 0 1 1 0 1 1 1 0 1 0 0 1 1\n\
    user 4: 0 1 1 1 1 1 1 0 0 0 1 0 1 0 1 0 1 0 0 1\n\
    user 5: 0 0 0 0 1 1 1 1 1 0 1 1 1 0 0 0 0 1 1 0\n\
    user 6: 0 1 1 1 0 0 1 0 1 0 1 0 0 0 1 0 1 1 1 0\n\
    user 7: 0 1 0 1 1 1 1 1 1 0 0 1 0 1 0 1 1 0 1 1\n\
    Constraints:\n\
    sod scope 2 3\n\
    sod scope 1 2\n\
    sod scope 1 9\n\
    sod scope 1 10\n\
    bod scope 1 11\n\
    bod scope 3 4\n\
    sod scope 7 8\n\
    sod scope 11 12\n\
    sod scope 12 13\n\
    sod scope 14 15\n\
    bod scope 13 15\n\
    bod scope 9 10\n", step_size);

    let cursor = Cursor::new(content);
    let mut binary_preds = binary_predicates::default();
    let (auth_sets, node_priorities) = binary_preds.read_constraints(cursor).unwrap();

    let mut node_indices: Vec<usize> = (0..node_priorities.len()).collect();
    node_indices.sort_by_key(|&index| std::cmp::Reverse(node_priorities[index]));

    let g = graph::new(step_size);

    c.bench_function("Combined Approach", |b| {
        b.iter(|| {
            let mut g = g.clone(); // Clone the original graph for each iteration
            let mut g = black_box(&mut g);
            let ui_preds = black_box(&binary_preds);
            let auth_sets = black_box(&auth_sets);

            let ud_preds = binary_predicates::default();
            let ud_scope = vec![1];

            let _res = match plan_all(
                &mut g,
                &node_indices,
                auth_sets,
                &ud_preds,
                &ud_scope,
                &ui_preds,
                &vec![1, 2, 3, 4, 5],
                7,
            ) {
                Some(ans) => ans,
                None => {
                    eprint!("No solutions here!");
                    return;
                }
            };
        })
    });

    // The naive benchmark is now heavily outperformed by the

    // c.bench_function("Backtracking", |b| {
    //     b.iter(|| {
    //         let mut g = g.clone(); // Clone the original graph for each iteration
    //         let mut g = black_box(&mut g);
    //         let ud_preds = black_box(&binary_preds);
    //         let auth_sets = black_box(&auth_sets);

    //         let ui_preds = binary_predicates::default();
    //         let ud_scope: Vec<usize> = (0..step_size).collect();
    //         // combined_approach(g, binary_preds, auth_sets);

    //         let _res = match plan_all(
    //             &mut g,
    //             auth_sets,
    //             &ud_preds,
    //             &ud_scope,
    //             &ui_preds,
    //             &vec![1, 2, 3, 4, 5],
    //             7,
    //         ) {
    //             Some(ans) => ans,
    //             None => {
    //                 eprint!("No solutions here!");
    //                 return;
    //             }
    //         };
    //     })
    // });
}

fn benchmark_from_file(c: &mut Criterion) {
    // Setup code here
    let filename = "instance0.txt";
    let path = Path::new(&filename);
    let file = File::open(&path).expect("Couldn't open file");
    let reader = BufReader::new(file);

    let mut ui_preds = binary_predicates::default();
    let (auth_sets, node_priorities) = ui_preds.read_constraints(reader).unwrap();
    
    let mut node_indices: Vec<usize> = (0..node_priorities.len()).collect();
    node_indices.sort_by_key(|&index| std::cmp::Reverse(node_priorities[index]));

    let step_size = auth_sets.len();
    let g = graph::new(step_size); // moved to setup

    c.bench_function("Benchmark From File", |b| {
        b.iter(|| {
            // Only the code to benchmark
            let mut g = g.clone(); // Clone the original graph for each iteration
            let g = black_box(&mut g); // Avoid compiler optimizations

            let auth_sets = black_box(&auth_sets);
            let ui_preds = black_box(&ui_preds);
            let node_indices = black_box(&node_indices);

            let ud_preds = binary_predicates::default();
            let ud_scope = vec![1];

            let _res = match plan_all(
                g,
                node_indices,
                auth_sets,
                &ud_preds,
                &ud_scope,
                ui_preds,
                &vec![1, 2, 3, 4, 5],
                90,
            ) {
                Some(ans) => ans,
                None => {
                    eprint!("No solutions here!");
                    return;
                }
            };
        })
    });
}


criterion_group!(benches, benchmark_combined_approach, benchmark_from_file);
criterion_main!(benches);