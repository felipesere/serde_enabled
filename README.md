`Enable<T>` is a wrapper that adds one extra field `enabled` when the `T`
is serialized with `Serde`. That field can be `true`, in which case all
of the fields for `T` need to be present, or it can be `false` at which point
all fields of `T` can be ommited.

The use case is for configuration YAMLs where sections can be toggled on or off.

```
use serde::{Deserialize, Serialize};
use serde_enabled::Enable;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
struct Outside {
    inside: Enable<Inside>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
struct Inside {
    thing: u32,
    other: String,
}

let raw = indoc::indoc! {r#"
    inside:
        enable: false
        thing: 1
        other: "Great"
    "#};

let o: Outside = serde_yaml::from_str(raw).unwrap();

 assert!(!o.inside.is_enabled());

 let raw = indoc::indoc! {r#"
     inside:
         enable: true
         thing: 1
         other: "Great"
     "#};

 let o: Outside = serde_yaml::from_str(raw).unwrap();

  assert!(o.inside.is_enabled());
  assert_eq!(
        o,
        Outside {
           inside: Enable::On(Inside {
            thing: 1,
            other: "Great".into()
        })
        }
    );
```
