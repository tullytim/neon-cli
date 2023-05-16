#![allow(dead_code)]
#![allow(unused_imports)]

use clap::Parser;
use comfy_table::*;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL};
use dotenv::dotenv;
use openssl::ssl::{SslConnector, SslMethod};
use postgres::Client;
use postgres_openssl::MakeTlsConnector;
use serde::Deserialize;
use std::{error::Error, io};
mod neonutils;
use crate::neonutils::reflective_get;

#[macro_use]
extern crate dotenv_codegen;

const NEON_BASE_URL: &str = "https://console.neon.tech/api/v2/";

#[derive(Parser)]
#[command(author = "Tim Tully. <tim@menlovc.com>")]
#[command(about = "Does awesome things")]
struct Cli {
    #[clap(subcommand)]
    action: Action,
    #[clap(short, long)]
    sql: Option<String>,
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
}

#[derive(Deserialize, Debug)]
struct NeonSession {
    user: String,
    password: String,
    hostname: String,
    database: String,
}

impl NeonSession {
    fn new() -> NeonSession {
        NeonSession {
            user: String::from(""),
            password: String::from(""),
            hostname: String::from(""),
            database: String::from(""),
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
        let column_headers: Vec<String> = Vec::new();
        let res = client.query(&self.query, &[]).unwrap();
        let num_rows: u64 = res.len() as u64;
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
    };
    config
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    //let command: String = cli.command.unwrap();
    let subcommand = cli.action;
    println!("My enum value: {:?}", subcommand);

    dotenv().ok();

    let config = initialize_env();

    match subcommand {
        Action::Query { sql } => {
            println!("sql is {}", sql);
            let c = config.connect().expect("couldn't connect");
            let q: Query = Query { query: sql };
            q.query(c);
        }
        _ => println!("something else!"),
    }

    Ok(())
}
