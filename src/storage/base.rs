use wasm_bindgen::JsValue;
use crate::query::Operation;
use js_sys::Object;
use wasm_bindgen::prelude::wasm_bindgen;

pub trait StorageBase {
    async fn write(&mut self, op: &Operation) -> Result<JsValue, JsValue>;
    async fn query(&self) -> Result<JsValue, JsValue>;
    async fn find_document_by_id(&self, primary_key:JsValue) -> Result<JsValue, JsValue>;
    async fn count(&self) -> Result<JsValue, JsValue>;
    async fn remove(&self) -> Result<JsValue, JsValue>;
    async fn close(&self) -> Result<JsValue, JsValue>;
}

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
/**
 * Represents a record of schema types, where each key is a string and the value is a `SchemaType`.
 */
export type SchemaTypeRecord = {
    [name: string]: SchemaType
};

/**
 * Represents a function type for creating storage with the provided schema type records.
 *
 * @template T - The schema type record.
 * @param {T} records - The schema type records.
 * @returns {Promise<InternalsRecord>} A promise that resolves to the created internals record.
 */
export type CreateStorage = <T extends SchemaTypeRecord = SchemaTypeRecord>(
    records: T
) => InternalsRecord;

/**
 * Represents a storage module with a method for creating storage.
 */
export type StorageModule = {
    /**
     * Creates storage with the provided schema type records.
     *
     * @type {CreateStorage}
     */
    createStorage: CreateStorage
};
"#;


#[wasm_bindgen]
extern "C" {
    #[derive(Clone, Default)]
    pub type StorageModule;

    #[wasm_bindgen(method, catch, js_name="createStorage")]
    pub fn create_storage(this: &StorageModule, records: &Object) -> Result<JsValue, JsValue>;
}
