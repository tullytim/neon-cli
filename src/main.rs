use clap::Parser;
use serde::Deserialize;

use openssl::ssl::{SslConnector, SslMethod};
use postgres::Client;
use postgres_openssl::MakeTlsConnector;

extern crate dotenv;
use dotenv::dotenv;

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
        let mut client = Client::connect(&uri, connector)?;
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

    fn execute(&self, mut client: postgres::Client) {
        println!("Executing query: {}", self.query);
        let res = client.batch_execute(&self.query).unwrap();
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

fn main() {
    let cli = Cli::parse();
    let query = cli.query.unwrap();
    println!("Got Query: {}", query);
    dotenv().ok();

    let config = initialize_env();

    let mut c = config.connect().expect("couldn't connect");
    let mut q: Query = Query { query: query };
    q.execute(c)


    /*
        c
        .batch_execute(
            "
            CREATE TABLE IF NOT EXISTS person (
                id              SERIAL PRIMARY KEY,
                name            TEXT NOT NULL,
                data            BYTEA
            )
            ",
        )
        .unwrap();
    */
}
/*
fn main() {
    // basic app information
    let app = App::new("hello-clap")
        .version("1.0")
        .about("Says hello")
        .author("Michael Snoyman");

    println!("Hello, world!");
    println!("hello again")
}*/

/* let API_KEY  = env::var("NEON_API_KEY").unwrap();
   let c = envy::from_env::<NeonConfig>()
       .expect("Failed to read config from environment");
*/

/*
println!("con_string: {}", con_string);

        let client = Connection::connect(
            "postgres://postgres@localhost:5432",
            TlsMode::Require(&negotiator),
        ).unwrap();

        //let mut client = Client::connect(&con_string.as_str(), NoTls);


        client
            .batch_execute(
                "
                CREATE TABLE IF NOT EXISTS person (
                    id              SERIAL PRIMARY KEY,
                    name            TEXT NOT NULL,
                    data            BYTEA
                )
                ",
            )
            .unwrap(); */

/*
let cert = fs::read("./neon.tech.cer")?;
      let cert = Certificate::from_pem(&cert)?;
      let connector = TlsConnector::builder()
          .add_root_certificate(cert)
          .build()?;
      let connector = MakeTlsConnector::new(connector);
      let uri = format!("host={}?options=project%3Dep-autumn-sun-308519 user=tim sslmode=require", self.database);

      println!("host is {}", uri);

      let client = postgres::Client::connect(
          "host={}} user=postgres sslmode=require",
          connector,
      )?;
      Ok(client) */
