#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use clap::Parser;
use comfy_table::*;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL};
use dotenv::dotenv;
use openssl::ssl::{SslConnector, SslMethod};
use postgres::Client;
use postgres_openssl::MakeTlsConnector;
use serde::Deserialize;
use std::{error::Error, io};
use futures::executor::block_on;
use serde_json::to_string_pretty;
use serde_json::{Value};
mod neonutils;
mod networking;
use crate::neonutils::reflective_get;
use crate::networking::do_http_get;

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
    Keys {
        #[clap(short, long)]
        action: String,
    },
    #[clap(about = "Get information about branches in Neon.")]
    Branch {
        #[clap(short, long)]
        project: Option<String>,
        #[clap(short, long)]
        branch: Option<String>,
    },
}

#[derive(Deserialize, Debug)]
pub struct NeonSession {
    user: String,
    password: String,
    hostname: String,
    database: String,
    neon_api_key:String,
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

fn build_uri(endpoint:String) -> String {
    format!("{}{}", NEON_BASE_URL.to_string(), endpoint)
}

fn handle_http_result(r: Result<String, Box<dyn Error>>) -> serde_json::Result<()>{
    match r {
        Ok(s) => {
            
            let json_blob:Value = serde_json::from_str(&s)?;
            let formatted = to_string_pretty(&json_blob);
            println!("{}", formatted.unwrap());
            
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
    Ok(())
}

fn perform_keys_action(action: &String, neon_config: &NeonSession) {
    match action {
        s if s == "list" => {
            let uri = build_uri("/api_keys".to_string());
            println!("uri is {}", uri);
            block_on(do_http_get(uri, neon_config));
        }
        s if s == "create" => {
            println!("create key");
        }
        s if s == "delete" => {
            println!("delete key");
        }
        _ => {
            println!("unknown action");
        }
    }
}

fn perform_projects_action(project: &String, neon_config: &NeonSession){
    if project == "" {
        let uri = build_uri("/projects".to_string());
        let r = block_on(do_http_get(uri, neon_config));
        let h: Result<(), serde_json::Error> = handle_http_result(r);
    }
    else if project != "" {
        let endpoint:String = format!("{}{}", "/projects/".to_string(), project);
        let uri = build_uri(endpoint);
        let r = block_on(do_http_get(uri, neon_config));
        let h: Result<(), serde_json::Error> = handle_http_result(r);
    }
}

fn perform_branches_action(project: &String, branch: &String, neon_config: &NeonSession) {
    if branch == "" {
        let endpoint:String = format!("{}{}/branches", "/projects/".to_string(), project);
        let uri = build_uri(endpoint);
        let r = block_on(do_http_get(uri, neon_config));
        let h: Result<(), serde_json::Error> = handle_http_result(r);
    }
    else if branch != ""{
        let endpoint:String = format!("{}{}/branches/{}", "/projects/".to_string(), project, branch);
        let uri = build_uri(endpoint);
        println!("URI: {}", uri);
        let r = block_on(do_http_get(uri, neon_config));
        let h: Result<(), serde_json::Error> = handle_http_result(r);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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
        },
        Action::Projects { project } => {
            let p = project.unwrap_or("".to_string());
            perform_projects_action(&p, &config)
        },
        Action::Keys { action } => {
            perform_keys_action(&action, &config);
        },
        Action::Branch { project, branch } => {
            let p = project.unwrap_or("".to_string());
            let b: String = branch.unwrap_or("".to_string());
            perform_branches_action(&p, &b, &config);
        }
    }

    Ok(())
}
