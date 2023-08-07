use predicates::{binary_predicates, generate_mock_ud_predicates, generate_mock_ui_predicates, weight_predicates, generate_mock_weight_predicates};
use workflow::{graph, topo_sort};

mod bipartite_matching;
mod partition_generator;
mod planning;
mod predicates;
mod workflow;
mod planning_misc;

fn main() {
    
    let mut g = graph::new(4);
    g.add_edge(0, 1);
    g.add_edge(0, 2);
    g.add_edge(1, 3);
    g.add_edge(2, 3);

    g.print();

    let mut generator = partition_generator::PartitionsGenerator::new(4);

    while let Some(partition) = generator.next() {
        println!("{:?}", partition);
    }
}
