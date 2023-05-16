#![allow(dead_code)]
#![allow(unused_imports)]

use clap::Parser;
use serde::Deserialize;
use openssl::ssl::{SslConnector, SslMethod};
use postgres::Client;
use postgres_openssl::MakeTlsConnector;
use dotenv::dotenv;
use std::{error::Error, io};
use comfy_table::*;
use comfy_table::{presets::UTF8_FULL, modifiers::UTF8_ROUND_CORNERS};
mod neonutils;
use crate::neonutils::reflective_get;

#[macro_use]
extern crate dotenv_codegen;

const NEON_BASE_URL: &str = "https://console.neon.tech/api/v2/";

#[derive(Parser)]
#[command(author = "Tim Tully. <tim@menlovc.com>")]
#[command(about = "Does awesome things", long_about = None)]
struct Cli {
    command: Option<String>,
    #[arg(short, long)]
    query: Option<String>,
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
        let column_headers:Vec<String> = Vec::new();
        let res = client.query(&self.query, &[]).unwrap();

      


        let mut table = Table::new();
        table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Header1", "Header2", "Header3"])
        .add_row(vec![
                 "This is a text",
                 "This is another text",
                 "This is the third text",
        ])
        .add_row(vec![
                 "This is another text",
                 "Now\nadd some\nmulti line stuff",
                 "This is awesome",
        ]);
        println!("{table}");

        for row in &res {

            if !saw_first_row {
                row.columns().iter().for_each(|c| {
                    println!("column: {:?}", c.name());
                });
                saw_first_row = true;
            }

            println!("row: {:?}", row  );
            let s1:&str = &reflective_get(row, 0);
            println!("row: {:?}", s1);
            /* 
            let id: i32 = row.get(0);
            let name: &str = row.get(1);
            let data: Option<&[u8]> = row.get(2);
            println!("found person: {} {} {:?}", id, name, data);
            */
        }
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
    let query = cli.query.unwrap();
    println!("Got Query: {}", query);
    dotenv().ok();

    let config = initialize_env();

    let  c = config.connect().expect("couldn't connect");
    let  q: Query = Query { query: query };
    q.query(c);

    Ok(())
}