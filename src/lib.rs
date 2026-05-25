#![doc = include_str!("../README.md")]

use std::marker::PhantomData;
use std::sync::Arc;

use arrow_array::{Array, Float64Array, UInt64Array};

#[cxx::bridge(namespace = "icebug_rust")]
mod ffi {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Edge {
        pub u: u64,
        pub v: u64,
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct WeightedEdge {
        pub u: u64,
        pub v: u64,
        pub weight: f64,
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct RankingEntry {
        pub node: u64,
        pub score: f64,
    }

    #[derive(Debug)]
    pub struct PartitionResult {
        pub membership: Vec<u64>,
        pub count: u64,
    }

    unsafe extern "C++" {
        include!("icebug_rust.hpp");

        type IcebugGraph;
        type Betweenness;
        type PageRank;
        type DegreeCentrality;
        type ConnectedComponents;
        type Louvain;
        type Leiden;

        fn new_graph(n: u64, weighted: bool, directed: bool) -> Result<UniquePtr<IcebugGraph>>;
        fn new_graph_r(
            n: u64,
            directed: bool,
            out_indices: &[u64],
            out_indptr: &[u64],
            in_indices: &[u64],
            in_indptr: &[u64],
            out_weights: &[f64],
            in_weights: &[f64],
        ) -> Result<UniquePtr<IcebugGraph>>;
        fn read_metis(path: &str) -> Result<UniquePtr<IcebugGraph>>;
        fn read_edge_list(
            path: &str,
            separator: &str,
            first_node: u64,
            directed: bool,
            comment_prefix: &str,
        ) -> Result<UniquePtr<IcebugGraph>>;

        fn graph_number_of_nodes(graph: &IcebugGraph) -> u64;
        fn graph_number_of_edges(graph: &IcebugGraph) -> u64;
        fn graph_upper_node_id_bound(graph: &IcebugGraph) -> u64;
        fn graph_is_weighted(graph: &IcebugGraph) -> bool;
        fn graph_is_directed(graph: &IcebugGraph) -> bool;
        fn graph_is_empty(graph: &IcebugGraph) -> bool;
        fn graph_add_node(graph: Pin<&mut IcebugGraph>) -> Result<u64>;
        fn graph_add_edge(
            graph: Pin<&mut IcebugGraph>,
            u: u64,
            v: u64,
            weight: f64,
        ) -> Result<bool>;
        fn graph_remove_node(graph: Pin<&mut IcebugGraph>, u: u64) -> Result<()>;
        fn graph_remove_edge(graph: Pin<&mut IcebugGraph>, u: u64, v: u64) -> Result<()>;
        fn graph_set_weight(
            graph: Pin<&mut IcebugGraph>,
            u: u64,
            v: u64,
            weight: f64,
        ) -> Result<()>;
        fn graph_has_node(graph: &IcebugGraph, u: u64) -> bool;
        fn graph_has_edge(graph: &IcebugGraph, u: u64, v: u64) -> bool;
        fn graph_degree(graph: &IcebugGraph, u: u64) -> Result<u64>;
        fn graph_degree_in(graph: &IcebugGraph, u: u64) -> Result<u64>;
        fn graph_weight(graph: &IcebugGraph, u: u64, v: u64) -> Result<f64>;
        fn graph_nodes(graph: &IcebugGraph) -> Result<Vec<u64>>;
        fn graph_neighbors(graph: &IcebugGraph, u: u64) -> Result<Vec<u64>>;
        fn graph_edges(graph: &IcebugGraph) -> Result<Vec<Edge>>;
        fn graph_weighted_edges(graph: &IcebugGraph) -> Result<Vec<WeightedEdge>>;

        fn new_betweenness(
            graph: &IcebugGraph,
            normalized: bool,
            compute_edge_centrality: bool,
        ) -> Result<UniquePtr<Betweenness>>;
        fn new_page_rank(
            graph: &IcebugGraph,
            damp: f64,
            tol: f64,
            normalized: bool,
        ) -> Result<UniquePtr<PageRank>>;
        fn new_degree_centrality(
            graph: &IcebugGraph,
            normalized: bool,
            out_deg: bool,
            ignore_self_loops: bool,
        ) -> Result<UniquePtr<DegreeCentrality>>;
        fn new_connected_components(graph: &IcebugGraph) -> Result<UniquePtr<ConnectedComponents>>;
        fn new_louvain(
            graph: &IcebugGraph,
            refine: bool,
            gamma: f64,
            max_iter: u64,
            turbo: bool,
            recurse: bool,
        ) -> Result<UniquePtr<Louvain>>;
        fn new_leiden(
            graph: &IcebugGraph,
            iterations: i32,
            randomize: bool,
            gamma: f64,
        ) -> Result<UniquePtr<Leiden>>;

        fn betweenness_run(algo: Pin<&mut Betweenness>) -> Result<()>;
        fn betweenness_has_finished(algo: &Betweenness) -> bool;
        fn betweenness_scores(algo: &Betweenness) -> Result<Vec<f64>>;
        fn betweenness_score(algo: Pin<&mut Betweenness>, node: u64) -> Result<f64>;
        fn betweenness_ranking(algo: Pin<&mut Betweenness>) -> Result<Vec<RankingEntry>>;
        fn betweenness_maximum(algo: Pin<&mut Betweenness>) -> Result<f64>;

        fn page_rank_run(algo: Pin<&mut PageRank>) -> Result<()>;
        fn page_rank_has_finished(algo: &PageRank) -> bool;
        fn page_rank_scores(algo: &PageRank) -> Result<Vec<f64>>;
        fn page_rank_score(algo: Pin<&mut PageRank>, node: u64) -> Result<f64>;
        fn page_rank_ranking(algo: Pin<&mut PageRank>) -> Result<Vec<RankingEntry>>;
        fn page_rank_number_of_iterations(algo: &PageRank) -> Result<u64>;

        fn degree_centrality_run(algo: Pin<&mut DegreeCentrality>) -> Result<()>;
        fn degree_centrality_has_finished(algo: &DegreeCentrality) -> bool;
        fn degree_centrality_scores(algo: &DegreeCentrality) -> Result<Vec<f64>>;
        fn degree_centrality_score(algo: Pin<&mut DegreeCentrality>, node: u64) -> Result<f64>;
        fn degree_centrality_ranking(algo: Pin<&mut DegreeCentrality>)
            -> Result<Vec<RankingEntry>>;

        fn connected_components_run(algo: Pin<&mut ConnectedComponents>) -> Result<()>;
        fn connected_components_has_finished(algo: &ConnectedComponents) -> bool;
        fn connected_components_number_of_components(algo: &ConnectedComponents) -> Result<u64>;
        fn connected_components_component_of_node(
            algo: &ConnectedComponents,
            node: u64,
        ) -> Result<u64>;
        fn connected_components_component_sizes(algo: &ConnectedComponents) -> Result<Vec<u64>>;

        fn louvain_run(algo: Pin<&mut Louvain>) -> Result<()>;
        fn louvain_has_finished(algo: &Louvain) -> bool;
        fn louvain_number_of_communities(algo: Pin<&mut Louvain>) -> Result<u64>;
        fn louvain_community_of_node(algo: Pin<&mut Louvain>, node: u64) -> Result<u64>;
        fn louvain_partition(algo: Pin<&mut Louvain>) -> Result<PartitionResult>;
        fn louvain_modularity(algo: Pin<&mut Louvain>) -> Result<f64>;

        fn leiden_run(algo: Pin<&mut Leiden>) -> Result<()>;
        fn leiden_has_finished(algo: &Leiden) -> bool;
        fn leiden_number_of_communities(algo: Pin<&mut Leiden>) -> Result<u64>;
        fn leiden_community_of_node(algo: Pin<&mut Leiden>, node: u64) -> Result<u64>;
        fn leiden_partition(algo: Pin<&mut Leiden>) -> Result<PartitionResult>;
        fn leiden_modularity(algo: Pin<&mut Leiden>) -> Result<f64>;
        fn leiden_load_move_scoring_extension(algo: Pin<&mut Leiden>, path: &str) -> Result<()>;
        fn leiden_unload_move_scoring_extension(algo: Pin<&mut Leiden>) -> Result<()>;
    }
}

pub use ffi::{Edge, PartitionResult, RankingEntry, WeightedEdge};

#[derive(Debug)]
pub enum Error {
    Cxx(cxx::Exception),
    InvalidInput(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cxx(err) => err.fmt(f),
            Self::InvalidInput(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for Error {}

impl From<cxx::Exception> for Error {
    fn from(value: cxx::Exception) -> Self {
        Self::Cxx(value)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait GraphRef {
    fn raw(&self) -> &ffi::IcebugGraph;
}

pub struct Graph {
    raw: cxx::UniquePtr<ffi::IcebugGraph>,
}

impl Graph {
    pub fn new(n: u64, weighted: bool, directed: bool) -> Result<Self> {
        Ok(Self {
            raw: ffi::new_graph(n, weighted, directed)?,
        })
    }

    pub fn add_node(&mut self) -> Result<u64> {
        ffi::graph_add_node(self.raw.pin_mut()).map_err(Error::from)
    }

    pub fn add_edge(&mut self, u: u64, v: u64) -> Result<bool> {
        self.add_weighted_edge(u, v, 1.0)
    }

    pub fn add_weighted_edge(&mut self, u: u64, v: u64, weight: f64) -> Result<bool> {
        ffi::graph_add_edge(self.raw.pin_mut(), u, v, weight).map_err(Error::from)
    }

    pub fn remove_node(&mut self, u: u64) -> Result<()> {
        ffi::graph_remove_node(self.raw.pin_mut(), u).map_err(Error::from)
    }

    pub fn remove_edge(&mut self, u: u64, v: u64) -> Result<()> {
        ffi::graph_remove_edge(self.raw.pin_mut(), u, v).map_err(Error::from)
    }

    pub fn set_weight(&mut self, u: u64, v: u64, weight: f64) -> Result<()> {
        ffi::graph_set_weight(self.raw.pin_mut(), u, v, weight).map_err(Error::from)
    }
}

impl GraphRef for Graph {
    fn raw(&self) -> &ffi::IcebugGraph {
        self.raw.as_ref().expect("graph handle is not null")
    }
}

pub struct GraphR {
    raw: cxx::UniquePtr<ffi::IcebugGraph>,
    _out_indices: Arc<UInt64Array>,
    _out_indptr: Arc<UInt64Array>,
    _in_indices: Option<Arc<UInt64Array>>,
    _in_indptr: Option<Arc<UInt64Array>>,
    _out_weights: Option<Arc<Float64Array>>,
    _in_weights: Option<Arc<Float64Array>>,
}

impl GraphR {
    pub fn from_csr(
        n: u64,
        directed: bool,
        out_indices: UInt64Array,
        out_indptr: UInt64Array,
    ) -> Result<Self> {
        Self::from_csr_with_inputs(n, directed, out_indices, out_indptr, None, None, None, None)
    }

    pub fn from_directed_csr(
        n: u64,
        out_indices: UInt64Array,
        out_indptr: UInt64Array,
        in_indices: UInt64Array,
        in_indptr: UInt64Array,
    ) -> Result<Self> {
        Self::from_csr_with_inputs(
            n,
            true,
            out_indices,
            out_indptr,
            Some(in_indices),
            Some(in_indptr),
            None,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_csr_with_inputs(
        n: u64,
        directed: bool,
        out_indices: UInt64Array,
        out_indptr: UInt64Array,
        in_indices: Option<UInt64Array>,
        in_indptr: Option<UInt64Array>,
        out_weights: Option<Float64Array>,
        in_weights: Option<Float64Array>,
    ) -> Result<Self> {
        validate_primitive_array("out_indices", &out_indices)?;
        validate_primitive_array("out_indptr", &out_indptr)?;
        validate_indptr("out_indptr", &out_indptr, n, out_indices.len())?;

        let out_indices = Arc::new(out_indices);
        let out_indptr = Arc::new(out_indptr);
        let in_indices = in_indices.map(Arc::new);
        let in_indptr = in_indptr.map(Arc::new);
        let out_weights = out_weights.map(Arc::new);
        let in_weights = in_weights.map(Arc::new);

        if directed && (in_indices.is_none() || in_indptr.is_none()) {
            return Err(invalid_input(
                "directed GraphR requires incoming CSR arrays",
            ));
        }
        if let Some(arr) = &in_indices {
            validate_primitive_array("in_indices", arr)?;
        }
        if let Some(arr) = &in_indptr {
            validate_primitive_array("in_indptr", arr)?;
            validate_indptr(
                "in_indptr",
                arr,
                n,
                in_indices.as_ref().map_or(0, |idx| idx.len()),
            )?;
        }
        if let Some(arr) = &out_weights {
            validate_float_array("out_weights", arr)?;
            if arr.len() != out_indices.len() {
                return Err(invalid_input("out_weights length must match out_indices"));
            }
        }
        if let Some(arr) = &in_weights {
            validate_float_array("in_weights", arr)?;
            if arr.len() != in_indices.as_ref().map_or(0, |idx| idx.len()) {
                return Err(invalid_input("in_weights length must match in_indices"));
            }
        }

        let empty_u64 = [];
        let empty_f64 = [];
        let raw = ffi::new_graph_r(
            n,
            directed,
            &out_indices.values()[..],
            &out_indptr.values()[..],
            in_indices
                .as_ref()
                .map_or(empty_u64.as_slice(), |arr| &arr.values()[..]),
            in_indptr
                .as_ref()
                .map_or(empty_u64.as_slice(), |arr| &arr.values()[..]),
            out_weights
                .as_ref()
                .map_or(empty_f64.as_slice(), |arr| &arr.values()[..]),
            in_weights
                .as_ref()
                .map_or(empty_f64.as_slice(), |arr| &arr.values()[..]),
        )?;

        Ok(Self {
            raw,
            _out_indices: out_indices,
            _out_indptr: out_indptr,
            _in_indices: in_indices,
            _in_indptr: in_indptr,
            _out_weights: out_weights,
            _in_weights: in_weights,
        })
    }
}

impl GraphRef for GraphR {
    fn raw(&self) -> &ffi::IcebugGraph {
        self.raw.as_ref().expect("graph handle is not null")
    }
}

pub trait GraphQuery: GraphRef {
    fn number_of_nodes(&self) -> u64 {
        ffi::graph_number_of_nodes(self.raw())
    }
    fn number_of_edges(&self) -> u64 {
        ffi::graph_number_of_edges(self.raw())
    }
    fn upper_node_id_bound(&self) -> u64 {
        ffi::graph_upper_node_id_bound(self.raw())
    }
    fn is_weighted(&self) -> bool {
        ffi::graph_is_weighted(self.raw())
    }
    fn is_directed(&self) -> bool {
        ffi::graph_is_directed(self.raw())
    }
    fn is_empty(&self) -> bool {
        ffi::graph_is_empty(self.raw())
    }
    fn has_node(&self, u: u64) -> bool {
        ffi::graph_has_node(self.raw(), u)
    }
    fn has_edge(&self, u: u64, v: u64) -> bool {
        ffi::graph_has_edge(self.raw(), u, v)
    }
    fn degree(&self, u: u64) -> Result<u64> {
        ffi::graph_degree(self.raw(), u).map_err(Error::from)
    }
    fn degree_in(&self, u: u64) -> Result<u64> {
        ffi::graph_degree_in(self.raw(), u).map_err(Error::from)
    }
    fn weight(&self, u: u64, v: u64) -> Result<f64> {
        ffi::graph_weight(self.raw(), u, v).map_err(Error::from)
    }
    fn nodes(&self) -> Result<Vec<u64>> {
        ffi::graph_nodes(self.raw()).map_err(Error::from)
    }
    fn neighbors(&self, u: u64) -> Result<Vec<u64>> {
        ffi::graph_neighbors(self.raw(), u).map_err(Error::from)
    }
    fn edges(&self) -> Result<Vec<Edge>> {
        ffi::graph_edges(self.raw()).map_err(Error::from)
    }
    fn weighted_edges(&self) -> Result<Vec<WeightedEdge>> {
        ffi::graph_weighted_edges(self.raw()).map_err(Error::from)
    }
}

impl<T: GraphRef> GraphQuery for T {}

pub fn read_metis(path: impl AsRef<str>) -> Result<Graph> {
    Ok(Graph {
        raw: ffi::read_metis(path.as_ref())?,
    })
}

pub fn read_edge_list(
    path: impl AsRef<str>,
    separator: impl AsRef<str>,
    first_node: u64,
    directed: bool,
    comment_prefix: impl AsRef<str>,
) -> Result<Graph> {
    Ok(Graph {
        raw: ffi::read_edge_list(
            path.as_ref(),
            separator.as_ref(),
            first_node,
            directed,
            comment_prefix.as_ref(),
        )?,
    })
}

macro_rules! centrality_wrapper {
    ($name:ident, $ffi_ty:ident, $run:ident, $finished:ident, $scores:ident, $score:ident, $ranking:ident) => {
        pub struct $name<'g> {
            raw: cxx::UniquePtr<ffi::$ffi_ty>,
            _graph: PhantomData<&'g dyn GraphRef>,
        }

        impl $name<'_> {
            pub fn run(&mut self) -> Result<()> {
                ffi::$run(self.raw.pin_mut()).map_err(Error::from)
            }
            pub fn has_finished(&self) -> bool {
                ffi::$finished(self.raw.as_ref().expect("algorithm handle is not null"))
            }
            pub fn scores(&self) -> Result<Vec<f64>> {
                ffi::$scores(self.raw.as_ref().expect("algorithm handle is not null"))
                    .map_err(Error::from)
            }
            pub fn score(&mut self, node: u64) -> Result<f64> {
                ffi::$score(self.raw.pin_mut(), node).map_err(Error::from)
            }
            pub fn ranking(&mut self) -> Result<Vec<RankingEntry>> {
                ffi::$ranking(self.raw.pin_mut()).map_err(Error::from)
            }
        }
    };
}

centrality_wrapper!(
    Betweenness,
    Betweenness,
    betweenness_run,
    betweenness_has_finished,
    betweenness_scores,
    betweenness_score,
    betweenness_ranking
);
centrality_wrapper!(
    PageRank,
    PageRank,
    page_rank_run,
    page_rank_has_finished,
    page_rank_scores,
    page_rank_score,
    page_rank_ranking
);
centrality_wrapper!(
    DegreeCentrality,
    DegreeCentrality,
    degree_centrality_run,
    degree_centrality_has_finished,
    degree_centrality_scores,
    degree_centrality_score,
    degree_centrality_ranking
);

impl<'g> Betweenness<'g> {
    pub fn new(
        graph: &'g impl GraphRef,
        normalized: bool,
        compute_edge_centrality: bool,
    ) -> Result<Self> {
        Ok(Self {
            raw: ffi::new_betweenness(graph.raw(), normalized, compute_edge_centrality)?,
            _graph: PhantomData,
        })
    }

    pub fn maximum(&mut self) -> Result<f64> {
        ffi::betweenness_maximum(self.raw.pin_mut()).map_err(Error::from)
    }
}

impl<'g> PageRank<'g> {
    pub fn new(graph: &'g impl GraphRef, damp: f64, tol: f64, normalized: bool) -> Result<Self> {
        Ok(Self {
            raw: ffi::new_page_rank(graph.raw(), damp, tol, normalized)?,
            _graph: PhantomData,
        })
    }

    pub fn number_of_iterations(&self) -> Result<u64> {
        ffi::page_rank_number_of_iterations(
            self.raw.as_ref().expect("algorithm handle is not null"),
        )
        .map_err(Error::from)
    }
}

impl<'g> DegreeCentrality<'g> {
    pub fn new(
        graph: &'g impl GraphRef,
        normalized: bool,
        out_deg: bool,
        ignore_self_loops: bool,
    ) -> Result<Self> {
        Ok(Self {
            raw: ffi::new_degree_centrality(graph.raw(), normalized, out_deg, ignore_self_loops)?,
            _graph: PhantomData,
        })
    }
}

pub struct ConnectedComponents<'g> {
    raw: cxx::UniquePtr<ffi::ConnectedComponents>,
    _graph: PhantomData<&'g dyn GraphRef>,
}

impl<'g> ConnectedComponents<'g> {
    pub fn new(graph: &'g impl GraphRef) -> Result<Self> {
        Ok(Self {
            raw: ffi::new_connected_components(graph.raw())?,
            _graph: PhantomData,
        })
    }
    pub fn run(&mut self) -> Result<()> {
        ffi::connected_components_run(self.raw.pin_mut()).map_err(Error::from)
    }
    pub fn has_finished(&self) -> bool {
        ffi::connected_components_has_finished(
            self.raw.as_ref().expect("algorithm handle is not null"),
        )
    }
    pub fn number_of_components(&self) -> Result<u64> {
        ffi::connected_components_number_of_components(
            self.raw.as_ref().expect("algorithm handle is not null"),
        )
        .map_err(Error::from)
    }
    pub fn component_of_node(&self, node: u64) -> Result<u64> {
        ffi::connected_components_component_of_node(
            self.raw.as_ref().expect("algorithm handle is not null"),
            node,
        )
        .map_err(Error::from)
    }
    pub fn component_sizes(&self) -> Result<Vec<u64>> {
        ffi::connected_components_component_sizes(
            self.raw.as_ref().expect("algorithm handle is not null"),
        )
        .map_err(Error::from)
    }
}

macro_rules! community_wrapper {
    ($name:ident, $ffi_ty:ident, $run:ident, $finished:ident, $count:ident, $community:ident, $partition:ident, $modularity:ident) => {
        pub struct $name<'g> {
            raw: cxx::UniquePtr<ffi::$ffi_ty>,
            _graph: PhantomData<&'g dyn GraphRef>,
        }

        impl $name<'_> {
            pub fn run(&mut self) -> Result<()> {
                ffi::$run(self.raw.pin_mut()).map_err(Error::from)
            }
            pub fn has_finished(&self) -> bool {
                ffi::$finished(self.raw.as_ref().expect("algorithm handle is not null"))
            }
            pub fn number_of_communities(&mut self) -> Result<u64> {
                ffi::$count(self.raw.pin_mut()).map_err(Error::from)
            }
            pub fn community_of_node(&mut self, node: u64) -> Result<u64> {
                ffi::$community(self.raw.pin_mut(), node).map_err(Error::from)
            }
            pub fn partition(&mut self) -> Result<PartitionResult> {
                ffi::$partition(self.raw.pin_mut()).map_err(Error::from)
            }
            pub fn modularity(&mut self) -> Result<f64> {
                ffi::$modularity(self.raw.pin_mut()).map_err(Error::from)
            }
        }
    };
}

community_wrapper!(
    Louvain,
    Louvain,
    louvain_run,
    louvain_has_finished,
    louvain_number_of_communities,
    louvain_community_of_node,
    louvain_partition,
    louvain_modularity
);
community_wrapper!(
    Leiden,
    Leiden,
    leiden_run,
    leiden_has_finished,
    leiden_number_of_communities,
    leiden_community_of_node,
    leiden_partition,
    leiden_modularity
);

impl<'g> Louvain<'g> {
    pub fn new(
        graph: &'g impl GraphRef,
        refine: bool,
        gamma: f64,
        max_iter: u64,
        turbo: bool,
        recurse: bool,
    ) -> Result<Self> {
        Ok(Self {
            raw: ffi::new_louvain(graph.raw(), refine, gamma, max_iter, turbo, recurse)?,
            _graph: PhantomData,
        })
    }
}

impl<'g> Leiden<'g> {
    pub fn new(
        graph: &'g impl GraphRef,
        iterations: i32,
        randomize: bool,
        gamma: f64,
    ) -> Result<Self> {
        Ok(Self {
            raw: ffi::new_leiden(graph.raw(), iterations, randomize, gamma)?,
            _graph: PhantomData,
        })
    }

    pub fn load_move_scoring_extension(&mut self, path: impl AsRef<str>) -> Result<()> {
        ffi::leiden_load_move_scoring_extension(self.raw.pin_mut(), path.as_ref())
            .map_err(Error::from)
    }

    pub fn unload_move_scoring_extension(&mut self) -> Result<()> {
        ffi::leiden_unload_move_scoring_extension(self.raw.pin_mut()).map_err(Error::from)
    }
}

fn validate_primitive_array(name: &str, array: &UInt64Array) -> Result<()> {
    if array.null_count() != 0 {
        return Err(invalid_input(format!("{name} must not contain nulls")));
    }
    Ok(())
}

fn validate_float_array(name: &str, array: &Float64Array) -> Result<()> {
    if array.null_count() != 0 {
        return Err(invalid_input(format!("{name} must not contain nulls")));
    }
    Ok(())
}

fn validate_indptr(name: &str, indptr: &UInt64Array, n: u64, indices_len: usize) -> Result<()> {
    if indptr.len() != n as usize + 1 {
        return Err(invalid_input(format!("{name} length must be n + 1")));
    }
    let values = &indptr.values()[..];
    if values.first().copied() != Some(0) {
        return Err(invalid_input(format!("{name} must start at 0")));
    }
    if values.last().copied() != Some(indices_len as u64) {
        return Err(invalid_input(format!(
            "{name} last offset must equal indices length"
        )));
    }
    if values.windows(2).any(|w| w[0] > w[1]) {
        return Err(invalid_input(format!(
            "{name} must be monotonically increasing"
        )));
    }
    Ok(())
}

fn invalid_input(message: impl Into<String>) -> Error {
    Error::InvalidInput(message.into())
}
