mod graph;
mod introspection;

use std::error::Error;

pub fn run(config: clap::ArgMatches) -> Result<(), Box<dyn Error>> {
    let introspection_query_result_path = config.value_of("introspect-query-result-path").unwrap();
    let type_name = config.value_of("type").unwrap();
    let show_connections = config.is_present("expand-connections");
    let include_mutations = config.is_present("include-mutations");

    let schema = introspection::Schema::new(introspection_query_result_path)?;
    let graph = graph::Graph::new(schema, show_connections)?;
    let results = graph.enumerate_paths_to_target(type_name, include_mutations)?;

    print_results(type_name, graph, results);

    Ok(())
}

fn print_results(
    destination: &str,
    graph: graph::Graph,
    results: Vec<(graph::NodeIndex, Vec<graph::EdgeIndex>)>,
) {
    let number_of_results = &results.len();

    println!(
        "Found {} way{} to reach the \"{}\" node:",
        number_of_results,
        if *number_of_results == 1 { "" } else { "s" },
        destination
    );

    for (origin_node, edge_index_list) in results {
        print!("- {}", &graph.nodes[origin_node].name);
        for edge_index in edge_index_list {
            let edge = &graph.edges[edge_index];
            print!(
                " ({}) -> {}",
                edge.name, &graph.nodes[edge.destination].name
            )
        }
        println!();
    }
}
