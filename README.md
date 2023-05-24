# neon-cli

Neon-CLI is a Rust Crate providing the binary for a command line interface to the [Neon Serverless Postgres database](https://neon.tech).

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

# Usage
Using the CLI is based on commands.  Use 'neon-cli' and pass it a command (query, projects, keys, etc) to direct aspect of Neon.  Each command takes a varying set of arguments.
```console
% neon-cli 
Neon Postgres Database CLI

Usage: neon-cli [OPTIONS] <COMMAND>

Commands:
  query        Execute a query
  projects     Get information about projects in Neon.
  keys         Get information about keys in Neon.
  branch       Get information about branches in Neon.
  endpoints    Get information about endpoints in Neon.
  operations   Get information about operations in Neon.
  consumption  Get information about consumption in Neon.
  import       Import data from csv file (TEXT only for now).
  help         Print this message or the help of the given subcommand(s)

Options:
  -d, --debug...  
  -h, --help      Print help
```

Here is an example of the arguments for a given command (query, in this case using --sql or -s to pass a sql statement in.):
```console
% neon-cli query --help
Execute a query

Usage: neon-cli query --sql <SQL>

Options:
  -s, --sql <SQL>  
  -h, --help       Print help
```

Here is an example of actually using an argument for a given command.  Note that "--sql=...." can also be used as -s "select * from ...."

```console
% neon-cli query --sql="select * from foo limit 2;"
Executing query: select * from foo limit 2;
╭──────┬───────┬────────╮
│ bar  ┆ baz   ┆ counts │
╞══════╪═══════╪════════╡
│ test ┆ test2 ┆ 42     │
├╌╌╌╌╌╌┼╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┤
│ test ┆ test2 ┆ 42     │
╰──────┴───────┴────────╯
``` 

# Importing Data
To import a CSV file, use:
```console

% neon-cli import --help
Import data from csv file (TEXT only for now).

Usage: neon-cli import [OPTIONS] --table <TABLE> --file <FILE>

Options:
  -t, --table <TABLE>          
  -f, --file <FILE>            
  -d, --delimiter <DELIMITER>  
  -h, --help                   Print help

% cat foo.csv
col1,col2,col3
it,works,100
this,is,42

% neon-cli import -f foo.csv -t foo
```

# Output Format
Most of the commands have a -f option for outputting the raw JSON from NeonDB or as a table, using "-f table" or "--format=table" or "--format=json", for example:
```console
% neon-cli keys -a list -f table
╭────────────────────────┬────────┬────────────────────────┬─────────────────────┬────────────╮
│ created_at             ┆ id     ┆ last_used_at           ┆ last_used_from_addr ┆ name       │
╞════════════════════════╪════════╪════════════════════════╪═════════════════════╪════════════╡
│ "2023-05-17T18:19:16Z" ┆ 382774 ┆ null                   ┆ ""                  ┆ "testkey2" │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┤
│ "2023-05-17T04:22:00Z" ┆ 380896 ┆ "2023-05-24T18:45:54Z" ┆ "98.116.56.186"     ┆ "test"     │
╰────────────────────────┴────────┴────────────────────────┴─────────────────────┴────────────╯
```

# Configuration (.env file)
Neon-cli uses a dotenv style setup and consumes the typical .env file you're used to.  Here is an example outlining all of the env vars neon-cli picks up.  If CONNECT_STRING exists, that will be used.  Otherwise CONNECT_STRING is built out of HOSTNAME, PORT, etc:
```console
HOSTNAME=a.b.com
PORT=1234
USER=tim
PASSWORD=asdf
DATABASE=neondb
CONNECT_STRING=postgres://tim:ddddddddddd@ep-foo-bar-1111111.us-west-2.aws.neon.tech:5432/neondb
NEON_API_KEY=abcdefghijklmnopkrstuvwxyz123456789987654321
```

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/tullytim/neon-cli/blob/master/LICENSE

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Tokio by you, shall be licensed as MIT, without any additional
terms or conditions.
