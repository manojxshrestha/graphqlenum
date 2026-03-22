extern crate clap;

use clap::{App, Arg};
use std::process;

fn main() {
    let matches = App::new("graphqlenum")
        .author("manojxshrestha")
        .about("GraphQL Path Enumeration & Security Auditor")
        .arg(
            Arg::with_name("introspect-query-result-path")
                .short("i")
                .long("introspect-query-path")
                .help("Path to the introspection query result saved as JSON.")
                .required(true)
                .value_name("FILE_PATH")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("type")
                .short("t")
                .long("type")
                .help("The type to look for in the graph.")
                .required(true)
                .value_name("TYPE_NAME")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("expand-connections")
                .long("expand-connections")
                .help("Expand connection nodes (with pageInfo, edges, etc. edges), they are skipped by default.")
        )
        .arg(
            Arg::with_name("include-mutations")
                .long("include-mutations")
                .help("Include paths from the Mutation node. Off by default because this often adds a lot of noise.")
        )
        .get_matches();

    if let Err(e) = graphql_path_enum::run(matches) {
        eprintln!("Runtime error: {}", e);

        process::exit(1);
    }
}
