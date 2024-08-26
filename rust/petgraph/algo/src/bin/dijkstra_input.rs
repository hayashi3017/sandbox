use std::collections::HashMap;

use petgraph::{algo::dijkstra, graph::NodeIndex, Graph, Undirected};

fn main() {
    // input
    let nodes_num = 6;
    let edges_num = 7;
    let weights = [
        (1, 2, 15),
        (1, 4, 20),
        (2, 3, 65),
        (2, 5, 4),
        (3, 6, 50),
        (4, 5, 30),
        (5, 6, 8),
    ];

    let mut graph: Graph<(), usize, Undirected> = Graph::new_undirected();

    let mut nodes_indices = vec![];
    for _ in 0..nodes_num {
        nodes_indices.push(graph.add_node(()));
    }
    // dbg!(&nodes_indices);

    for (node1, node2, weight) in weights {
        graph.add_edge(nodes_indices[node1 - 1], nodes_indices[node2 - 1], weight);
    }

    // dbg!(&graph);

    let expected: HashMap<NodeIndex, usize> = [
        (nodes_indices[0], 0),
        (nodes_indices[1], 15),
        (nodes_indices[2], 77),
        (nodes_indices[3], 20),
        (nodes_indices[4], 19),
        (nodes_indices[5], 27),
    ]
    .iter()
    .cloned()
    .collect();
    let res = dijkstra(&graph, nodes_indices[0], None, |e| *e.weight());
    // dbg!(res);
    assert_eq!(res, expected);
}
