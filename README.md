# Takion
An experimental type level parser combinator library in rust. *Alternatively, a "combinatory parser" perchance.*

## Features
- String Parsing
  - [x] well-founded iteration
  - [ ] well-founded recursion
  - [ ] term-level parsers
- All parsers get error diagnostics pretty printed with grammar prints
  - [ ] custom/preset formats
- fuzzing/enumeration for parsers *(removed for now)

## Demo
```rust
use takion::{*, syntax::Ident};

type Generic = UseAll<(Ident, Option<Commit<WsTok<":">, Intersperse<Ident, WsTok<"+">>>>)>;

assert_eq!(
    Generic::parse::<Traced>("T: Clone + Sized".into()).must(),
    ("T", Some(vec!["Clone", "Sized"])),
);
```

```rust
use takion::*;

struct Boolean;
parser!{[Boolean] -> [bool] {
    ("true") => true;
    ("false") => false;
}}

assert!(Boolean::parse::<()>("true".into()).must());
assert!(!Boolean::parse::<()>("false".into()).must());
assert!(Boolean::parse::<()>("trv".into()).is_miss());
```

`bool`, as well as other rust primitives, already implement `Parse`.

## Motivation
"too complex for `regex`" .. ??? .. "too simple for `nom`/`chumsky` proper". `???` is where `takion` is built for.

## Stability
`takion` currently depeds on nightly and uses a number of features. It is between "stable" unstable, and "debugged on zulip" unstable. Currently the former, used to be the latter. None of the feature dependencies are "strictly necessary", but they do make life easier.

**Primitives Depend on**
- `Repeat`|`Hex`|`Tok`: `adt_const_params`
- `Tok`: `unsized_const_params`
**User Facing Ergonomics**
- `try_trait_v2`, `try_trait_v2_residual`
**Internal Ergonomics**
- `associated_type_defaults`
- `decl_macro`

## MSRV
`rustc 1.100.0-nightly (0ed41eb41 2026-09-04)`

*This will lower over time as I get around to testing older versions. Presumably works all the way to, idk, 1.96 at least.*

## License
`takion` is licensed under MPL 2.0.
