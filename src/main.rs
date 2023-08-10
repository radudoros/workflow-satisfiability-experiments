use planning::planning::plan_all;
use workflow::graph;

mod bipartite_matching;
mod partition_generator;
mod planning;
mod planning_misc;
mod predicates;
mod workflow;

fn main() {
    // Read graph from a file
    let mut g = match graph::from_file("graph.txt") {
        Ok(graph) => graph,
        Err(err) => {
            eprintln!("Error reading the graph: {}", err);
            return;
        }
    };
    g.print();

    // Read authentication sets
    let auth_sets = match planning::read_auth_sets("auth_sets.txt") {
        Ok(auth) => auth,
        Err(err) => {
            eprintln!("Error reading authentication sets: {}", err);
            return;
        }
    };

    // Read Separation of Duty and Binding of Duty constraints into predicates
    let mut ui_preds = predicates::binary_predicates::default();

    if let Err(err) = ui_preds.read_sod_from_file("sod.txt") {
        eprintln!("Error reading SoD constraints: {}", err);
        return;
    }

    if let Err(err) = ui_preds.read_bod_from_file("bod.txt") {
        eprintln!("Error reading BoD constraints: {}", err);
        return;
    }

    let mut ud_preds = predicates::binary_predicates::default();
    let ud_scope = vec![3, 4, 5];
    ud_preds.generate_k_different(ud_scope.clone(), 2);

    let res = match plan_all(
        &mut g,
        &auth_sets,
        &ud_preds,
        &ud_scope,
        &ui_preds,
        &vec![1, 2, 3],
        4,
    ) {
        Some(ans) => ans,
        None => {
            eprint!("No solutions here!");
            return;
        }
    };

    for v in res.iter() {
        print!(" {}", v);
    }
}
