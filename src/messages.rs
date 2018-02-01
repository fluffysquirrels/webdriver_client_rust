#![allow(non_snake_case)]

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
    alwaysMatch: BTreeMap<String, JsonValue>,
}

#[derive(Serialize)]
pub struct NewSessionCmd {
    capabilities: Capabilities,
}

impl NewSessionCmd {
    /// Adds a required capability. If the capability was already set, it is replaced.
    pub fn always_match(&mut self, name: &str, capability: Option<JsonValue>) -> &mut Self {
        match capability {
            Some(value) => self.capabilities.alwaysMatch.insert(name.to_string(), value),
            None => self.capabilities.alwaysMatch.remove(name),
        };
        self
    }

    /// Extend a capability requirement with a new object.
    ///
    /// Extending a capability that does not exist, or attempting to extend non objects will have
    /// the same effect as calling [always_match()](#method.always_match).
    pub fn extend_always_match(&mut self, name: &str, capability: JsonValue) {
        if let JsonValue::Object(capability) = capability {
            if let Some(&mut JsonValue::Object(ref mut map)) = self.capabilities.alwaysMatch.get_mut(name) {
                map.extend(capability);
                return;
            }
            self.capabilities.alwaysMatch.insert(name.to_string(), JsonValue::Object(capability));
        } else {
            self.capabilities.alwaysMatch.insert(name.to_string(), capability);
        }
    }
}

impl Default for NewSessionCmd {
    fn default() -> Self {
        let mut capabilities: Capabilities = Default::default();
        capabilities.alwaysMatch.insert("goog:chromeOptions".to_string(), json!({"w3c": true}));
        NewSessionCmd {
            capabilities,
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
