pub mod storage_internal;
pub mod base_storage;
use js_sys::Reflect;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen::prelude::wasm_bindgen;
use crate::error::RIDBError;
use crate::query::{Operation, OpType};
use crate::schema::property_type::PropertyType;
use crate::schema::Schema;
use crate::storage::internals::storage_internal::StorageInternal;

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
/**
 * Represents the internals of a storage system, including the base storage and schema.
 *
 * @template T - The schema type.
 */
export class Internals<T extends SchemaType> {
    /**
     * The base storage instance.
     */
    readonly internal: BaseStorage<T>;
    /**
     * Creates a new `Internals` instance with the provided base storage.
     *
     * @param {BaseStorage<T>} internal - The base storage instance.
     */
    constructor(internal: BaseStorage<T>);
    /**
     * The schema associated with the storage.
     */
    readonly schema: T;
}
"#;


#[wasm_bindgen(skip_typescript)]
#[derive(Clone, Default)]
/// Represents the internals of a storage system, including schema and storage internal components.
pub struct Internals {
    /// The schema of the storage system.
    pub(crate) schema: Schema,
    /// The internal storage mechanism.
    pub(crate) internal: StorageInternal,
}

#[wasm_bindgen]
impl Internals {
    /// Creates a new `Internals` instance with the provided internal storage.
    ///
    /// # Arguments
    ///
    /// * `internal` - The internal storage mechanism.
    ///
    /// # Returns
    ///
    /// * `Internals` - A new instance of `Internals`.
    #[wasm_bindgen(constructor)]
    pub fn new(internal: StorageInternal) -> Result<Internals, JsValue> {
        let schema = internal.schema().clone();
        match schema.is_valid() {
            Ok(_) => Ok(Internals { schema, internal }),
            Err(e) => Err(JsValue::from(e))
        }
    }

    /// Ensures that the document has a primary key, generating one if necessary.
    ///
    /// # Arguments
    ///
    /// * `document` - The document to ensure a primary key for.
    ///
    /// # Returns
    ///
    /// * `Result<JsValue, JsValue>` - A result containing the document with the primary key or an error.
    fn ensure_primary_key(&self, document: JsValue) -> Result<JsValue, JsValue> {
        let key = self.schema.primary_key.clone();
        let doc_property = Reflect::get(&document, &JsValue::from(&key))
            .map_err(|e| JsValue::from(RIDBError::from(e)))?;
        let properties = self.schema.properties.clone();
        let primary_key_property = properties
            .get(&key)
            .ok_or(RIDBError::from("Invalid Schema cannot find primaryKey field"))
            .map_err(|e| JsValue::from(e))?;
        let primary_key_type = primary_key_property.property_type();
        if doc_property.is_null() || doc_property.is_undefined() {
            if primary_key_type == PropertyType::String {
                Reflect::set(&document, &JsValue::from(&key), &JsValue::from("12345"))
                    .map_err(|e| JsValue::from(RIDBError::from(e)))?;
            } else {
                Reflect::set(&document, &JsValue::from(&key), &JsValue::from(12345))
                    .map_err(|e| JsValue::from(RIDBError::from(e)))?;
            }
        }
        let doc_property = Reflect::get(&document, &JsValue::from(&key))
            .map_err(|e| JsValue::from(RIDBError::from(e)))?;
        if primary_key_type == PropertyType::String && !doc_property.is_string() {
            Err(JsValue::from(RIDBError::from("Unexpected primary key should be a string")))
        } else if primary_key_type == PropertyType::Number && !doc_property.is_bigint() {
            Err(JsValue::from(RIDBError::from("Unexpected primary key should be number")))
        } else {
            Ok(document)
        }
    }

    /// Validates a document against the schema, ensuring all required fields and types are correct.
    ///
    /// # Arguments
    ///
    /// * `document_without_pk` - The document to validate.
    ///
    /// # Returns
    ///
    /// * `Result<JsValue, JsValue>` - A result containing the validated document or an error.
    pub fn validate_schema(&self, document_without_pk: JsValue) -> Result<JsValue, JsValue> {
        let document = self.ensure_primary_key(document_without_pk)?;
        let properties = self.schema.properties.clone();
        for (key, prop) in properties {
            let value = Reflect::get(&document, &JsValue::from_str(key.clone().as_str()))?;
            if value.is_undefined() {
                if let Some(required_fields) = &self.schema.required {
                    if required_fields.contains(&key) {
                        return Err(JsValue::from(RIDBError::error(format!("Field {} is required", key).as_str())));
                    }
                }
            } else {
                if !self.is_type_correct(&value, prop.property_type) {
                    return Err(JsValue::from(RIDBError::error(format!("Field {} should match type {:?}", key, prop.property_type).as_str())));
                }
            }
        }
        Ok(document)
    }

    /// Checks if a value is of the correct type based on the property type.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to check.
    /// * `prop_type` - The expected property type.
    ///
    /// # Returns
    ///
    /// * `bool` - `true` if the value is of the correct type, otherwise `false`.
    pub fn is_type_correct(&self, value: &JsValue, prop_type: PropertyType) -> bool {
        match prop_type {
            PropertyType::String => value.is_string(),
            PropertyType::Number => value.as_f64().is_some(),
            PropertyType::Object => value.is_object(),
            PropertyType::Array => {
                if let Some(array) = value.dyn_ref::<js_sys::Array>() {
                    !array.is_array()
                } else {
                    false
                }
            },
            PropertyType::Boolean => value.is_falsy() || value.is_truthy(),
            _ => false,
        }
    }

    /// Writes a document to the storage system after validating the schema.
    ///
    /// # Arguments
    ///
    /// * `document_without_pk` - The document to write.
    ///
    /// # Returns
    ///
    /// * `Result<JsValue, JsValue>` - A result containing the written document or an error.
    #[wasm_bindgen]
    pub async fn write(&self, document_without_pk: JsValue) -> Result<JsValue, JsValue> {
        let primary_key = self.schema.primary_key.clone();
        let document = self.validate_schema(document_without_pk)
            .map_err(|e| JsValue::from(RIDBError::from(e)))?;
        let indexes = match self.schema.indexes.clone() {
            Some(mut existing) => {
                existing.push(primary_key.clone());
                existing
            },
            _ => {
                let mut new_index: Vec<String> = Vec::new();
                new_index.push(primary_key.clone());
                new_index
            }
        };
        let op = Operation {
            collection: self.internal.name().clone(),
            op_type: OpType::CREATE,
            data: document,
            indexes,
        };
        let result = self.internal.write(op).await;
        result.map_err(|e| JsValue::from(RIDBError::from(e)))
    }

    /// Placeholder for querying the storage system.
    #[wasm_bindgen]
    pub async fn query() {}

    /// Placeholder for finding a document by its ID.
    #[wasm_bindgen]
    pub async fn find_document_by_id(&self, primary_key: JsValue) -> Result<JsValue, JsValue>{
        let result = self.internal.findDocument_by_id(primary_key).await;
        todo!()
    }

    /// Placeholder for counting documents in the storage system.
    #[wasm_bindgen]
    pub async fn count() {}

    /// Placeholder for removing a document from the storage system.
    #[wasm_bindgen]
    pub async fn remove() {}

    /// Placeholder for closing the storage system.
    #[wasm_bindgen]
    pub async fn close() {}
}
