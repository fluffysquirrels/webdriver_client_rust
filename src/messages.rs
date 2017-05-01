#![allow(non_snake_case)]

use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{Visitor, MapAccess};
use serde::de::Error as DeError;
use serde::ser::SerializeStruct;
use serde_json::Value as JsonValue;
use std::fmt;

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

#[derive(Deserialize, Debug)]
pub struct WebDriverError {
    pub error: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct NewSessionCmd {
    required: JsonValue,
}

impl NewSessionCmd {
    pub fn new() -> Self {
        NewSessionCmd {
            required: JsonValue::Null,
        }
    }

// TODO firefox specifc prefs
// [moz:firefoxOptions][prefs][name] = value;


}

#[derive(Deserialize)]
pub struct Session {
    pub sessionId: String,
}

#[derive(Serialize)]
pub struct GoCmd {
    pub url: String,
}

#[derive(Deserialize)]
pub struct Value<T> {
    pub value: T,
}

#[derive(Deserialize)]
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

#[derive(Deserialize, Serialize)]
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

#[derive(Deserialize)]
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
    use serde_json::{from_str, to_string};
    use super::*;

    #[test]
    fn element_ref_serialize() {
        let r: ElementReference = from_str("{\"element-6066-11e4-a52e-4f735466cecf\": \"ZZZZ\"}").unwrap();
        assert_eq!(r.reference, "ZZZZ");
        let r2 = from_str(&to_string(&r).unwrap()).unwrap();
        assert_eq!(r, r2);
    }
}
