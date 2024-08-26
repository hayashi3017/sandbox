fn main() {
    use petgraph::algo::dijkstra;
    use petgraph::prelude::*;
    use petgraph::Graph;
    use std::collections::HashMap;

    let mut graph: Graph<(), (), Directed> = Graph::new();
    let a = graph.add_node(()); // node with no weight
    let b = graph.add_node(());
    let c = graph.add_node(());
    let d = graph.add_node(());
    let e = graph.add_node(());
    let f = graph.add_node(());
    let g = graph.add_node(());
    let h = graph.add_node(());
    // z will be in another connected component
    let z = graph.add_node(());

    graph.extend_with_edges(&[
        (a, b),
        (b, c),
        (c, d),
        (d, a),
        (e, f),
        (b, e),
        (f, g),
        (g, h),
        (h, e),
    ]);
    // a ----> b ----> e ----> f
    // ^       |       ^       |
    // |       v       |       v
    // d <---- c       h <---- g

    let expected_res: HashMap<NodeIndex, usize> = [
        (a, 3),
        (b, 0),
        (c, 1),
        (d, 2),
        (e, 1),
        (f, 2),
        (g, 3),
        (h, 4),
    ]
    .iter()
    .cloned()
    .collect();
    let res = dijkstra(&graph, b, None, |_| 1);
    // dbg!(res);
    assert_eq!(res, expected_res);
    // z is not inside res because there is not path from b to z.
}
