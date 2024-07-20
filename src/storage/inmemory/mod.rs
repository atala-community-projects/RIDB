use std::collections::HashMap;
use js_sys::{Object, Reflect};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use crate::query::{Operation, OpType};
use crate::schema::Schema;
use crate::storage::base::StorageBase;
use crate::storage::internals::base_storage::BaseStorage;

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
/**
 * Represents an in-memory storage system extending the base storage functionality.
 *
 * @template T - The schema type.
 */
export class InMemory<T extends SchemaType> extends BaseStorage<T> {
    /**
     * Frees the resources used by the in-memory storage.
     */
    free(): void;
}
"#;



#[wasm_bindgen(skip_typescript)]
pub struct InMemory{
    pub(crate) base: BaseStorage,
    pub(crate) by_index: HashMap<
        String, HashMap<
            String, JsValue
        >
    >
}


impl StorageBase for InMemory {
    async fn write(&mut self, op: &Operation) -> Result<JsValue, JsValue> {
        todo!()
    }

    async fn query(&self) -> Result<JsValue, JsValue> {
        todo!()
    }

    async  fn find_document_by_id(&self, primary_key:JsValue) -> Result<JsValue, JsValue> {
        todo!()
    }

    async fn count(&self) -> Result<JsValue, JsValue> {
        todo!()
    }

    async fn remove(&self) -> Result<JsValue, JsValue> {
        todo!()
    }

    async fn close(&self) -> Result<JsValue, JsValue> {
        todo!()
    }
}

#[wasm_bindgen]
impl InMemory {
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str, schema_type: JsValue) -> Result<InMemory, JsValue> {
        let base_res = BaseStorage::new(
            name.to_string(),
            schema_type
        );
        match base_res {
            Ok(base) => Ok(
                InMemory {
                    base,
                    by_index: HashMap::new()
                }
            ),
            Err(e) => Err(e)
        }
    }

    #[wasm_bindgen(getter)]
    pub fn by_index(&self) -> Result<JsValue, JsValue> {
        let outer_obj = Object::new();

        for (outer_key, inner_map) in &self.by_index {
            let inner_obj = Object::new();
            for (inner_key, value) in inner_map {
                Reflect::set(&inner_obj, &JsValue::from_str(inner_key), value)
                    .map_err(|_| JsValue::from_str("Failed to set inner object property"))?;
            }
            Reflect::set(&outer_obj, &JsValue::from_str(outer_key), &JsValue::from(inner_obj))
                .map_err(|_| JsValue::from_str("Failed to set outer object property"))?;
        }

        Ok(JsValue::from(outer_obj))
    }

    #[wasm_bindgen(getter)]
    pub fn schema(&self) -> Schema {
        self.base.schema.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.base.name.clone()
    }

    #[wasm_bindgen(js_name = "write")]
    pub async fn write_js(&mut self, op: &Operation) -> Result<JsValue, JsValue> {
        self.write(op).await
    }

    #[wasm_bindgen(js_name = "query")]
    pub async fn query_js(&self) -> Result<JsValue, JsValue> {
        self.query().await
    }

    #[wasm_bindgen(js_name = "findDocumentById")]
    pub async fn find_document_by_id_js(&self, primary_key:JsValue) -> Result<JsValue, JsValue> {
        self.find_document_by_id(primary_key).await
    }

    #[wasm_bindgen(js_name = "count")]
    pub async fn count_js(&self) -> Result<JsValue, JsValue> {
        self.count().await
    }

    #[wasm_bindgen(js_name = "remove")]
    pub async fn remove_js(&self) -> Result<JsValue, JsValue> {
        self.remove().await
    }

    #[wasm_bindgen(js_name = "close")]
    pub async fn close_js(&self) -> Result<JsValue, JsValue> {
        self.close().await
    }
}
