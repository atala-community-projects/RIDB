use std::collections::HashMap;
use js_sys::{Object, Reflect};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use crate::collection::Collection;
use crate::error::RIDBError;
use crate::storage::internals::storage_internal::StorageModule;
use crate::storage::Storage;

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
/**
 * Represents a database containing collections of documents.
 *
 * @template T - A record of schema types.
 */
export class Database<T extends SchemaTypeRecord> {

    /**
     * Creates a new `Database` instance with the provided schemas and storage module.
     *
     * @template TS - A record of schema types.
     * @param {TS} schemas - The schemas to use for the collections.
     * @param {StorageModule} storage - The storage module to use.
     * @returns {Promise<Database<TS>>} A promise that resolves to the created `Database` instance.
     */
    static create<TS extends SchemaTypeRecord>(
        schemas: TS,
        storage: StorageModule
    ): Promise<Database<TS>>;

    /**
     * The collections in the database.
     *
     * This is a read-only property where the key is the name of the collection and the value is a `Collection` instance.
     */
    readonly collections: {
        [name in keyof T]: Collection<Schema<T[name]>>
    }
}

"#;


#[wasm_bindgen(skip_typescript)]
#[derive(Clone)]
/// Represents a database with collections of documents.
pub struct Database {
    /// The storage mechanism for the database.
    pub(crate) storage: Storage
}

#[wasm_bindgen]
impl Database {

    /// Retrieves the collections in the database.
    ///
    /// This function returns an `Object` containing the collections.
    ///
    /// # Returns
    ///
    /// * `Result<Object, JsValue>` - A result containing an `Object` with the collections or an error.
    #[wasm_bindgen(getter)]
    pub fn collections(&self) -> Result<Object, JsValue> {
        // Create a HashMap to store the collections
        let  collections: HashMap<String, Collection> =
            self.storage.internals
                .iter()
                .map(|(key, internals)| {
                    (key.clone(), Collection::from(key.clone(), internals.clone()))
                })
                .collect();

        // Create a new JavaScript Object
        let object = Object::new();
        for (key, collection) in collections {
            // Set each collection in the JavaScript Object
            Reflect::set(
                &object,
                &JsValue::from_str(key.as_str()),
                &JsValue::from(collection)
            ).map_err(|e| JsValue::from(RIDBError::from(e)))?;
        }

        Ok(object)
    }

    /// Creates a new `Database` instance.
    ///
    /// This function initializes the database with the given schemas and storage module.
    ///
    /// # Arguments
    ///
    /// * `schemas_map_js` - A JavaScript `Object` containing the schemas.
    /// * `module` - The storage module to use for the database.
    ///
    /// # Returns
    ///
    /// * `Result<Database, JsValue>` - A result containing the new `Database` instance or an error.
    #[wasm_bindgen]
    pub async fn create(
        schemas_map_js: Object,
        module: StorageModule
    ) -> Result<Database, JsValue> {

        if !schemas_map_js.is_object() {
            return Err(JsValue::from(RIDBError::from("Unexpected object")));
        }
        let storage_internal_map_js = module.create_storage(&schemas_map_js.clone())?;
        let storage =
            Storage::create(storage_internal_map_js.clone().into())
                .map_err(|e| {
                    JsValue::from(RIDBError::from(e))
                })?;

        Ok(
            Database {
                storage
            }
        )
    }
}