# Key-value storage
This repository has implementation of [Practical Networking Application](https://github.com/pingcap/talent-plan/tree/master/rust) by [PingCap](https://github.com/pingcap)

## Roadmap
- [x] [Project 1](https://github.com/pingcap/talent-plan/tree/master/rust/projects/project-1)
- [x] [Project 2](https://github.com/pingcap/talent-plan/tree/master/rust/projects/project-2)
- [ ] [Project 3](https://github.com/pingcap/talent-plan/tree/master/rust/projects/project-3)
- [ ] [Project 4](https://github.com/pingcap/talent-plan/tree/master/rust/projects/project-4)
- [ ] [Project 5](https://github.com/pingcap/talent-plan/tree/master/rust/projects/project-5)

## Usage

```
USAGE:
    kvs [OPTIONS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --db <db>    path to database file [default: kvs.db]

SUBCOMMANDS:
    get      get key from storage
    help     Prints this message or the help of the given subcommand(s)
    rm       remove key-value pair from storage
    set      set key with given value
    shell    start KVS shell
```

## Example

### Command-line client
```
$ kvs set a 3
$ kvs set b 5
$ kvs get a
3
$ kvs set a 55
$ kvs get a
55
$ kvs get c
Key not found
```

### Shell
```
$ kvs shell
>>> get a
55
>>> set c 25
>>> get c
25
>>> exit
Bye!
```
