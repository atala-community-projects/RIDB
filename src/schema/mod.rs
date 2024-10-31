

pub mod property_type;
pub mod property;

use std::collections::HashMap;
use js_sys::{Object, Reflect, JSON};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_test::{ wasm_bindgen_test};
use crate::error::RIDBError;
use crate::schema::property::Property;

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
/**
 * Represents the type definition for a schema.
 */
export type SchemaType = {
    /**
     * The version of the schema.
     */
    readonly version: number;

    /**
     * The primary key of the schema.
     */
    readonly primaryKey: string;

    /**
     * The type of the schema.
     */
    readonly type: string;

    /**
     * An optional array of required fields.
     */
    readonly required?: string[];

    /**
     * An optional array of indexes.
     */
    readonly indexes?: string[];

    /**
     * The properties defined in the schema.
     */
    readonly properties: {
        [name: string]: Property;
    };
};

/**
 * Represents a schema, including its definition and related methods.
 *
 * @template T - The schema type.
 */
export class Schema<T extends SchemaType> {
    /**
     * The schema definition.
     */
    schema: Schema<T>;

    /**
     * Creates a new `Schema` instance from the provided definition.
     *
     * @template TS - The schema type.
     * @param {TS} definition - The schema definition.
     * @returns {Schema<TS>} The created `Schema` instance.
     */
    static create<TS extends SchemaType>(definition: TS): Schema<TS>;

    /**
     * The version of the schema.
     */
    readonly version: number;

    /**
     * The primary key of the schema.
     */
    readonly primaryKey: string;

    /**
     * The type of the schema.
     */
    readonly type: string;

    /**
     * An optional array of indexes.
     */
    readonly indexes?: string[];

    /**
     * The properties defined in the schema.
     */
    readonly properties: {
        [K in keyof T['properties'] as T['properties'][K]['required'] extends false ? K : never]?: T['properties'][K];
    } & {
        [K in keyof T['properties'] as T['properties'][K]['required'] extends false ? never : K]: T['properties'][K];
    };
    /**
     * Converts the schema to a JSON representation.
     *
     * @returns {SchemaType} The JSON representation of the schema.
     */
    toJSON(): SchemaType;
}
"#;

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
#[wasm_bindgen(skip_typescript)]
/// Represents the schema of a collection, including version, primary key, type, required fields, properties, and indexes.
pub struct Schema {
    /// The version of the schema.
    pub(crate) version: i32,
    /// The primary key of the schema.
    #[serde(rename = "primaryKey")]
    pub(crate) primary_key: String,
    /// The type of the schema.
    #[serde(rename = "type")]
    pub(crate) schema_type: String,
    /// The properties defined in the schema.
    pub(crate) properties: HashMap<String, Property>,
    /// The indexes defined in the schema, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) indexes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) required: Option<Vec<String>>,
}


#[wasm_bindgen]
impl Schema {
    pub fn validate_schema(&self, document: JsValue) -> Result<(), JsValue> {
        let required = self.required.clone().unwrap_or(Vec::new());
        let properties = self.properties.clone();

        for (key, prop) in properties {
            let value = Reflect::get(&document, &JsValue::from_str(&key))
                .map_err(|e| JsValue::from_str(&format!("Failed to get property '{}': {:?}", key, e)))?;

            if value.is_undefined() {
                if required.contains(&key) {
                    return Err(JsValue::from_str(&format!("Field '{}' is required", key)));
                }
            } else {
                if !self.is_type_correct(&value, prop.property_type) {
                    return Err(JsValue::from_str(&format!(
                        "Field '{}' should be of type '{:?}'",
                        key, prop.property_type
                    )));
                }
            }
        }
        Ok(())
    }


    fn is_type_correct(&self, value: &JsValue, prop_type: PropertyType) -> bool {
        match prop_type {
            PropertyType::String => value.is_string(),
            PropertyType::Number => {
                // Check if the value can be converted to an f64
                value.as_f64().is_some()
            },
            PropertyType::Boolean => value.as_bool().is_some(),
            PropertyType::Object => {
                // Exclude null, arrays, and functions
                value.is_object()
                    && !value.is_null()
                    && !js_sys::Array::is_array(value)
            },
            PropertyType::Array => js_sys::Array::is_array(value),
            // Add other property types as needed
            _ => false,
        }
    }

    pub fn is_valid(&self) -> Result<bool, RIDBError> {
        // Check if the schema type is "object"
        let schema_type = self.get_schema_type();
        if schema_type != "object" {
            return Err(RIDBError::validation(
                format!("Schema type is invalid (\"{}\")", schema_type).as_str(),
            ));
        }
        // Validate all properties
        for property in self.properties.values() {
            // Directly propagate the specific error from the property's validation
            property.is_valid()?; // This will automatically return Err if is_valid() returns an Err
        }
        // If all properties are valid and the schema type is "object"
        Ok(true)
    }

    /// Creates a new `Schema` instance from a given `JsValue`.
    ///
    /// # Arguments
    ///
    /// * `schema` - A `JsValue` representing the schema.
    ///
    /// # Returns
    ///
    /// * `Result<Schema, JsValue>` - A result containing the new `Schema` instance or an error.
    #[wasm_bindgen]
    pub fn create(schema: JsValue) -> Result<Schema, JsValue> {
        let schema: Schema = from_value(schema)
            .map_err(|e| JsValue::from(RIDBError::from(e)))?;
        let valid = schema.is_valid();
        match valid {
            Ok(_) =>  Ok(schema),
            Err(e) => Err(JsValue::from(e))
        }
    }

    /// Retrieves the version of the schema.
    ///
    /// # Returns
    ///
    /// * `i32` - The version of the schema.
    #[wasm_bindgen(getter, js_name="version")]
    pub fn get_version(&self) -> i32 {
        self.version
    }

    /// Retrieves the primary key of the schema.
    ///
    /// # Returns
    ///
    /// * `String` - The primary key of the schema.
    #[wasm_bindgen(getter, js_name="primaryKey")]
    pub fn get_primary_key(&self) -> String {
        self.primary_key.clone()
    }

    /// Retrieves the type of the schema.
    ///
    /// # Returns
    ///
    /// * `String` - The type of the schema.
    #[wasm_bindgen(getter, js_name="type")]
    pub fn get_schema_type(&self) -> String {
        self.schema_type.clone()
    }

    /// Retrieves the indexes of the schema, if any.
    ///
    /// # Returns
    ///
    /// * `Option<Vec<String>>` - The indexes of the schema, if any.
    #[wasm_bindgen(getter, js_name="indexes")]
    pub fn get_indexes(&self) -> Option<Vec<String>> {
        self.indexes.clone()
    }

    #[wasm_bindgen(getter, js_name="required")]
    pub fn get_required(&self) -> Option<Vec<String>> {
        self.required.clone()
    }

    /// Retrieves the properties of the schema.
    ///
    /// # Returns
    ///
    /// * `Result<JsValue, JsValue>` - A result containing the properties as a `JsValue` or an error.
    #[wasm_bindgen(getter, js_name="properties")]
    pub fn get_properties(&self) -> Result<JsValue, JsValue> {
        // Create a new JavaScript object to hold all properties
        let result = Object::new();

        for (key, property) in &self.properties {
            // Create a JavaScript object for the property
            let prop_obj = Object::new();

            // Get the 'type' field as a string
            let prop_type_str = match property.property_type {
                PropertyType::String => "string",
                PropertyType::Number => "number",
                PropertyType::Boolean => "boolean",
                PropertyType::Array => "array",
                PropertyType::Object => "object",
                _ => "object",
            };

            // Set the 'type' field in the property object
            Reflect::set(&prop_obj, &JsValue::from_str("type"), &JsValue::from_str(prop_type_str))?;

            // If you have other fields like 'maxLength', 'minLength', etc., set them here as well
            // Example:
            if let Some(max_length) = property.max_length {
                Reflect::set(&prop_obj, &JsValue::from_str("maxLength"), &JsValue::from_f64(max_length as f64))?;
            }

            // Set the property object in the result object under the property name
            Reflect::set(&result, &JsValue::from_str(key), &prop_obj)?;
        }

        // Return the result as a JsValue
        Ok(result.into())
    }

}


#[cfg(feature = "browser")]
use wasm_bindgen_test::{wasm_bindgen_test_configure};
use crate::schema::property_type::PropertyType;

#[cfg(feature = "browser")]
wasm_bindgen_test_configure!(run_in_browser);



#[wasm_bindgen_test]
fn test_schema_creation() {
    let schema_js = r#"{
        "version": 1,
        "primaryKey": "id",
        "type": "object",
        "properties": {
            "id": {"type": "string"},
            "name": {"type": "string"},
            "age": {"type": "number"}
        }
    }"#;
    let schema_value = JSON::parse(&schema_js).unwrap();
    let schema = Schema::create(schema_value).unwrap();
    assert_eq!(schema.get_version(), 1);
    assert_eq!(schema.get_primary_key(), "id");
    assert_eq!(schema.get_schema_type(), "object");
}

#[wasm_bindgen_test]
fn test_schema_validation() {
    let schema_js = r#"{
        "version": 1,
        "primaryKey": "id",
        "type": "object",
        "properties": {
            "id": {"type": "string"}
        }
    }"#;
    let schema_value = JSON::parse(schema_js).unwrap();
    let schema = Schema::create(schema_value).unwrap();

    assert!(schema.is_valid().is_ok());
}


#[wasm_bindgen_test]
fn test_invalid_schema() {
    let schema_js = r#"{
        "version": 1,
        "primaryKey": "id",
        "type": "invalid",
        "properties": {
            "id": {"type": "string"}
        }
    }"#;
    let schema_value = JSON::parse(schema_js).unwrap();
    let result = Schema::create(schema_value);

    assert!(result.is_err());
}

