use arrow_array::UInt64Array;
use icebug::{ConnectedComponents, DegreeCentrality, Graph, GraphQuery, GraphR, Leiden};

#[test]
fn mutable_graph_and_degree_centrality() {
    let mut graph = Graph::new(3, false, false).unwrap();
    assert!(graph.add_edge(0, 1).unwrap());
    assert!(graph.add_edge(1, 2).unwrap());

    assert_eq!(graph.number_of_nodes(), 3);
    assert_eq!(graph.number_of_edges(), 2);
    assert_eq!(graph.degree(1).unwrap(), 2);

    let mut degree = DegreeCentrality::new(&graph, false, true, true).unwrap();
    degree.run().unwrap();
    assert_eq!(degree.scores().unwrap().len(), 3);
}

#[test]
fn arrow_csr_graph_queries() {
    let graph = GraphR::from_csr(
        3,
        false,
        UInt64Array::from(vec![1_u64, 2, 0, 0]),
        UInt64Array::from(vec![0_u64, 2, 3, 4]),
    )
    .unwrap();

    assert_eq!(graph.number_of_nodes(), 3);
    assert_eq!(graph.neighbors(0).unwrap(), vec![1, 2]);
    assert!(graph.has_edge(0, 1));
}

#[test]
fn connected_components_and_parallel_leiden_view() {
    let mut graph = Graph::new(4, false, false).unwrap();
    graph.add_edge(0, 1).unwrap();
    graph.add_edge(2, 3).unwrap();

    let mut components = ConnectedComponents::new(&graph).unwrap();
    components.run().unwrap();
    assert_eq!(components.number_of_components().unwrap(), 2);

    let mut leiden = Leiden::new(&graph, 1, false, 1.0).unwrap();
    leiden.run().unwrap();
    assert!(leiden.number_of_communities().unwrap() >= 1);
}
