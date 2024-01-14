use serde::{de::Visitor, Deserialize, Serialize};

///
/// `Enable<T>` is a wrapper to properly `Serialize` and `Deserialize`
/// settings that can be turned `On` or `Off`.
/// Particularly, in the `Off` variant the aditional fields are not required.
#[allow(private_interfaces)]
#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum Enable<T> {
    On(On<T>),
    Off { enable: False },
}

impl<T> Enable<T> {
    pub fn into_inner(self) -> Option<T> {
        match self {
            Enable::On(On { inner, .. }) => Some(inner),
            Enable::Off { .. } => None,
        }
    }

    pub fn as_ref(&self) -> Option<&T> {
        match self {
            Enable::On(On { inner, .. }) => Some(inner),
            Enable::Off { .. } => None,
        }
    }

    pub fn is_enabled(&self) -> bool {
        matches!(self, Enable::On(_))
    }

    pub fn off() -> Enable<T> {
        Self::Off { enable: False }
    }

    pub fn on(inner: T) -> Enable<T> {
        Self::On(On {
            enable: True,
            inner,
        })
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

struct BoolVistor;

impl Visitor<'_> for BoolVistor {
    type Value = bool;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a bool")
    }

    fn visit_bool<E>(self, v: bool) -> std::prelude::v1::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v)
    }
}

impl<'de> Deserialize<'de> for True {
    fn deserialize<D>(deserializer: D) -> std::prelude::v1::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let val = deserializer.deserialize_bool(BoolVistor)?;
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
        let val = deserializer.deserialize_bool(BoolVistor)?;
        if !val {
            Ok(False)
        } else {
            Err(serde::de::Error::custom("Expected a false value"))
        }
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use crate::Enable;

    #[derive(Deserialize)]
    struct Outside {
        inside: Enable<Inside>,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
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
}
