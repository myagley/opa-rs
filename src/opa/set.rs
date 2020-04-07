use std::fmt;
use std::marker::PhantomData;

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

pub const FIELD: &str = "$__opa_private_set";
pub const NAME: &str = "$__opa_private_Set";

pub struct Set<T> {
    elements: T,
}

impl<T> Serialize for Set<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut s = serializer.serialize_struct(NAME, 1)?;
        s.serialize_field(FIELD, &self.elements)?;
        s.end()
    }
}

impl<'de, T> Deserialize<'de> for Set<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Set<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SetVisitor<T>(PhantomData<T>);

        impl<'de, T> de::Visitor<'de> for SetVisitor<T>
        where
            T: Deserialize<'de>,
        {
            type Value = Set<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a opa Set")
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Set<T>, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let value = visitor.next_key::<SetKey>()?;
                if value.is_none() {
                    return Err(serde::de::Error::custom("set key not found"));
                }

                let t: T = visitor.next_value()?;
                let s = Set { elements: t };
                Ok(s)
            }
        }

        static FIELDS: [&str; 1] = [FIELD];
        deserializer.deserialize_struct(NAME, &FIELDS, SetVisitor(PhantomData::default()))
    }
}

struct SetKey;

impl<'de> Deserialize<'de> for SetKey {
    fn deserialize<D>(deserializer: D) -> Result<SetKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FieldVisitor;

        impl<'de> de::Visitor<'de> for FieldVisitor {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a valid set field")
            }

            fn visit_str<E>(self, s: &str) -> Result<(), E>
            where
                E: de::Error,
            {
                if s == FIELD {
                    Ok(())
                } else {
                    Err(de::Error::custom("expected field with custom name"))
                }
            }
        }

        deserializer.deserialize_identifier(FieldVisitor)?;
        Ok(SetKey)
    }
}
