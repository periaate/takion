- Sort/clean this file into proper categories

## interfaces
- [ ] Cursor -> trait
  - To enable more kinds of inputs to be parsed
- [ ] Parse -> { Parses + Parse }
  - Similar to `Iterator` + `IntoIterator` in essence; makes owned inputs usable
- [ ] Map/Pipe overlap semantically, consider if one can be derived
- [ ] Prog/Parse | Func/Parser overlap

## formatting/diagnostics
- [ ] Format markers and Format trait
  - [ ] PEG style
  - [ ] regex style
  - [ ] declarative macro style (currently assumed implicitly)

## statics
- [ ] benchmarks
  - determine which; should provide comparability and good coverage for regressions
- [ ] table test macros


## primitives
- [ ] `char`
- [ ] More thorough `Unescape` primitives ()

