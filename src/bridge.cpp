#include "icebug_rust.hpp"
#include "icebug/src/lib.rs.h"

#include <networkit/centrality/Betweenness.hpp>
#include <networkit/centrality/DegreeCentrality.hpp>
#include <networkit/centrality/PageRank.hpp>
#include <networkit/community/Modularity.hpp>
#include <networkit/community/PLM.hpp>
#include <networkit/community/ParallelLeidenView.hpp>
#include <networkit/components/ConnectedComponents.hpp>
#include <networkit/graph/GraphR.hpp>
#include <networkit/graph/GraphW.hpp>
#include <networkit/io/EdgeListReader.hpp>
#include <networkit/io/METISGraphReader.hpp>

#include <arrow/api.h>

#include <map>
#include <memory>
#include <string>
#include <utility>
#include <vector>

namespace icebug_rust {
namespace {

using NetworKit::count;
using NetworKit::edgeweight;
using NetworKit::Graph;
using NetworKit::GraphR;
using NetworKit::GraphW;
using NetworKit::node;

std::string to_string(rust::Str value) { return std::string(value.data(), value.size()); }

std::shared_ptr<arrow::UInt64Array> make_u64_array(rust::Slice<const uint64_t> values) {
    if (values.empty()) {
        return nullptr;
    }
    auto buffer = std::make_shared<arrow::Buffer>(
        reinterpret_cast<const uint8_t *>(values.data()),
        static_cast<int64_t>(values.size() * sizeof(uint64_t)));
    return std::make_shared<arrow::UInt64Array>(static_cast<int64_t>(values.size()), buffer);
}

std::shared_ptr<arrow::DoubleArray> make_f64_array(rust::Slice<const double> values) {
    if (values.empty()) {
        return nullptr;
    }
    auto buffer = std::make_shared<arrow::Buffer>(
        reinterpret_cast<const uint8_t *>(values.data()),
        static_cast<int64_t>(values.size() * sizeof(double)));
    return std::make_shared<arrow::DoubleArray>(static_cast<int64_t>(values.size()), buffer);
}

rust::Vec<double> copy_scores(const std::vector<double> &scores) {
    rust::Vec<double> out;
    out.reserve(scores.size());
    for (double score : scores) {
        out.push_back(score);
    }
    return out;
}

rust::Vec<RankingEntry> copy_ranking(const std::vector<std::pair<node, double>> &ranking) {
    rust::Vec<RankingEntry> out;
    out.reserve(ranking.size());
    for (const auto &[u, score] : ranking) {
        out.push_back(RankingEntry{u, score});
    }
    return out;
}

PartitionResult partition_result(const NetworKit::Partition &partition) {
    const auto &vec = partition.getVector();
    rust::Vec<uint64_t> membership;
    membership.reserve(vec.size());
    for (auto community : vec) {
        membership.push_back(static_cast<uint64_t>(community));
    }
    return PartitionResult{std::move(membership), partition.numberOfSubsets()};
}

} // namespace

IcebugGraph::IcebugGraph(std::unique_ptr<Graph> graph) : graph_(std::move(graph)) {}

Graph &IcebugGraph::graph() { return *graph_; }
const Graph &IcebugGraph::graph() const { return *graph_; }

GraphW &IcebugGraph::mutable_graph() {
    auto *graph = dynamic_cast<GraphW *>(graph_.get());
    if (graph == nullptr) {
        throw std::runtime_error("operation requires mutable Graph, not GraphR");
    }
    return *graph;
}

Betweenness::Betweenness(const IcebugGraph &graph, bool normalized, bool compute_edge_centrality)
    : algo(graph.graph(), normalized, compute_edge_centrality) {}

PageRank::PageRank(const IcebugGraph &graph, double damp, double tol, bool normalized)
    : algo(graph.graph(), damp, tol, normalized) {}

DegreeCentrality::DegreeCentrality(const IcebugGraph &graph, bool normalized, bool out_deg,
                                   bool ignore_self_loops)
    : algo(graph.graph(), normalized, out_deg, ignore_self_loops) {}

ConnectedComponents::ConnectedComponents(const IcebugGraph &graph) : algo(graph.graph()) {}

Louvain::Louvain(const IcebugGraph &graph, bool refine, double gamma, uint64_t max_iter,
                 bool turbo, bool recurse)
    : graph(&graph.graph()),
      algo(graph.graph(), refine, gamma, "balanced", max_iter, turbo, recurse) {}

Leiden::Leiden(const IcebugGraph &graph, int32_t iterations, bool randomize, double gamma)
    : graph(&graph.graph()), algo(graph.graph(), iterations, randomize, gamma) {}

std::unique_ptr<IcebugGraph> new_graph(uint64_t n, bool weighted, bool directed) {
    return std::make_unique<IcebugGraph>(
        std::make_unique<GraphW>(static_cast<count>(n), weighted, directed));
}

std::unique_ptr<IcebugGraph> new_graph_r(uint64_t n, bool directed,
                                         rust::Slice<const uint64_t> out_indices,
                                         rust::Slice<const uint64_t> out_indptr,
                                         rust::Slice<const uint64_t> in_indices,
                                         rust::Slice<const uint64_t> in_indptr,
                                         rust::Slice<const double> out_weights,
                                         rust::Slice<const double> in_weights) {
    return std::make_unique<IcebugGraph>(std::make_unique<GraphR>(
        static_cast<count>(n), directed, make_u64_array(out_indices), make_u64_array(out_indptr),
        make_u64_array(in_indices), make_u64_array(in_indptr), make_f64_array(out_weights),
        make_f64_array(in_weights)));
}

std::unique_ptr<IcebugGraph> read_metis(rust::Str path) {
    NetworKit::METISGraphReader reader;
    return std::make_unique<IcebugGraph>(
        std::make_unique<GraphW>(reader.read(to_string(path))));
}

std::unique_ptr<IcebugGraph> read_edge_list(rust::Str path, rust::Str separator,
                                            uint64_t first_node, bool directed,
                                            rust::Str comment_prefix) {
    const auto sep = to_string(separator);
    if (sep.size() != 1) {
        throw std::runtime_error("separator must be exactly one byte");
    }
    NetworKit::EdgeListReader reader(sep[0], static_cast<node>(first_node),
                                     to_string(comment_prefix), true, directed);
    return std::make_unique<IcebugGraph>(
        std::make_unique<GraphW>(reader.read(to_string(path))));
}

uint64_t graph_number_of_nodes(const IcebugGraph &graph) { return graph.graph().numberOfNodes(); }
uint64_t graph_number_of_edges(const IcebugGraph &graph) { return graph.graph().numberOfEdges(); }
uint64_t graph_upper_node_id_bound(const IcebugGraph &graph) {
    return graph.graph().upperNodeIdBound();
}
bool graph_is_weighted(const IcebugGraph &graph) { return graph.graph().isWeighted(); }
bool graph_is_directed(const IcebugGraph &graph) { return graph.graph().isDirected(); }
bool graph_is_empty(const IcebugGraph &graph) { return graph.graph().isEmpty(); }

uint64_t graph_add_node(IcebugGraph &graph) { return graph.mutable_graph().addNode(); }
bool graph_add_edge(IcebugGraph &graph, uint64_t u, uint64_t v, double weight) {
    return graph.mutable_graph().addEdge(u, v, weight);
}
void graph_remove_node(IcebugGraph &graph, uint64_t u) { graph.mutable_graph().removeNode(u); }
void graph_remove_edge(IcebugGraph &graph, uint64_t u, uint64_t v) {
    graph.mutable_graph().removeEdge(u, v);
}
void graph_set_weight(IcebugGraph &graph, uint64_t u, uint64_t v, double weight) {
    graph.mutable_graph().setWeight(u, v, weight);
}
bool graph_has_node(const IcebugGraph &graph, uint64_t u) { return graph.graph().hasNode(u); }
bool graph_has_edge(const IcebugGraph &graph, uint64_t u, uint64_t v) {
    return graph.graph().hasEdge(u, v);
}
uint64_t graph_degree(const IcebugGraph &graph, uint64_t u) { return graph.graph().degree(u); }
uint64_t graph_degree_in(const IcebugGraph &graph, uint64_t u) { return graph.graph().degreeIn(u); }
double graph_weight(const IcebugGraph &graph, uint64_t u, uint64_t v) {
    return graph.graph().weight(u, v);
}

rust::Vec<uint64_t> graph_nodes(const IcebugGraph &graph) {
    rust::Vec<uint64_t> out;
    graph.graph().forNodes([&](node u) { out.push_back(u); });
    return out;
}

rust::Vec<uint64_t> graph_neighbors(const IcebugGraph &graph, uint64_t u) {
    rust::Vec<uint64_t> out;
    graph.graph().forNeighborsOf(u, [&](node v) { out.push_back(v); });
    return out;
}

rust::Vec<Edge> graph_edges(const IcebugGraph &graph) {
    rust::Vec<Edge> out;
    graph.graph().forEdges([&](node u, node v) { out.push_back(Edge{u, v}); });
    return out;
}

rust::Vec<WeightedEdge> graph_weighted_edges(const IcebugGraph &graph) {
    rust::Vec<WeightedEdge> out;
    graph.graph().forEdges([&](node u, node v, edgeweight w) {
        out.push_back(WeightedEdge{u, v, w});
    });
    return out;
}

std::unique_ptr<Betweenness> new_betweenness(const IcebugGraph &graph, bool normalized,
                                             bool compute_edge_centrality) {
    return std::make_unique<Betweenness>(graph, normalized, compute_edge_centrality);
}
std::unique_ptr<PageRank> new_page_rank(const IcebugGraph &graph, double damp, double tol,
                                        bool normalized) {
    return std::make_unique<PageRank>(graph, damp, tol, normalized);
}
std::unique_ptr<DegreeCentrality> new_degree_centrality(const IcebugGraph &graph, bool normalized,
                                                        bool out_deg, bool ignore_self_loops) {
    return std::make_unique<DegreeCentrality>(graph, normalized, out_deg, ignore_self_loops);
}
std::unique_ptr<ConnectedComponents> new_connected_components(const IcebugGraph &graph) {
    return std::make_unique<ConnectedComponents>(graph);
}
std::unique_ptr<Louvain> new_louvain(const IcebugGraph &graph, bool refine, double gamma,
                                     uint64_t max_iter, bool turbo, bool recurse) {
    return std::make_unique<Louvain>(graph, refine, gamma, max_iter, turbo, recurse);
}
std::unique_ptr<Leiden> new_leiden(const IcebugGraph &graph, int32_t iterations, bool randomize,
                                   double gamma) {
    return std::make_unique<Leiden>(graph, iterations, randomize, gamma);
}

void betweenness_run(Betweenness &algo) { algo.algo.run(); }
bool betweenness_has_finished(const Betweenness &algo) { return algo.algo.hasFinished(); }
rust::Vec<double> betweenness_scores(const Betweenness &algo) {
    algo.algo.assureFinished();
    return copy_scores(algo.algo.scores());
}
double betweenness_score(Betweenness &algo, uint64_t node) { return algo.algo.score(node); }
rust::Vec<RankingEntry> betweenness_ranking(Betweenness &algo) {
    return copy_ranking(algo.algo.ranking());
}
double betweenness_maximum(Betweenness &algo) { return algo.algo.maximum(); }

void page_rank_run(PageRank &algo) { algo.algo.run(); }
bool page_rank_has_finished(const PageRank &algo) { return algo.algo.hasFinished(); }
rust::Vec<double> page_rank_scores(const PageRank &algo) {
    algo.algo.assureFinished();
    return copy_scores(algo.algo.scores());
}
double page_rank_score(PageRank &algo, uint64_t node) { return algo.algo.score(node); }
rust::Vec<RankingEntry> page_rank_ranking(PageRank &algo) {
    return copy_ranking(algo.algo.ranking());
}
uint64_t page_rank_number_of_iterations(const PageRank &algo) {
    return algo.algo.numberOfIterations();
}

void degree_centrality_run(DegreeCentrality &algo) { algo.algo.run(); }
bool degree_centrality_has_finished(const DegreeCentrality &algo) {
    return algo.algo.hasFinished();
}
rust::Vec<double> degree_centrality_scores(const DegreeCentrality &algo) {
    algo.algo.assureFinished();
    return copy_scores(algo.algo.scores());
}
double degree_centrality_score(DegreeCentrality &algo, uint64_t node) {
    return algo.algo.score(node);
}
rust::Vec<RankingEntry> degree_centrality_ranking(DegreeCentrality &algo) {
    return copy_ranking(algo.algo.ranking());
}

void connected_components_run(ConnectedComponents &algo) { algo.algo.run(); }
bool connected_components_has_finished(const ConnectedComponents &algo) {
    return algo.algo.hasFinished();
}
uint64_t connected_components_number_of_components(const ConnectedComponents &algo) {
    return algo.algo.numberOfComponents();
}
uint64_t connected_components_component_of_node(const ConnectedComponents &algo, uint64_t node) {
    return algo.algo.componentOfNode(node);
}
rust::Vec<uint64_t> connected_components_component_sizes(const ConnectedComponents &algo) {
    rust::Vec<uint64_t> out;
    auto sizes = algo.algo.getComponentSizes();
    out.reserve(sizes.size());
    for (const auto &[_, size] : sizes) {
        out.push_back(size);
    }
    return out;
}

void louvain_run(Louvain &algo) { algo.algo.run(); }
bool louvain_has_finished(const Louvain &algo) { return algo.algo.hasFinished(); }
uint64_t louvain_number_of_communities(Louvain &algo) {
    algo.algo.assureFinished();
    return algo.algo.getPartition().numberOfSubsets();
}
uint64_t louvain_community_of_node(Louvain &algo, uint64_t node) {
    algo.algo.assureFinished();
    return algo.algo.getPartition().subsetOf(node);
}
PartitionResult louvain_partition(Louvain &algo) {
    algo.algo.assureFinished();
    return partition_result(algo.algo.getPartition());
}
double louvain_modularity(Louvain &algo) {
    algo.algo.assureFinished();
    NetworKit::Modularity modularity;
    return modularity.getQuality(algo.algo.getPartition(), *algo.graph);
}

void leiden_run(Leiden &algo) { algo.algo.run(); }
bool leiden_has_finished(const Leiden &algo) { return algo.algo.hasFinished(); }
uint64_t leiden_number_of_communities(Leiden &algo) {
    algo.algo.assureFinished();
    return algo.algo.getPartition().numberOfSubsets();
}
uint64_t leiden_community_of_node(Leiden &algo, uint64_t node) {
    algo.algo.assureFinished();
    return algo.algo.getPartition().subsetOf(node);
}
PartitionResult leiden_partition(Leiden &algo) {
    algo.algo.assureFinished();
    return partition_result(algo.algo.getPartition());
}
double leiden_modularity(Leiden &algo) {
    algo.algo.assureFinished();
    NetworKit::Modularity modularity;
    return modularity.getQuality(algo.algo.getPartition(), *algo.graph);
}
void leiden_load_move_scoring_extension(Leiden &algo, rust::Str path) {
    algo.algo.loadMoveScoringExtension(to_string(path));
}
void leiden_unload_move_scoring_extension(Leiden &algo) { algo.algo.unloadMoveScoringExtension(); }

} // namespace icebug_rust
