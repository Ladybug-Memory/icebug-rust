#pragma once

#include "rust/cxx.h"

#include <networkit/centrality/Betweenness.hpp>
#include <networkit/centrality/DegreeCentrality.hpp>
#include <networkit/centrality/PageRank.hpp>
#include <networkit/community/PLM.hpp>
#include <networkit/community/ParallelLeidenView.hpp>
#include <networkit/components/ConnectedComponents.hpp>
#include <networkit/graph/Graph.hpp>
#include <networkit/graph/GraphR.hpp>
#include <networkit/graph/GraphW.hpp>

#include <memory>
#include <string>
#include <variant>

namespace icebug_rust {

struct Edge;
struct WeightedEdge;
struct RankingEntry;
struct PartitionResult;

class IcebugGraph {
public:
    explicit IcebugGraph(std::unique_ptr<NetworKit::GraphW> graph);
    explicit IcebugGraph(std::unique_ptr<NetworKit::GraphR> graph);

    NetworKit::Graph &graph();
    const NetworKit::Graph &graph() const;
    NetworKit::GraphW &mutable_graph();

private:
    // Owns the concrete graph storage; the view_ handle references one of these arms.
    std::variant<std::unique_ptr<NetworKit::GraphW>, std::unique_ptr<NetworKit::GraphR>> storage_;
    // Non-owning handle (NetworKit::Graph == ReferenceGraph) kept stable so algorithms that
    // store a `const Graph &` bind to a member, not a temporary.
    NetworKit::Graph view_;
};

class Betweenness {
public:
    Betweenness(const IcebugGraph &graph, bool normalized, bool compute_edge_centrality);
    NetworKit::Betweenness algo;
};

class PageRank {
public:
    PageRank(const IcebugGraph &graph, double damp, double tol, bool normalized);
    NetworKit::PageRank algo;
};

class DegreeCentrality {
public:
    DegreeCentrality(const IcebugGraph &graph, bool normalized, bool out_deg,
                     bool ignore_self_loops);
    NetworKit::DegreeCentrality algo;
};

class ConnectedComponents {
public:
    explicit ConnectedComponents(const IcebugGraph &graph);
    NetworKit::ConnectedComponents algo;
};

class Louvain {
public:
    Louvain(const IcebugGraph &graph, bool refine, double gamma, uint64_t max_iter, bool turbo,
            bool recurse);

    const NetworKit::Graph *graph;
    NetworKit::PLM algo;
};

class Leiden {
public:
    Leiden(const IcebugGraph &graph, int32_t iterations, bool randomize, double gamma);

    const NetworKit::Graph *graph;
    NetworKit::ParallelLeidenView algo;
};

std::unique_ptr<IcebugGraph> new_graph(uint64_t n, bool weighted, bool directed);
std::unique_ptr<IcebugGraph> new_graph_r(uint64_t n, bool directed,
                                         rust::Slice<const uint64_t> out_indices,
                                         rust::Slice<const uint64_t> out_indptr,
                                         rust::Slice<const uint64_t> in_indices,
                                         rust::Slice<const uint64_t> in_indptr,
                                         rust::Slice<const double> out_weights,
                                         rust::Slice<const double> in_weights);
std::unique_ptr<IcebugGraph> read_metis(rust::Str path);
std::unique_ptr<IcebugGraph> read_edge_list(rust::Str path, rust::Str separator,
                                            uint64_t first_node, bool directed,
                                            rust::Str comment_prefix);

uint64_t graph_number_of_nodes(const IcebugGraph &graph);
uint64_t graph_number_of_edges(const IcebugGraph &graph);
uint64_t graph_upper_node_id_bound(const IcebugGraph &graph);
bool graph_is_weighted(const IcebugGraph &graph);
bool graph_is_directed(const IcebugGraph &graph);
bool graph_is_empty(const IcebugGraph &graph);
uint64_t graph_add_node(IcebugGraph &graph);
bool graph_add_edge(IcebugGraph &graph, uint64_t u, uint64_t v, double weight);
void graph_remove_node(IcebugGraph &graph, uint64_t u);
void graph_remove_edge(IcebugGraph &graph, uint64_t u, uint64_t v);
void graph_set_weight(IcebugGraph &graph, uint64_t u, uint64_t v, double weight);
bool graph_has_node(const IcebugGraph &graph, uint64_t u);
bool graph_has_edge(const IcebugGraph &graph, uint64_t u, uint64_t v);
uint64_t graph_degree(const IcebugGraph &graph, uint64_t u);
uint64_t graph_degree_in(const IcebugGraph &graph, uint64_t u);
double graph_weight(const IcebugGraph &graph, uint64_t u, uint64_t v);
rust::Vec<uint64_t> graph_nodes(const IcebugGraph &graph);
rust::Vec<uint64_t> graph_neighbors(const IcebugGraph &graph, uint64_t u);
rust::Vec<Edge> graph_edges(const IcebugGraph &graph);
rust::Vec<WeightedEdge> graph_weighted_edges(const IcebugGraph &graph);

std::unique_ptr<Betweenness> new_betweenness(const IcebugGraph &graph, bool normalized,
                                             bool compute_edge_centrality);
std::unique_ptr<PageRank> new_page_rank(const IcebugGraph &graph, double damp, double tol,
                                        bool normalized);
std::unique_ptr<DegreeCentrality> new_degree_centrality(const IcebugGraph &graph,
                                                        bool normalized, bool out_deg,
                                                        bool ignore_self_loops);
std::unique_ptr<ConnectedComponents> new_connected_components(const IcebugGraph &graph);
std::unique_ptr<Louvain> new_louvain(const IcebugGraph &graph, bool refine, double gamma,
                                     uint64_t max_iter, bool turbo, bool recurse);
std::unique_ptr<Leiden> new_leiden(const IcebugGraph &graph, int32_t iterations,
                                   bool randomize, double gamma);

void betweenness_run(Betweenness &algo);
bool betweenness_has_finished(const Betweenness &algo);
rust::Vec<double> betweenness_scores(const Betweenness &algo);
double betweenness_score(Betweenness &algo, uint64_t node);
rust::Vec<RankingEntry> betweenness_ranking(Betweenness &algo);
double betweenness_maximum(Betweenness &algo);

void page_rank_run(PageRank &algo);
bool page_rank_has_finished(const PageRank &algo);
rust::Vec<double> page_rank_scores(const PageRank &algo);
double page_rank_score(PageRank &algo, uint64_t node);
rust::Vec<RankingEntry> page_rank_ranking(PageRank &algo);
uint64_t page_rank_number_of_iterations(const PageRank &algo);

void degree_centrality_run(DegreeCentrality &algo);
bool degree_centrality_has_finished(const DegreeCentrality &algo);
rust::Vec<double> degree_centrality_scores(const DegreeCentrality &algo);
double degree_centrality_score(DegreeCentrality &algo, uint64_t node);
rust::Vec<RankingEntry> degree_centrality_ranking(DegreeCentrality &algo);

void connected_components_run(ConnectedComponents &algo);
bool connected_components_has_finished(const ConnectedComponents &algo);
uint64_t connected_components_number_of_components(const ConnectedComponents &algo);
uint64_t connected_components_component_of_node(const ConnectedComponents &algo, uint64_t node);
rust::Vec<uint64_t> connected_components_component_sizes(const ConnectedComponents &algo);

void louvain_run(Louvain &algo);
bool louvain_has_finished(const Louvain &algo);
uint64_t louvain_number_of_communities(Louvain &algo);
uint64_t louvain_community_of_node(Louvain &algo, uint64_t node);
PartitionResult louvain_partition(Louvain &algo);
double louvain_modularity(Louvain &algo);

void leiden_run(Leiden &algo);
bool leiden_has_finished(const Leiden &algo);
uint64_t leiden_number_of_communities(Leiden &algo);
uint64_t leiden_community_of_node(Leiden &algo, uint64_t node);
PartitionResult leiden_partition(Leiden &algo);
double leiden_modularity(Leiden &algo);
void leiden_load_move_scoring_extension(Leiden &algo, rust::Str path);
void leiden_unload_move_scoring_extension(Leiden &algo);

} // namespace icebug_rust
