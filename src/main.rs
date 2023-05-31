//#![allow(dead_code)]
//#![allow(unused_imports)]
//#![allow(unused_variables)]

use clap::Parser;
use comfy_table::*;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL};
use core::panic;
use dotenv::dotenv;
use futures::executor::block_on;
use openssl::ssl::{SslConnector, SslMethod};
use postgres::{types::ToSql, Client};
use postgres_openssl::MakeTlsConnector;
use serde::Deserialize;
use serde_json::{json, to_string_pretty, Value};
use std::{collections::HashMap, error::Error, vec::Vec};
mod neonutils;
mod networking;
use csv::StringRecord;

use crate::neonutils::{print_generic_json_table, reflective_get};
use crate::networking::*;

#[macro_use]
extern crate dotenv_codegen;

const NEON_BASE_URL: &str = "https://console.neon.tech/api/v2";

#[derive(Parser)]
#[command(author = "Tim Tully. <tim@menlovc.com>")]
#[command(about = "Neon Postgres Database CLI")]
struct Cli {
    #[clap(subcommand)]
    action: Action,
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
}
#[derive(clap::Subcommand, Debug)]
enum Action {
    #[clap(about = "Execute a query")]
    Query {
        #[arg(short, long, help = String::from("SQL query string to execute."))]
        sql: String,
    },
    #[clap(about = "Get information about projects in Neon.")]
    Projects {
        #[arg(short, long, help = String::from(r#"Format output for the projects. Can be one of "list-projects", "project-details", "delete-project""#))]
        action: String,
        #[arg(short, long, help = String::from(r#"The project identifier to use in the operation, if any. list-projects does not use this arg."#))]
        project: Option<String>,
        #[arg(short, long, default_value_t = String::from("json"), help = String::from(r#"Format output for the projects. Can be one of "json" or "table""#))]
        format: String,
    },
    #[clap(about = "Get information about keys in Neon.")]
    Keys {
        #[arg(short, long, help = String::from(r#"Keys action to take.  Can be one of "list", "create", or "revoke""#))]
        action: String,
        #[arg(short, long, help = String::from("Project the key belongs to."))]
        name: Option<String>,
        #[arg(short, long, default_value_t = String::from("json"), help = String::from(r#"Format output for the keys. Can be one of "json" or "table""#))]
        format: String,
    },
    #[clap(about = "Get information about branches in Neon.")]
    Branch {
        #[arg(short, long, help = String::from(r#"Branch action to be performed. Can be one of "list-branches"."#))]
        action: String,
        #[arg(short, long, help = String::from("Project the branch belongs to."))]
        project: Option<String>,
        #[arg(short, long, help = String::from("Branch to get data for."))]
        branch: Option<String>,
        #[arg(short, long, default_value_t = String::from("json"), help = String::from(r#"Format output for the keys. Can be one of "json" or "table""#))]
        format: String,
        #[clap(short, long)]
        roles: Option<String>,
    },
    #[clap(about = "Get information about endpoints in Neon.")]
    Endpoints {
        #[arg(short, long, help = String::from(r#"Endpoint action to be performed. Can be one of "start", "suspend", "list" or "details"."#))]
        action: String,
        #[arg(short, long, help = String::from("Project the endpoint belongs to."))]
        project: Option<String>,
        #[arg(short, long, help = String::from("The endpoint id."))]
        branch: Option<String>,
        #[arg(short, long, help = String::from("Branch to get data for."))]
        endpoint: Option<String>,
        #[arg(short, long, help = String::from("Config for endpoint create (json object as a string). See https://api-docs.neon.tech/reference/createprojectendpoint"))]
        initconfig: Option<String>,
    },
    #[clap(about = "Get information about operations in Neon.")]
    Operations {
        #[arg(short, long, help = String::from(r#"Action to be performed. Can be one of "list-operations" or "operation-details"."#))]
        action: String,
        #[arg(short, long, help = String::from("Project id to get operations for."))]
        project: Option<String>,
        #[arg(short, long, help = String::from("Identifier for an operation to get data for."))]
        operation: Option<String>,
        #[arg(short, long, default_value_t = String::from("json"), help = String::from(r#"Format output for the keys. Can be one of "json" or "table""#))]
        format: String,
    },
    #[clap(about = "Get information about consumption in Neon.")]
    Consumption {
        #[arg(short, long, help = String::from("Pagination limit for the report."))]
        limit: Option<u32>,
        #[arg(short, long, help = String::from("Cursor value used for next page in pagination."))]
        cursor: Option<String>,
    },
    #[clap(about = "Import data from csv file (TEXT only for now).")]
    Import {
        #[arg(short, long, help = String::from("The table to load data into."))]
        table: String,
        #[arg(short, long, help = String::from("The CSV file from while to load data. Ensure you have a header row at the top."))]
        file: String,
        #[arg(short, long, help = String::from("Delimiter used in the row.  Default is ','."))]
        delimiter: Option<String>,
    },
}

#[derive(Deserialize, Debug)]
pub struct NeonSession {
    database: String,
    neon_api_key: String,
    connect_string: String,
}

impl NeonSession {
    fn new(
        connect_string: &String,
        user: &String,
        password: &String,
        hostname: &String,
        database: &String,
        neon_api_key: &String,
    ) -> NeonSession {
        let mut final_connect: String = String::from(connect_string);
        if final_connect.is_empty() {
            final_connect = format!("postgres://{user}:{password}@{hostname}:5432/{database}");
        }
        NeonSession {
            database: database.clone(),
            neon_api_key: neon_api_key.clone(),
            connect_string: final_connect,
        }
    }

    fn connect(&self) -> Result<postgres::Client, Box<dyn std::error::Error>> {
        let builder = SslConnector::builder(SslMethod::tls())?;
        let connector = MakeTlsConnector::new(builder.build());
        let uri = format!("{}", self.connect_string);
        println!("uri is {uri}");
        let client = Client::connect(&uri, connector)?;
        Ok(client)
    }
}

struct Query {
    query: String,
}

impl Query {
    //https://github.com/sfackler/rust-postgres/issues/858
    fn query(&self, mut client: postgres::Client) {
        println!("Executing query: {}", self.query);
        let mut saw_first_row = false;
        let res = client.query(&self.query, &[]).unwrap();
        let mut num_cols: u64 = 0;
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec!["Header"]);

        for row in &res {
            if !saw_first_row {
                let col_names: Vec<String> =
                    row.columns().iter().map(|c| c.name().to_string()).collect();
                num_cols = col_names.len() as u64;
                table.set_header(col_names);
                saw_first_row = true;
            }
            let row_strs: Vec<String> = (0..num_cols)
                .map(|i: u64| reflective_get(row, i as usize))
                .collect();
            table.add_row(row_strs);
        }
        println!("{table}");
    }
}

impl Drop for Query {
    fn drop(&mut self) {}
}

fn initialize_env() -> NeonSession {
    let config = NeonSession::new(
        &dotenv!("CONNECT_STRING").to_string(),
        &dotenv!("USER").to_string(),
        &dotenv!("PASSWORD").to_string(),
        &dotenv!("HOSTNAME").to_string(),
        &dotenv!("DATABASE").to_string(),
        &dotenv!("NEON_API_KEY").to_string(),
    );
    return config;
}

fn build_uri(endpoint: String) -> String {
    format!("{}{endpoint}", NEON_BASE_URL.to_string())
}

// String in the Result is unformatted JSON (non-pretty).
fn handle_http_result(r: Result<String, Box<dyn Error>>) -> serde_json::Result<()> {
    match r {
        Ok(s) => {
            let json_blob: Value = serde_json::from_str(&s)?;
            let formatted: Result<String, serde_json::Error> = to_string_pretty(&json_blob);
            println!("{}", formatted.unwrap());
        }
        Err(e) => {
            panic!("Error: {e}");
        }
    }
    Ok(())
}

#[tokio::main]
async fn perform_keys_action(
    action: &String,
    name: &String,
    format: &String,
    neon_config: &NeonSession,
) {
    let r: Result<String, Box<dyn Error>>;
    match action {
        s if s == "list" => {
            let uri = build_uri("/api_keys".to_string());
            r = block_on(do_http_get(uri, &neon_config));
        }
        s if s == "create" => {
            let mut post_body: HashMap<String, String> = HashMap::new();
            post_body.insert("key_name".to_string(), name.to_string());
            let uri = build_uri("/api_keys".to_string());
            r = block_on(do_http_post(uri, &post_body, &neon_config));
        }
        s if s == "revoke" => {
            let uri = build_uri(format!("/api_keys/{}", name.to_string()));
            r = block_on(do_http_delete(uri, &neon_config));
        }
        _ => {
            panic!("Unknown Keys action.  Must specify correct aciton, use --help for list.");
        }
    }

    if format.is_empty() || format == "json" {
        handle_http_result(r).ok();
    } else if format == "table" {
        let json_str = r.unwrap();
        let parsed_array: Vec<Value> = serde_json::from_str(json_str.as_str()).unwrap();
        print_generic_json_table(&parsed_array);
    } else {
        panic!("Unknown format: {format}");
    }
}

#[tokio::main]
async fn perform_projects_action(
    action: &String,
    project: &String,
    format: &String,
    neon_config: &NeonSession,
) {
    let r: Result<String, Box<dyn Error>>;
    if action == "list-projects" {
        let uri = build_uri("/projects".to_string());
        r = block_on(do_http_get(uri, neon_config));
    } else if action == "project-details" {
        // target/debug/neon-cli projects -a project-details -p white-voice-129396
        let uri = build_uri(format!("/projects/{project}"));
        r = block_on(do_http_get(uri, neon_config));
    } else if action == "delete-project" {
        let uri = build_uri(format!("/projects/{project}"));
        r = block_on(do_http_delete(uri, neon_config));
    } else {
        panic!("Unknown Project Action: {action}");
    }
    handle_formatting_output(r, format, "projects");
}

// % target/debug/neon-cli branch -a list-roles -p white-voice-129396 -b br-dry-silence-599905
#[tokio::main]
async fn perform_branches_action(
    action: &String,
    project: &String,
    branch: &String,
    format: &String,
    role: &String,
    neon_config: &NeonSession,
) {
    let mut r: Result<String, Box<dyn Error>> = Ok("".to_string());
    let mut rows_key = "branches";

    if action == "list-endpoints" {
        let endpoint: String = format!("/projects/{project}/branches/{branch}/endpoints");
        rows_key = "endpoints";
        r = block_on(do_http_get(build_uri(endpoint), &neon_config));
    } else if action == "list-branches" {
        // target/debug/neon-cli branch -a list-branches -p white-voice-129396 -b br-dry-silence-599905
        let endpoint: String = format!("/projects/{project}/branches");
        r = block_on(do_http_get(build_uri(endpoint), &neon_config));
    } else if action == "list-roles" {
        // neon-cli branch -a list-roles -p white-voice-129396 -b br-dry-silence-599905
        let endpoint: String = format!("/projects/{project}/branches/{branch}/roles");
        rows_key = "roles";
        r = block_on(do_http_get(build_uri(endpoint), &neon_config));
    } else if action == "role-details" {
        // % target/debug/neon-cli branch -a role-details -p white-voice-129396 -b br-dry-silence-599905  -r tim
        if role.is_empty() {
            panic!("Role name is required");
        }
        rows_key = "role";
        let endpoint: String = format!("/projects/{project}/branches/{branch}/roles/{role}");
        r = block_on(do_http_get(build_uri(endpoint), &neon_config));
    } else if action == "role-delete" {
        if role.is_empty() {
            panic!("Role name is required");
        }
        let endpoint: String = format!("/projects/{project}/branches/{branch}/roles/{role}");
        r = block_on(do_http_delete(build_uri(endpoint), &neon_config));
    } else if action == "branch-details" {
        // target/debug/neon-cli branch -a branch-details -p white-voice-129396 -b br-dry-silence-599905  -f table
        let endpoint: String = format!("/projects/{project}/branches/{branch}");
        r = block_on(do_http_get(build_uri(endpoint), &neon_config));
    } else if action == "list-databases" {
        let endpoint: String = format!("/projects/{project}/branches/{branch}/databases");
        rows_key = "databases";
        r = block_on(do_http_get(build_uri(endpoint), &neon_config));
    } else if action == "database-details" {
        let endpoint: String = format!(
            "/projects/{project}/branches/{branch}/databases/{}",
            neon_config.database
        );
        r = block_on(do_http_get(build_uri(endpoint), &neon_config));
    } else if action == "delete-branch" {
        let endpoint: String = format!("/projects/{project}/branches/{branch}");
        r = block_on(do_http_delete(build_uri(endpoint), &neon_config));
    } else if action == "create-branch" {
    } else {
        panic!("Unknown Branch Action: {action}")
    }
    handle_formatting_output(r, format, rows_key);
}

#[tokio::main]
async fn perform_endpoints_action(
    action: &String,
    project: &String,
    endpoint: &String,
    branch: &String,
    config: &String, // the endpoint configuration, not the postgres setup
    neon_config: &NeonSession,
) {
    let r: Result<String, Box<dyn Error>>;
    if action == "create" {
        // target/debug/neon-cli endpoints -a create  -p white-voice-129396 --initconfig='{"type": "read_write","pooler_mode": "transaction","branch_id": "asdf","autoscaling_limit_min_cu": 2,"autoscaling_limit_max_cu": 2}' -b br-dry-silence-599905
        let uri: String = format!("/projects/{project}/endpoints");
        if config.is_empty() {
            panic!("Missing or empty configuration for new endpoint.  Use the --initconfig param.")
        }
        let json_value: Result<Value, serde_json::Error> = serde_json::from_str(config);
        let mut final_obj = json!({
            "endpoint": json_value.unwrap(),
        });
        final_obj["endpoint"]["branch_id"] = json!(branch);
        r = block_on(do_http_post_text(
            build_uri(uri),
            &final_obj.to_string(),
            &neon_config,
        ));
    } else if action == "list" {
        // target/debug/neon-cli endpoints -a list -p white-voice-129396
        let uri: String = format!("/projects/{project}/endpoints");
        r = block_on(do_http_get(build_uri(uri), &neon_config));
    } else if action == "details" {
        let uri: String = format!("/projects/{project}/endpoints/{endpoint}");
        r = block_on(do_http_get(build_uri(uri), &neon_config));
    } else if action == "delete" {
        let uri: String = format!("/projects/{project}/endpoints/{endpoint}");
        r = block_on(do_http_delete(build_uri(uri), &neon_config));
    } else if action == "start" || action == "suspend" {
        if endpoint.is_empty() {
            panic!("Endpoint name is required");
        }
        let uri: String = format!("/projects/{project}/endpoints/{endpoint}/{action}");
        let post_body: HashMap<String, String> = HashMap::new();
        r = block_on(do_http_post(build_uri(uri), &post_body, &neon_config));
    } else {
        panic!("Unknown Endpoints Action: {action}");
    }
    handle_http_result(r).ok();
}

#[tokio::main]
async fn perform_consumption_action(limit: u32, cursor: &String, neon_config: &NeonSession) {
    let endpoint: String = format!("/consumption/projects?cursor={cursor}&limit={limit}");
    let r = block_on(do_http_get(build_uri(endpoint), &neon_config));
    handle_http_result(r).ok();
}

#[tokio::main]
async fn perform_operations_action(
    action: &String,
    project: &String,
    operation: &String,
    format: &String,
    neon_config: &NeonSession,
) {
    let r: Result<String, Box<dyn Error>>;
    if action == "list-operations" {
        let endpoint: String = format!("/projects/{project}/operations");
        r = block_on(do_http_get(build_uri(endpoint), &neon_config));
    } else if action == "operation-details" {
        if operation.is_empty() {
            panic!("Operation ID is required");
        }
        let endpoint: String = format!("/projects/{project}/operations/{operation}");
        r = block_on(do_http_get(build_uri(endpoint), &neon_config));
    } else {
        panic!("Unknown Operation Action: {action}");
    }
    handle_formatting_output(r, format, "operations");
}

fn handle_formatting_output(r: Result<String, Box<dyn Error>>, format: &String, rows_key: &str) {
    if format.is_empty() || format == "json" {
        handle_http_result(r).ok();
    } else if format == "table" {
        let json_blob: Value = serde_json::from_str(&r.unwrap()).unwrap();
        let candidate = json_blob[rows_key].as_array();
        let mut rows: Vec<Value> = Vec::new();
        if candidate.is_none() {
            if let Some((_key, value)) = json_blob.as_object().unwrap().iter().next() {
                rows = vec![value.clone()];
            }
        } else {
            rows = json_blob[rows_key]
                .as_array()
                .expect("No rows found in response")
                .to_vec();
        }
        print_generic_json_table(&rows);
    } else {
        panic!("Unknown format: {format}");
    }
}

#[inline(always)]
fn add_conditionally(
    record: &StringRecord,
    params: &mut Vec<Box<dyn ToSql + Sync>>,
    column_types: &Vec<String>,
) {
    for i in 0..record.len() {
        let ct = &column_types[i];
        match ct.as_str() {
            "text" | "character varying" | "varchar" => {
                params.push(Box::new(
                    record[i]
                        .parse::<String>()
                        .expect("Expected string in column."),
                ));
            }
            "smallint" => {
                params.push(Box::new(
                    record[i]
                        .parse::<i16>()
                        .expect("Excpted smallint in column."),
                ));
            }
            "integer" | "int" | "int4" => {
                params.push(Box::new(
                    record[i]
                        .parse::<i32>()
                        .expect("Expected integer in column."),
                ));
            }
            "real" | "float8" => {
                params.push(Box::new(
                    record[i].parse::<f64>().expect("Expected float in column."),
                ));
            }
            "bigint" | "int8" => {
                params.push(Box::new(
                    record[i].parse::<i64>().expect("Expeted bigint in column."),
                ));
            }
            "bool" | "boolean" => {
                params.push(Box::new(
                    record[i]
                        .parse::<bool>()
                        .expect("Expected boolean in column."),
                ));
            }
            _ => {
                panic!("Unknown column type: {ct}");
            }
        }
    }
}

fn perform_import_action(
    table: &String,
    file: &String,
    delimiter: &String,
    neon_config: &NeonSession,
) -> Result<(), Box<dyn Error>> {
    let mut client = neon_config.connect().expect("couldn't connect");

    // grab type so that when we insert from CSV later, we can parse from StringRecord properly before INSERTs run
    let q = format!(
        "SELECT column_name, data_type FROM information_schema.columns WHERE table_name = '{}';",
        table
    );
    let res = client.query(&q, &[]).unwrap();
    let mut column_types: Vec<String> = Vec::new();
    // grab column types for each column (a row here is a column description)
    for row in &res {
        let col_type: String = row.get("data_type");
        column_types.push(col_type);
    }

    let rdr = csv::ReaderBuilder::new()
        .delimiter(delimiter.as_bytes()[0])
        .from_path(file);
    let mut binding = rdr.unwrap();
    let records = binding.records();

    let mut params = Vec::<Box<dyn ToSql + Sync>>::new();
    let num_cols: u16 = column_types.len() as u16;

    #[inline(always)]
    fn format_row_params(num_params: u32, row_num: u32) -> String {
        let range = 1..=num_params; // Create a range from start to end (inclusive)
        let mapped_values: String = range
            .map(|i| format!("${}", i + row_num * num_params))
            .collect::<Vec<String>>()
            .join(",");
        return format!("({mapped_values})");
    }

    // Build the parameterized portion of the insert statement separately
    let vv = records
        .enumerate()
        .map(|(index, row)| {
            let record = row.unwrap();
            add_conditionally(&record, &mut params, &column_types);
            format_row_params(num_cols as u32, index as u32)
        })
        .collect::<Vec<String>>()
        .join(",");

    // Rebuild the params from heap values
    let param_values: Vec<&(dyn ToSql + Sync)> = params
        .iter()
        .map(|x: &Box<dyn ToSql + Sync>| &**x)
        .collect::<Vec<_>>();

    let final_stmt = format!("INSERT INTO {table} VALUES {vv}");

    client
        .execute(final_stmt.as_str(), &param_values)
        .expect("Couldn't execute prepared stmt.");

    Ok(())
}

fn main() {
    let cli = Cli::parse();
    let subcommand = cli.action;
    dotenv().ok();

    let config = initialize_env();

    match subcommand {
        Action::Query { sql } => {
            let c = config.connect().expect("couldn't connect");
            let q: Query = Query { query: sql };
            q.query(c);
        }
        Action::Projects {
            action,
            project,
            format,
        } => {
            let p = project.unwrap_or("".to_string()); // project id
            perform_projects_action(&action, &p, &format, &config)
        }
        Action::Keys {
            action,
            name,
            format,
        } => {
            let name = name.unwrap_or("".to_string()); // name of the key to create
            perform_keys_action(&action, &name, &format, &config);
        }
        Action::Branch {
            action,
            project,
            branch,
            format,
            roles,
        } => {
            let p = project.unwrap_or("".to_string());
            let b: String = branch.unwrap_or("".to_string());
            let r: String = roles.unwrap_or("".to_string());
            perform_branches_action(&action, &p, &b, &format, &r, &config);
        }
        Action::Endpoints {
            action,
            project,
            branch,
            endpoint,
            initconfig,
        } => {
            let p = project.unwrap_or("".to_string());
            let b: String = branch.unwrap_or("".to_string());
            let e: String = endpoint.unwrap_or("".to_string());
            let params: String = initconfig.unwrap_or("".to_string()); // the json blob for endpoint config
            perform_endpoints_action(&action, &p, &e, &b, &params, &config);
        }
        Action::Consumption { limit, cursor } => {
            let limit = limit.unwrap_or(16);
            let cursor: String = cursor.unwrap_or("".to_string());
            perform_consumption_action(limit, &cursor, &config);
        }
        Action::Operations {
            action,
            project,
            operation,
            format,
        } => {
            let p = project.expect("Project ID is required for operations");
            let o: String = operation.unwrap_or("".to_string());
            perform_operations_action(&action, &p, &o, &format, &config);
        }
        Action::Import {
            table,
            file,
            delimiter,
        } => {
            let _delim = delimiter.unwrap_or(",".to_string());
            perform_import_action(&table, &file, &_delim, &config).unwrap();
        }
    }
}
