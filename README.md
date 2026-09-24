# fixture-parser

A small Rust library for parsing sports fixture lists - the kind of plain
text schedule you'd copy out of a league website or a club newsletter -
into structured data you can actually work with.

No third-party dependencies. Standard library only.

## The problem

Fixture lists show up as text: a date, two team names, sometimes a
competition. They get passed around as files, pasted into terminals,
piped between tools. A parser that only accepts `File` handles is
awkward the moment you want to do:

```sh
cat fixtures.txt | your-tool
```

or read from a file directly, without writing two versions of the same
logic. This library has one parsing path (`from_reader`, generic over
`std::io::Read`) that `from_path` and `from_stdin` both call into, so a
file and a stdin pipe are handled identically.

## Format

One fixture per line, in either of two formats. Which one applies is
decided per line, by whether it contains a `|`, so a file can mix both.

Pipe-separated:

```
DATE | HOME v AWAY | COMPETITION
```

CSV:

```
DATE,HOME,AWAY,COMPETITION
```

A CSV team name containing a comma can be quoted (`"Bath, City"`); a
doubled quote (`""`) inside a quoted field is an escaped quote.

`COMPETITION` is optional in both formats. Blank lines and lines starting
with `#` are ignored.

```
# week 4
2026-09-19 | Arsenal v Chelsea | Premier League
2026-09-20 | Celtic v Rangers | Scottish Premiership
2026-09-20 | Bath v Leicester
2026-09-21,Newcastle,Everton,Premier League
2026-09-21,Hearts,Hibernian
```

## Usage

Reading a file:

```rust
let fixtures = fixture_parser::from_path("fixtures.txt")?;
for f in &fixtures {
    println!("{}: {} v {}", f.date, f.home_team, f.away_team);
}
```

Reading from stdin:

```rust
let fixtures = fixture_parser::from_stdin()?;
```

Reading from any other source (useful in tests):

```rust
let data: &[u8] = b"2026-09-19 | Arsenal v Chelsea\n";
let fixtures = fixture_parser::from_reader(data)?;
```

All three return `Result<Vec<Fixture>, FixtureError>`, where `Fixture` is:

```rust
pub struct Fixture {
    pub date: Date,
    pub home_team: String,
    pub away_team: String,
    pub competition: Option<String>,
}
```

## Status

Early skeleton. Dates are validated against a real calendar (leap years,
days-per-month), and both the pipe-delimited and CSV formats above are
supported - see the roadmap in the commit history for what's planned next.
