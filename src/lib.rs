use serde::{Deserialize, Serialize};

/// `Enable<T>` is a wrapper to properly `Serialize` and `Deserialize`
/// settings that can be turned `On` or `Off`.
/// Particularly, in the `Off` variant the aditional fields are not required.
#[derive(Clone, Debug)]
pub enum Enable<T> {
    On(T),
    Off,
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
enum InnerEnable<T> {
    On(On<T>),
    #[allow(dead_code)]
    Off {
        enable: False,
    },
}

impl<'de, T: Deserialize<'de> + std::fmt::Debug> Deserialize<'de> for Enable<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        InnerEnable::<T>::deserialize(deserializer).map(|enabled| match enabled {
            InnerEnable::On(On { inner, .. }) => Enable::On(inner),
            InnerEnable::Off { .. } => Enable::Off,
        })
    }
}

impl<T: Serialize> Serialize for Enable<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let inner = match self {
            Enable::On(inner) => InnerEnable::On(On {
                enable: True,
                inner,
            }),
            Enable::Off => InnerEnable::Off { enable: False },
        };

        inner.serialize(serializer)
    }
}

impl<T> Enable<T> {
    pub fn into_inner(self) -> Option<T> {
        match self {
            Enable::On(inner) => Some(inner),
            Enable::Off => None,
        }
    }

    pub fn as_ref(&self) -> Option<&T> {
        match self {
            Enable::On(inner) => Some(inner),
            Enable::Off => None,
        }
    }

    pub fn is_enabled(&self) -> bool {
        matches!(self, Enable::On(_))
    }

    pub fn off() -> Enable<T> {
        Self::Off
    }

    pub fn on(inner: T) -> Enable<T> {
        Self::On(inner)
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
struct On<T> {
    enable: True,
    #[serde(flatten)]
    inner: T,
}

#[derive(Clone, Debug)]
struct True;

impl Serialize for True {
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bool(true)
    }
}

impl<'de> Deserialize<'de> for True {
    fn deserialize<D>(deserializer: D) -> std::prelude::v1::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let val = bool::deserialize(deserializer)?;
        if val {
            Ok(True)
        } else {
            Err(serde::de::Error::custom("Expected a true value"))
        }
    }
}

#[derive(Debug, Clone)]
struct False;

impl Serialize for False {
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bool(false)
    }
}

impl<'de> Deserialize<'de> for False {
    fn deserialize<D>(deserializer: D) -> std::prelude::v1::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let val = bool::deserialize(deserializer)?;
        if !val {
            Ok(False)
        } else {
            Err(serde::de::Error::custom("Expected a false value"))
        }
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use crate::Enable;

    #[derive(Deserialize, Serialize)]
    struct Outside {
        inside: Enable<Inside>,
    }

    #[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
    struct Inside {
        thing: u32,
        other: String,
    }

    #[test]
    fn extra_fields_not_needed_when_struct_disabled() {
        let raw = indoc::indoc! {r#"
            inside:
                enable: false
            "#};

        let o: Outside = serde_yaml::from_str(raw).unwrap();

        assert!(!o.inside.is_enabled());
    }

    #[test]
    fn extra_fields_allowed_when_struct_disabled() {
        let raw = indoc::indoc! {r#"
            inside:
                enable: false
                thing: 1
                other: "Great"
            "#};

        let o: Outside = serde_yaml::from_str(raw).unwrap();

        assert!(!o.inside.is_enabled());
    }

    #[test]
    fn extra_fields_needed_when_struct_enabled() {
        let raw = indoc::indoc! {r#"
            inside:
                enable: true
                thing: 1
                other: "Great"
            "#};

        let o: Outside = serde_yaml::from_str(raw).unwrap();

        assert!(o.inside.is_enabled());
        assert_eq!(
            o.inside.into_inner(),
            Some(Inside {
                thing: 1,
                other: "Great".into()
            })
        );
    }

    #[test]
    fn serialize_an_enabled_feature() {
        let o = Outside {
            inside: Enable::On(Inside {
                thing: 1,
                other: "Great".into(),
            }),
        };
        let raw = indoc::indoc! {r#"
            inside:
              enable: true
              thing: 1
              other: Great
            "#};

        let result = serde_yaml::to_string(&o).unwrap();
        assert_eq!(result, raw,);
    }

    #[test]
    fn serialize_a_disabled_feature() {
        let o = Outside {
            inside: Enable::Off,
        };
        let raw = indoc::indoc! {r#"
            inside:
              enable: false
            "#};

        let result = serde_yaml::to_string(&o).unwrap();
        assert_eq!(result, raw,);
    }
}
