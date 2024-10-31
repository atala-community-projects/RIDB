
extern crate wasm_bindgen_test;

use std::collections::HashMap;
use js_sys::{ JSON};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use crate::error::RIDBError;
use crate::schema::property_type::PropertyType;


#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
/**
 * Represents a property within a schema, including various constraints and nested properties.
 */
export class Property {
    /**
     * The type of the property.
     */
    readonly type: string;

    /**
     * The version of the property, if applicable.
     */
    readonly version?: number;

    /**
     * The primary key of the property, if applicable.
     */
    readonly primaryKey?: string;

    /**
     * An optional array of nested properties for array-type properties.
     */
    readonly items?: Property[];

    /**
     * The maximum number of items for array-type properties, if applicable.
     */
    readonly maxItems?: number;

    /**
     * The minimum number of items for array-type properties, if applicable.
     */
    readonly minItems?: number;

    /**
     * The maximum length for string-type properties, if applicable.
     */
    readonly maxLength?: number;

    /**
     * The minimum length for string-type properties, if applicable.
     */
    readonly minLength?: number;

    /**
     * An optional array of required fields for object-type properties.
     */
    readonly required?: boolean;

    /**
     * An optional map of nested properties for object-type properties.
     */
    readonly properties?: {
        [name: string]: Property;
    };
}
"#;


#[wasm_bindgen(skip_typescript)]
#[derive(Deserialize, Serialize, Clone, PartialEq, Debug)]
/// Represents a property within a schema, including type, items, length constraints, and other attributes.
pub struct Property {
    /// The type of the property.
    #[serde(rename = "type")]
    pub(crate) property_type: PropertyType,

    /// Optional nested items for array-type properties.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) items: Option<Vec<Property>>,

    /// Optional maximum number of items for array-type properties.
    #[serde(rename = "maxItems", skip_serializing_if = "Option::is_none")]
    pub(crate) max_items: Option<i32>,

    /// Optional minimum number of items for array-type properties.
    #[serde(rename = "minItems", skip_serializing_if = "Option::is_none")]
    pub(crate) min_items: Option<i32>,

    /// Optional nested properties for object-type properties.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) properties: Option<HashMap<String, Property>>,

    /// Optional maximum length for string-type properties.
    #[serde(rename = "maxLength", skip_serializing_if = "Option::is_none")]
    pub(crate) max_length: Option<i32>,

    /// Optional minimum length for string-type properties.
    #[serde(rename = "minLength", skip_serializing_if = "Option::is_none")]
    pub(crate) min_length: Option<i32>,
}

#[wasm_bindgen]
impl Property {
    /// Checks is the schema is valid.
    ///
    /// # Returns
    ///
    /// Throws exception if not valid
    #[wasm_bindgen]
    pub fn is_valid(&self) -> Result<bool, RIDBError> {
        match self.property_type {
            PropertyType::String => {
                let min = self.min_length.unwrap_or_else(|| 0);
                let max = self.max_length.unwrap_or_else(|| -1);
                if min < 0 {
                    Err(RIDBError::validation("Min property not valid"))
                } else if min > max && max >= 1 {
                    Err(RIDBError::validation("Min higher than max"))
                } else {
                    Ok(true)
                }
            },
            PropertyType::Number => Ok(true),
            PropertyType::Boolean => Ok(true),
            PropertyType::Array => match self.clone().items {
                Some(items) => {
                    let item = items.first();
                    match item {
                        Some(p) => {
                            let is_valid = p.is_valid().unwrap();
                            match is_valid {
                                true => {
                                    let min = self.min_items.unwrap_or_else(|| 0);
                                    let max = self.max_items.unwrap_or_else(|| -1);
                                    if min < 0 {
                                        Err(RIDBError::validation("Min property not valid"))
                                    } else if min > max && max >= 1 {
                                        Err(RIDBError::validation("Min higher than max"))
                                    } else {
                                        Ok(true)
                                    }
                                },
                                false => Err(RIDBError::validation("Items property not valid"))
                            }
                        },
                        None => Err(RIDBError::validation("Invalid property items"))
                    }
                },
                None => Err(RIDBError::validation("Items is empty"))
            },
            PropertyType::Object => match self.clone().properties {
                Some(props) => {
                    if props.len() > 0 {
                        Ok(true)
                    } else {
                        Err(RIDBError::validation("Properties empty"))
                    }
                },
                _ => Err(RIDBError::validation("Properties empty"))
            },
            _ => Err(RIDBError::validation("Property type invalid"))
        }
    }
    /// Retrieves the type of the property.
    ///
    /// # Returns
    ///
    /// * `PropertyType` - The type of the property.
    #[wasm_bindgen(getter, js_name = "type")]
    pub fn property_type(&self) -> PropertyType {
        self.property_type
    }

    /// Retrieves the items of the property.
    ///
    /// # Returns
    ///
    /// * `Result<JsValue, JsValue>` - A result containing the items as a `JsValue` or an error.
    #[wasm_bindgen(getter)]
    pub fn items(&self) -> Result<JsValue, JsValue> {
        Ok(to_value(&self.items).map_err(|e| JsValue::from(RIDBError::from(e)))?)
    }

    /// Retrieves the maximum number of items of the property.
    ///
    /// # Returns
    ///
    /// * `Result<JsValue, JsValue>` - A result containing the maximum number of items as a `JsValue` or an error.
    #[wasm_bindgen(getter, js_name = "maxItems")]
    pub fn max_items(&self) -> Result<JsValue, JsValue> {
        Ok(to_value(&self.max_items).map_err(|e| JsValue::from(RIDBError::from(e)))?)
    }

    /// Retrieves the minimum number of items of the property.
    ///
    /// # Returns
    ///
    /// * `Result<JsValue, JsValue>` - A result containing the minimum number of items as a `JsValue` or an error.
    #[wasm_bindgen(getter, js_name = "minItems")]
    pub fn min_items(&self) -> Result<JsValue, JsValue> {
        Ok(to_value(&self.min_items).map_err(|e| JsValue::from(RIDBError::from(e)))?)
    }

    /// Retrieves the maximum length of the property.
    ///
    /// # Returns
    ///
    /// * `Result<JsValue, JsValue>` - A result containing the maximum length as a `JsValue` or an error.
    #[wasm_bindgen(getter, js_name = "maxLength")]
    pub fn max_length(&self) -> Result<JsValue, JsValue> {
        Ok(to_value(&self.max_length).map_err(|e| JsValue::from(RIDBError::from(e)))?)
    }

    /// Retrieves the minimum length of the property.
    ///
    /// # Returns
    ///
    /// * `Result<JsValue, JsValue>` - A result containing the minimum length as a `JsValue` or an error.
    #[wasm_bindgen(getter, js_name = "minLength")]
    pub fn min_length(&self) -> Result<JsValue, JsValue> {
        Ok(to_value(&self.min_length).map_err(|e| JsValue::from(RIDBError::from(e)))?)
    }

    /// Retrieves the nested properties of the property.
    ///
    /// # Returns
    ///
    /// * `Result<JsValue, JsValue>` - A result containing the nested properties as a `JsValue` or an error.
    #[wasm_bindgen(getter)]
    pub fn properties(&self) -> Result<JsValue, JsValue> {
        Ok(to_value(&self.properties).map_err(|e| JsValue::from(RIDBError::from(e)))?)
    }
}



#[cfg(feature = "browser")]
use wasm_bindgen_test::{wasm_bindgen_test_configure};
use wasm_bindgen_test::wasm_bindgen_test;

#[cfg(feature = "browser")]
wasm_bindgen_test_configure!(run_in_browser);


#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::schema::property::Property;
    use crate::schema::property_type::PropertyType;

    #[test]
    fn test_property_defaults() {
        let default_property = Property {
            property_type: PropertyType::String,
            items: None,
            max_items: None,
            min_items: None,
            max_length: None,
            min_length: None,
            properties: None,
        };
        assert_eq!(default_property.property_type, PropertyType::String);
        assert!(default_property.items.is_none());
        assert!(default_property.max_items.is_none());
        assert!(default_property.min_items.is_none());
        assert!(default_property.max_length.is_none());
        assert!(default_property.min_length.is_none());
        assert!(default_property.properties.is_none());
        assert!(default_property.is_valid().unwrap());
    }

    #[test]
    fn test_property_array_items_required() {
        let default_property = Property {
            property_type: PropertyType::Array,
            items: None,
            max_items: None,
            min_items: None,
            max_length: None,
            min_length: None,
            properties: None,
        };
        // Test default values to ensure proper initialization
        assert_eq!(default_property.property_type, PropertyType::Array);
        assert!(default_property.items.is_none());
        assert!(default_property.max_items.is_none());
        assert!(default_property.min_items.is_none());
        assert!(default_property.max_length.is_none());
        assert!(default_property.min_length.is_none());
        assert!(default_property.properties.is_none());
        let result = default_property.is_valid();
        // Check the result for an error message
        match result {
            Ok(_) => panic!("Expected an error, but got Ok"),
            Err(js_val) => assert_eq!(js_val.message, "Validation Error: Items is empty", "Error: Expected 'Validation Error: Items is empty'")
        }
    }

    #[test]
    fn test_property_array_items_with_max_min_items_wrong() {
        let prop = Property {
            property_type: PropertyType::String,
            items:None,
            max_items: None,
            min_items: None,
            max_length: None,
            min_length: None,
            properties: None,
        };
        let default_property = Property {
            property_type: PropertyType::Array,
            items: Some(vec![prop]),
            max_items: Some(-1),
            min_items: Some(-1),
            max_length: None,
            min_length: None,
            properties: None,
        };
        let result = default_property.is_valid();
        match result {
            Ok(_js_val) => panic!("Expected an error, but got Ok"),
            Err(js_val) => assert_eq!(js_val.message, "Validation Error: Min property not valid", "Error: Expected 'Validation Error: Min property not valid'")
        }
    }

    #[test]
    fn test_property_array_items_with_max_min_items_wrong_min_higher() {
        let prop = Property {
            property_type: PropertyType::String,
            items:None,
            max_items: None,
            min_items: None,
            max_length: None,
            min_length: None,
            properties: None,
        };

        let default_property2 = Property {
            property_type: PropertyType::Array,
            items: Some(vec![prop]),
            max_items: Some(1),
            min_items: Some(2),
            max_length: None,
            min_length: None,
            properties: None,
        };
        let result = default_property2.is_valid();
        // Check the result for an error message
        match result {
            Ok(_) => panic!("Expected an error, but got Ok"),
            Err(js_val) => assert_eq!(js_val.message, "Validation Error: Min higher than max", "Error: Expected 'Validation Error: Min higher than max'")
        }
    }


    #[test]
    fn test_property_array_items_with_max_min_items_wrong_min_lower0() {
        let prop = Property {
            property_type: PropertyType::String,
            items:None,
            max_items: None,
            min_items: None,
            max_length: None,
            min_length: None,
            properties: None,
        };

        let default_property2 = Property {
            property_type: PropertyType::Array,
            items: Some(vec![prop]),
            max_items: Some(1),
            min_items: Some(-1),
            max_length: None,
            min_length: None,
            properties: None,
        };
        let result = default_property2.is_valid();
        // Check the result for an error message
        match result {
            Ok(_) => panic!("Expected an error, but got Ok"),
            Err(js_val) => assert_eq!(js_val.message, "Validation Error: Min property not valid", "Error: Expected 'Validation Error: Min property not valid'")
        }
    }


    #[test]
    fn test_property_number_ok() {
        let default_property2 = Property {
            property_type: PropertyType::Number,
            items: None,
            max_items: None,
            min_items: None,
            max_length: None,
            min_length: None,
            properties: None,
        };
        let result = default_property2.is_valid();
        // Check the result for an error message
        match result {
            Ok(_) => {},
            Err(_) => panic!("Expected an error, but got Ok")
        }
    }

    #[test]
    fn test_property_boolean_ok() {
        let default_property2 = Property {
            property_type: PropertyType::Boolean,
            items: None,
            max_items: None,
            min_items: None,
            max_length: None,
            min_length: None,
            properties: None,
        };
        let result = default_property2.is_valid();
        // Check the result for an error message
        match result {
            Ok(_) => {},
            Err(_) => panic!("Expected an error, but got Ok")
        }
    }

    #[test]
    fn test_property_string_with_max_min_length_wrong_min_higher() {
        let default_property2 = Property {
            property_type: PropertyType::String,
            items: None,
            max_items: None,
            min_items: None,
            max_length: Some(1),
            min_length: Some(2),
            properties: None,
        };
        let result = default_property2.is_valid();
        // Check the result for an error message
        match result {
            Ok(_) => panic!("Expected an error, but got Ok"),
            Err(js_val) => assert_eq!(js_val.message, "Validation Error: Min higher than max", "Error: Expected 'Validation Error: Min higher than max'")
        }
    }

    #[test]
    fn test_property_string_with_max_min_length_wrong_min_lower0() {
        let default_property2 = Property {
            property_type: PropertyType::String,
            items: None,
            max_items: None,
            min_items: None,
            max_length: Some(1),
            min_length: Some(-1),
            properties: None,
        };
        let result = default_property2.is_valid();
        // Check the result for an error message
        match result {
            Ok(_) => panic!("Expected an error, but got Ok"),
            Err(js_val) => assert_eq!(js_val.message, "Validation Error: Min property not valid", "Error: Expected 'Validation Error: Min property not valid'")
        }
    }

    #[test]
    fn test_property_object_no_props_err() {
        let result = Property {
            property_type: PropertyType::Object,
            items: None,
            max_items: None,
            min_items: None,
            max_length: None,
            min_length: None,
            properties: None,
        }.is_valid();
        // Check the result for an error message
        match result {
            Ok(_) => panic!("Expected an error, but got Ok"),
            Err(js_val) => assert_eq!(js_val.message, "Validation Error: Properties empty", "Error: Expected 'Validation Error: Properties empty'")
        }
    }

    #[test]
    fn test_property_object_props_empty_err() {
        let result = Property {
            property_type: PropertyType::Object,
            items: None,
            max_items: None,
            min_items: None,
            max_length: None,
            min_length: None,
            properties: Some(HashMap::new()),
        }.is_valid();
        // Check the result for an error message
        match result {
            Ok(_) => panic!("Expected an error, but got Ok"),
            Err(js_val) => assert_eq!(js_val.message, "Validation Error: Properties empty", "Error: Expected 'Validation Error: Properties empty'")
        }
    }

}


#[wasm_bindgen_test]
fn test_property_creation() {
    let property_js = r#"{
        "type": "string",
        "maxLength": 50,
        "minLength": 1
    }"#;
    let property_value = JSON::parse(property_js).unwrap();
    let property = serde_wasm_bindgen::from_value::<Property>(property_value).unwrap();

    assert_eq!(property.property_type(), PropertyType::String);
    assert_eq!(property.max_length().unwrap(), JsValue::from(50));
    assert_eq!(property.min_length().unwrap(), JsValue::from(1));
}

#[wasm_bindgen_test]
fn test_property_validation() {
    let property_js = r#"{
        "type": "string",
        "maxLength": 50,
        "minLength": 1
    }"#;
    let property_value = JSON::parse(property_js).unwrap();
    let property = serde_wasm_bindgen::from_value::<Property>(property_value).unwrap();

    assert!(property.is_valid().is_ok());
}

#[wasm_bindgen_test]
fn test_invalid_property() {
    let property_js = r#"{
        "type": "string",
        "maxLength": 10,
        "minLength": 20
    }"#;
    let property_value = JSON::parse(property_js).unwrap();
    let property = serde_wasm_bindgen::from_value::<Property>(property_value).unwrap();

    assert!(property.is_valid().is_err());
}