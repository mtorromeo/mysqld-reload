# MySQL configuration reloading utility

![Status: Alpha](https://img.shields.io/badge/status-alpha-red.svg?longCache=true "Status: Alpha")

## Install

Clone the repository or download a zip/tar from github.

Run `make` in the source directory, and then `cargo build --release`.

You'll find the binary in `target/release/mysqld-reload`.

## Usage

```
mysqld-reload 0.2.0
Massimiliano Torromeo <massimiliano.torromeo@gmail.com>
Reads the MySQL configuration files and applies the configured variables at runtime when their values don't match.

USAGE:
    mysqld-reload [FLAGS] [OPTIONS]

FLAGS:
    -d, --dry-run        Do not apply values
    -h, --help           Prints help information
        --no-defaults    Don't read default options from any option file, except for login file
    -V, --version        Prints version information
    -v, --verbose        Print the SQL statements to stdout

OPTIONS:
    -c, --cnf <cnf>                        MySQL server configuration file [default: /etc/my.cnf]
        --defaults-file <defaults-file>    Only read default options from the given file
    -H, --host <host>                      Connect to host
    -p, --password <password>              Password to use when connecting to server
    -P, --port <port>                      Port number to use for connection
    -S, --socket <socket>                  The socket file to use for connection
    -u, --user <user>                      User for login if not current user
```

## Known issues

`!include`/`!includedir` directive in the mysql configuration files are currently ignored. You can workaround it by running the command against all included files with `mysqld-reload -c CONFIG.cnf`.

## Contributing

Please see [CONTRIBUTING](CONTRIBUTING.md) and [CONDUCT](CONDUCT.md) for details.

## Security

If you discover any security related issues, please email massimiliano.torromeo@gmail.com instead of using the issue tracker.

## Credits

- [Massimiliano Torromeo][link-author]
- [All Contributors][link-contributors]

## License

This software is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[link-author]: https://github.com/mtorromeo
[link-contributors]: https://github.com/mtorromeo/mysqld-reload/graphs/contributors
