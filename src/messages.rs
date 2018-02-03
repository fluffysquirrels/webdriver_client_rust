//! Messages sent and received in the WebDriver protocol.

#![allow(non_snake_case)]

use ::util::merge_json_mut;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{Visitor, MapAccess};
use serde::de::Error as DeError;
use serde::ser::SerializeStruct;
use serde_json::Value as JsonValue;
use std::fmt;
use std::collections::BTreeMap;

#[derive(Debug)]
pub enum LocationStrategy {
    Css,
    LinkText,
    PartialLinkText,
    XPath,
}

impl Serialize for LocationStrategy {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            &LocationStrategy::Css => s.serialize_str("css selector"),
            &LocationStrategy::LinkText => s.serialize_str("link text"),
            &LocationStrategy::PartialLinkText => s.serialize_str("partial link text"),
            &LocationStrategy::XPath => s.serialize_str("xpath"),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct WebDriverError {
    pub error: String,
    pub message: String,
    pub stacktrace: Option<String>,
}

#[derive(Serialize, Default)]
struct Capabilities {
    alwaysMatch: JsonValue,
}

/// The arguments to create a new session, including the capabilities
/// as defined by the [WebDriver specification][cap-spec].
///
/// [cap-spec]: https://www.w3.org/TR/webdriver/#capabilities
#[derive(Serialize)]
pub struct NewSessionCmd {
    capabilities: Capabilities,
}

impl NewSessionCmd {
    /// Merges a new `alwaysMatch` capability with the given `key` and
    /// `value` into the new session's capabilities.
    ///
    /// For the merging rules, see the documentation of
    /// `webdriver_client::util::merge_json`.
    pub fn always_match(&mut self, key: &str, value: JsonValue) -> &mut Self {
        merge_json_mut(&mut self.capabilities.alwaysMatch,
                       &json!({ key: value }));
        self
    }

    /// Resets the `alwaysMatch` capabilities to an empty JSON object.
    pub fn reset_always_match(&mut self) -> &mut Self {
        self.capabilities.alwaysMatch = json!({});
        self
    }
}

impl Default for NewSessionCmd {
    fn default() -> Self {
        NewSessionCmd {
            capabilities: Capabilities {
                alwaysMatch: json!({

                    // By default chromedriver is NOT compliant with the w3c
                    // spec. But one can request w3c compliance with a capability
                    // extension included in the new session command payload.
                    "goog:chromeOptions": {
                        "w3c": true
                    }
                })
            },
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Session {
    pub sessionId: String,
    pub capabilities: BTreeMap<String, JsonValue>,
}

#[derive(Serialize)]
pub struct GoCmd {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Value<T> {
    pub value: T,
}

#[derive(Debug, Deserialize)]
pub struct CurrentTitle {
    pub title: String,
}

#[derive(Serialize)]
pub struct SwitchFrameCmd {
    pub id: JsonValue,
}

impl SwitchFrameCmd {
    pub fn from(id: JsonValue) -> Self {
        SwitchFrameCmd { id: id }
    }
}

#[derive(Serialize)]
pub struct SwitchWindowCmd {
    handle: String,
}

impl SwitchWindowCmd {
    pub fn from(handle: &str) -> Self {
        SwitchWindowCmd { handle: handle.to_string() }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Empty {}

#[derive(Serialize)]
pub struct FindElementCmd<'a> {
    pub using: LocationStrategy,
    pub value: &'a str,
}

#[derive(PartialEq, Debug)]
pub struct ElementReference {
    pub reference: String,
}

impl ElementReference {
    pub fn from_str(handle: &str) -> ElementReference {
        ElementReference { reference: handle.to_string() }
    }
}

impl Serialize for ElementReference {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut ss = s.serialize_struct("ElementReference", 1)?;
        ss.serialize_field("element-6066-11e4-a52e-4f735466cecf", &self.reference)?;
        // even in w3c compliance mode chromedriver only accepts a reference name ELEMENT
        ss.serialize_field("ELEMENT", &self.reference)?;
        ss.end()
    }
}

impl<'de> Deserialize<'de> for ElementReference {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        enum Field { Reference };

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                struct FieldVisitor;
                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;
                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("element-6066-11e4-a52e-4f735466cecf")
                    }

                    fn visit_str<E: DeError>(self, value: &str) -> Result<Field, E>
                    {
                        match value {
                            "element-6066-11e4-a52e-4f735466cecf" => Ok(Field::Reference),
                            _ => Err(DeError::unknown_field(value, FIELDS)),
                        }
                    }
                }

                d.deserialize_identifier(FieldVisitor)
            }
        }

        struct ElementReferenceVisitor;
        impl<'de> Visitor<'de> for ElementReferenceVisitor {
            type Value = ElementReference;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct ElementReference")
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<ElementReference, V::Error>
                where V: MapAccess<'de>
            {
                let mut reference = None;
                while let Some(key) = visitor.next_key()? {
                    match key {
                        Field::Reference => {
                            if reference.is_some() {
                                return Err(DeError::duplicate_field("element-6066-11e4-a52e-4f735466cecf"));
                            }
                            reference = Some(visitor.next_value()?);
                        }
                    }
                }
                match reference {
                    Some(r) => Ok(ElementReference { reference: r }),
                    None => return Err(DeError::missing_field("element-6066-11e4-a52e-4f735466cecf")),
                }
            }
        }

        const FIELDS: &'static [&'static str] = &["element-6066-11e4-a52e-4f735466cecf"];
        d.deserialize_struct("ElementReference", FIELDS, ElementReferenceVisitor)
    }
}

#[derive(Debug, Deserialize)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub path: String,
    pub domain: String,
    pub secure: bool,
    pub httpOnly: bool,
    // TODO: expiry:
}

#[derive(Serialize)]
pub struct ExecuteCmd {
    pub script: String,
    pub args: Vec<JsonValue>,
}

#[cfg(test)]
mod tests {
    use super::NewSessionCmd;
    #[test]
    fn capability_extend() {
        let mut session = NewSessionCmd::default();
        session.always_match("cap", Some(json!({"a": true})));
        assert_eq!(session.capabilities.alwaysMatch.get("cap").unwrap(), &json!({"a": true}));

        session.extend_always_match("cap", json!({"b": false}));
        assert_eq!(session.capabilities.alwaysMatch.get("cap").unwrap(), &json!({"a": true, "b": false}));

        session.extend_always_match("cap", json!({"a": false}));
        assert_eq!(session.capabilities.alwaysMatch.get("cap").unwrap(), &json!({"a": false, "b": false}));
    }
    #[test]
    fn capability_extend_replaces_non_obj() {
        let mut session = NewSessionCmd::default();
        session.always_match("cap", Some(json!("value")));
        assert_eq!(session.capabilities.alwaysMatch.get("cap").unwrap(), &json!("value"));

        session.extend_always_match("cap", json!({"a": false}));
        assert_eq!(session.capabilities.alwaysMatch.get("cap").unwrap(), &json!({"a": false}));
    }
    #[test]
    fn capability_extend_replaces_obj_with_non_obj() {
        let mut session = NewSessionCmd::default();
        session.always_match("cap", Some(json!({"value": true})))
            .extend_always_match("cap", json!("new"));
        assert_eq!(session.capabilities.alwaysMatch.get("cap").unwrap(), &json!("new"));
    }
}
