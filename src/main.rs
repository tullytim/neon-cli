#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use clap::Parser;
use comfy_table::*;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL};
use dotenv::dotenv;
use futures::executor::block_on;
use openssl::ssl::{SslConnector, SslMethod};
use postgres::Client;
use postgres_openssl::MakeTlsConnector;
use serde::Deserialize;
use serde_json::to_string_pretty;
use serde_json::Value;
use std::collections::HashMap;
use std::{error::Error, io};
mod neonutils;
mod networking;
use crate::neonutils::reflective_get;
use crate::networking::{do_http_delete, do_http_get, do_http_post};

#[macro_use]
extern crate dotenv_codegen;

const NEON_BASE_URL: &str = "https://console.neon.tech/api/v2";

#[derive(Parser)]
#[command(author = "Tim Tully. <tim@menlovc.com>")]
#[command(about = "Does awesome things")]
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
        #[clap(short, long)]
        sql: String,
    },
    #[clap(about = "Get information about projects in Neon.")]
    Projects {
        #[clap(short, long)]
        project: Option<String>,
    },
    #[clap(about = "Get information about keys in Neon.")]
    Keys {
        #[clap(short, long)]
        action: String,
        #[clap(short, long)]
        name: Option<String>,
    },
    #[clap(about = "Get information about branches in Neon.")]
    Branch {
        #[clap(short, long)]
        action: String,
        #[clap(short, long)]
        project: Option<String>,
        #[clap(short, long)]
        branch: Option<String>,
        #[clap(short, long)]
        roles: Option<String>,
    },
}

#[derive(Deserialize, Debug)]
pub struct NeonSession {
    user: String,
    password: String,
    hostname: String,
    database: String,
    neon_api_key: String,
}

impl NeonSession {
    fn new() -> NeonSession {
        NeonSession {
            user: String::from(""),
            password: String::from(""),
            hostname: String::from(""),
            database: String::from(""),
            neon_api_key: String::from(""),
        }
    }

    fn connect(&self) -> Result<postgres::Client, Box<dyn std::error::Error>> {
        let builder = SslConnector::builder(SslMethod::tls())?;
        let connector = MakeTlsConnector::new(builder.build());
        let uri = format!("{}", self.database);
        println!("uri is {}", uri);
        let client = Client::connect(&uri, connector)?;
        Ok(client)
    }
}

struct Query {
    query: String,
}

impl Query {
    fn new() -> Query {
        Query {
            query: String::from(""),
        }
    }
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
            .set_header(vec!["Header1"]);

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
    fn execute(&self, mut client: postgres::Client) {
        println!("Executing query: {}", self.query);
        let _res = client.batch_execute(&self.query).unwrap();
    }
}

fn initialize_env() -> NeonSession {
    let config = NeonSession {
        user: dotenv!("USER").to_string(),
        password: dotenv!("PASSWORD").to_string(),
        hostname: dotenv!("HOSTNAME").to_string(),
        database: dotenv!("DATABASE").to_string(),
        neon_api_key: dotenv!("NEON_API_KEY").to_string(),
    };
    config
}

fn build_uri(endpoint: String) -> String {
    format!("{}{}", NEON_BASE_URL.to_string(), endpoint)
}

fn handle_http_result(r: Result<String, Box<dyn Error>>) -> serde_json::Result<()> {
    match r {
        Ok(s) => {
            let json_blob: Value = serde_json::from_str(&s)?;
            let formatted = to_string_pretty(&json_blob);
            println!("{}", formatted.unwrap());
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
    Ok(())
}

#[tokio::main]
async fn perform_keys_action(action: &String, name: &String, neon_config: &NeonSession) {
    match action {
        s if s == "list" => {
            let uri = build_uri("/api_keys".to_string());
            let r = block_on(do_http_get(uri, &neon_config));
            let h: Result<(), serde_json::Error> = handle_http_result(r);
        }
        s if s == "create" => {
            let mut post_body: HashMap<String, String> = HashMap::new();
            post_body.insert("key_name".to_string(), name.to_string());
            let uri = build_uri("/api_keys".to_string());
            let r = block_on(do_http_post(uri, &post_body, &neon_config));
            let h: Result<(), serde_json::Error> = handle_http_result(r);
        }
        s if s == "revoke" => {
            println!("revoke key");
            let uri = build_uri(format!("/api_keys/{}", name.to_string()));
            let r = block_on(do_http_delete(uri, &neon_config));
            let h: Result<(), serde_json::Error> = handle_http_result(r);
        }
        _ => {
            println!("unknown action");
        }
    }
}

#[tokio::main]
async fn perform_projects_action(project: &String, neon_config: &NeonSession) {
    if project == "" {
        let uri = build_uri("/projects".to_string());
        let r = block_on(do_http_get(uri, neon_config));
        let h: Result<(), serde_json::Error> = handle_http_result(r);
    } else if project != "" {
        let endpoint: String = format!("{}{}", "/projects/".to_string(), project);
        let uri = build_uri(endpoint);
        let r = block_on(do_http_get(uri, neon_config));
        let h: Result<(), serde_json::Error> = handle_http_result(r);
    }
}

// tim@yoda neon-cli % target/debug/neon-cli branch -a list-roles -p white-voice-129396 -b br-dry-silence-599905
#[tokio::main]
async fn perform_branches_action(
    action: &String,
    project: &String,
    branch: &String,
    roles: &String,
    neon_config: &NeonSession,
) {
    if action == "list-roles" {
        let endpoint: String = format!("/projects/{}/branches/{}/roles", project, branch);
        let r = block_on(do_http_get(build_uri(endpoint), neon_config));
        let h: Result<(), serde_json::Error> = handle_http_result(r);
    } else if action == "list-endpoints" {
        let endpoint: String = format!("/projects/{}/branches/{}/endpoints", project, branch);
        let r = block_on(do_http_get(build_uri(endpoint), neon_config));
        let h: Result<(), serde_json::Error> = handle_http_result(r);
    } else if action == "list-branches" {
        let endpoint: String = format!("/projects/{}/branches", project);
        let r = block_on(do_http_get(build_uri(endpoint), neon_config));
        let h: Result<(), serde_json::Error> = handle_http_result(r);
    } else if action == "branch-details" {
        let endpoint: String = format!("/projects/{}/branches/{}", project, branch);
        let r = block_on(do_http_get(build_uri(endpoint), neon_config));
        let h: Result<(), serde_json::Error> = handle_http_result(r);
    } else if action == "list-databases" {
        let endpoint: String = format!("/projects/{}/branches/{}/databases", project, branch);
        let r = block_on(do_http_get(build_uri(endpoint), neon_config));
        let h: Result<(), serde_json::Error> = handle_http_result(r);
    } else if action == "database-details" {
        let endpoint: String = format!("/projects/{}/branches/{}/databases/{}", project, branch, neon_config.database);
        let r = block_on(do_http_get(build_uri(endpoint), neon_config));
        let h: Result<(), serde_json::Error> = handle_http_result(r);
    }
}

fn main() {
    let cli = Cli::parse();
    let subcommand = cli.action;
    dotenv().ok();

    let config = initialize_env();

    match subcommand {
        Action::Query { sql } => {
            println!("sql is {}", sql);
            let c = config.connect().expect("couldn't connect");
            let q: Query = Query { query: sql };
            q.query(c);
        }
        Action::Projects { project } => {
            let p = project.unwrap_or("".to_string()); // project id
            perform_projects_action(&p, &config)
        }
        Action::Keys { action, name } => {
            let name = name.unwrap_or("".to_string()); // name of the key to create
            perform_keys_action(&action, &name, &config);
        }
        Action::Branch {
            action,
            project,
            branch,
            roles,
        } => {
            let p = project.unwrap_or("".to_string());
            let b: String = branch.unwrap_or("".to_string());
            let r: String = roles.unwrap_or("".to_string());
            perform_branches_action(&action, &p, &b, &r, &config);
        }
    }
}
