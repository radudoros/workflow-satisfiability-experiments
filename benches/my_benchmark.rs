// use mycrate::fibonacci;
use planner::planning::planning::plan_all;
use planner::predicates::binary_predicates;
use planner::workflow::graph;


use std::io::Cursor;


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

    let step_size = 14;
    let content = format!("#Steps: {}\n\
    #Users: 4\n\
    #Constraints: 8\n\
    Authorizations:\n\
    user 1: 1 0 1 1 0 1 0 1 1 0 1 1 1 1\n\
    user 2: 0 1 0 1 1 0 1 0 1 1 1 0 1 0\n\
    user 3: 1 1 0 1 0 0 1 0 1 1 0 1 1 1\n\
    user 4: 0 1 1 1 1 1 1 0 0 0 1 0 1 0\n\
    Constraints:\n\
    sod scope 1 2\n\
    sod scope 0 1\n\
    bod scope 2 3\n\
    sod scope 6 7\n\
    sod scope 10 11\n\
    sod scope 11 12\n\
    bod scope 8 9\n", step_size);

    let cursor = Cursor::new(content);
    let mut binary_preds = binary_predicates::default();
    let mut auth_sets = binary_preds.read_constraints(cursor).unwrap();
    auth_sets = (0..step_size)
    .map(|step| {
        auth_sets
            .iter()
            .enumerate()
            .filter(|(_, users)| users[step] == 1)
            .map(|(index, _)| index)
            .collect()
    })
    .collect();

    let g = graph::new(step_size);

    c.bench_function("Combined Approach", |b| {
        b.iter(|| {
            let mut g = g.clone(); // Clone the original graph for each iteration
            let mut g = black_box(&mut g);
            let ui_preds = black_box(&binary_preds);
            let auth_sets = black_box(&auth_sets);

            let ud_preds = binary_predicates::default();
            let ud_scope = vec![1];
            // combined_approach(g, binary_preds, auth_sets);

            let _res = match plan_all(
                &mut g,
                auth_sets,
                &ud_preds,
                &ud_scope,
                &ui_preds,
                &vec![1, 2, 3, 4],
                4,
            ) {
                Some(ans) => ans,
                None => {
                    eprint!("No solutions here!");
                    return;
                }
            };
        })
    });

    c.bench_function("Backtracking", |b| {
        b.iter(|| {
            let mut g = g.clone(); // Clone the original graph for each iteration
            let mut g = black_box(&mut g);
            let ud_preds = black_box(&binary_preds);
            let auth_sets = black_box(&auth_sets);

            let ui_preds = binary_predicates::default();
            let ud_scope: Vec<usize> = (0..step_size).collect();
            // combined_approach(g, binary_preds, auth_sets);

            let _res = match plan_all(
                &mut g,
                auth_sets,
                &ud_preds,
                &ud_scope,
                &ui_preds,
                &vec![1, 2, 3, 4],
                4,
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

criterion_group!(benches, benchmark_combined_approach);
criterion_main!(benches);