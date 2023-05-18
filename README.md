# neon-cli

Neon-CLI is a Rust Crate providing the binary for a command line interface to the [Neon Serverless Postgres database](https://neon.tech).

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

# Usage
Using the CLI is based on commands.  Use 'neon-cli' and pass it a command (query, projects, keys, etc) to direct aspect of Neon.  Each command takes a varying set of arguments.
```console
% neon-cli --help      
Does awesome things

Usage: neon-cli [OPTIONS] <COMMAND>

Commands:
  query        Execute a query
  projects     Get information about projects in Neon.
  keys         Get information about keys in Neon.
  branch       Get information about branches in Neon.
  endpoints    Get information about endpoints in Neon.
  operations   Get information about operations in Neon.
  consumption  Get information about endpoints in Neon.
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

# Configuration (.env file)
Neon-cli uses a dotenv style setup and consumes the typical .env file you're used to.  Here is an example outlining all of the env vars neon-cli picks up:
```console
HOSTNAME=a.b.com
PORT=1234
USER=tim
PASSWORD=asdf
DATABASE=neondb
CONNECT_STRING=postgres://tim:ddddddddddd@ep-foo-bar-1111111.us-west-2.aws.neon.tech:5432/neondb
NEON_API_KEY=abcdefghijklmnopkrstuvwxyz123456789987654321
```
