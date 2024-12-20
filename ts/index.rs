use js_sys::{Array, Promise, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen::prelude::{wasm_bindgen, Closure};
use wasm_bindgen_futures::JsFuture;
use crate::query::Query;
use crate::schema::Schema;
use crate::storage::base::StorageBase;
use crate::storage::internals::base_storage::BaseStorage;
use crate::storage::internals::core::CoreStorage;
use crate::operation::{OpType, Operation};
use web_sys::{IdbDatabase, IdbOpenDbRequest, IdbRequest, console};
use std::sync::Arc;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Weak;
use lazy_static::lazy_static;
#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
/**
 * Represents an IndexDB storage system extending the base storage functionality.
 *
 * @template T - The schema type.
 */
export class IndexDB<T extends SchemaType> extends BaseStorage<T> {
    /**
     * Frees the resources used by the IndexDB storage.
     */
    free(): void;

    static create<TS extends SchemaType>(
        name: string,
        schema_type: TS,
        migrations: MigrationPathsForSchema<TS>,
    ): Promise<IndexDB<TS>>;
}
"#;

#[wasm_bindgen(skip_typescript)]
pub struct IndexDB {
    core: CoreStorage,
    base: BaseStorage,
    db: IdbDatabase,
    _error_handler: Option<Closure<dyn FnMut(web_sys::Event)>>,
    _success_handler: Option<Closure<dyn FnMut(web_sys::Event)>>,
} 

impl Drop for IndexDB {
    fn drop(&mut self) {
        self._error_handler.take();
        self._success_handler.take();
        self.db.close();
    }
}

impl StorageBase for IndexDB {
    async fn write(&mut self, op: &Operation) -> Result<JsValue, JsValue> {
        console::log_1(&JsValue::from_str(&format!("Starting operation...")));
        let store_name = "documents";
        
        let transaction = match self.db.transaction_with_str_and_mode(
            store_name,
            web_sys::IdbTransactionMode::Readwrite,
        ) {
            Ok(t) => t,
            Err(e) => {
                console::error_1(&JsValue::from_str("Failed to create transaction"));
                return Err(e);
            }
        };

        let store = match transaction.object_store(store_name) {
            Ok(s) => s,
            Err(e) => {
                console::error_1(&JsValue::from_str("Failed to get object store"));
                return Err(e);
            }
        };

        match op.op_type {
            OpType::CREATE | OpType::UPDATE => {
                let document = op.data.clone();
                
                // Extract primary key
                let primary_key = self.base.schema.primary_key.clone();
                let pk_value = match Reflect::get(&document, &JsValue::from_str(&primary_key)) {
                    Ok(v) => v,
                    Err(e) => {
                        console::error_1(&JsValue::from_str(&format!(
                            "Failed to get primary key '{}' from document",
                            primary_key
                        )));
                        return Err(e);
                    }
                };

                if pk_value.is_undefined() || pk_value.is_null() {
                    console::error_1(&JsValue::from_str(&format!(
                        "Document must contain primary key '{}'",
                        primary_key
                    )));
                    return Err(JsValue::from_str("Document must contain a primary key"));
                }

                console::log_1(&JsValue::from_str(&format!(
                    "Processing  operation for document with {} = {:?}",
                     primary_key, pk_value
                )));

                // Validate document against schema
                self.base.schema.validate_schema(document.clone())?;

                // Store the document and wait for completion
                let request = store.put_with_key(&document, &pk_value)?;

                let promise = Promise::new(&mut |resolve, reject| {
                    let onsucess = Closure::once(Box::new(move |event: web_sys::Event| {
                        let request: IdbRequest = event.target().unwrap().dyn_into().unwrap();
                        let result = request.result().unwrap();
                        resolve.call1(&JsValue::undefined(), &result).unwrap();
                    }));
                    
                    let onerror = Closure::once(Box::new(move |e: web_sys::Event| {
                        reject.call1(&JsValue::undefined(), &e).unwrap();
                    }));
                    
                    request.set_onsuccess(Some(onsucess.as_ref().unchecked_ref()));
                    request.set_onerror(Some(onerror.as_ref().unchecked_ref()));

                    onsucess.forget();
                    onerror.forget();
                });

                JsFuture::from(promise).await?;

                Ok(
                    document.clone()
                )
            },
            OpType::DELETE => {
                let pk_value = op.data.clone();
                if pk_value.is_undefined() || pk_value.is_null() {
                    return Err(JsValue::from_str("Primary key value is required for delete operation"));
                }

                // Delete the document and wait for completion
                let request = store.delete(&pk_value)?;
                let promise = Promise::new(&mut |resolve, reject| {
                    let onsucess = Closure::once(Box::new(move |_event: web_sys::Event| {
                        resolve.call1(&JsValue::undefined(), &JsValue::from_str("Document deleted")).unwrap();
                    }));
                    
                    let onerror = Closure::once(Box::new(move |e: web_sys::Event| {
                        reject.call1(&JsValue::undefined(), &e).unwrap();
                    }));
                    
                    request.set_onsuccess(Some(onsucess.as_ref().unchecked_ref()));
                    request.set_onerror(Some(onerror.as_ref().unchecked_ref()));
                    onsucess.forget();
                    onerror.forget();
                });

                JsFuture::from(promise).await
            },
            _ => Err(JsValue::from_str("Unsupported operation type")),
        }
    }

    async fn find(&self, collection_name: &str, query: Query) -> Result<JsValue, JsValue> {
        // console::log_2(&JsValue::from_str("Starting find operation with query:"), &query.query);
        let store_name = "documents";
        let transaction = self.db.transaction_with_str(store_name)?;
        let store = transaction.object_store(store_name)?;
        
        let normalized_query = query.parse()?;
        let request = store.get_all()?;
        let normalized_query = normalized_query.clone();
        let promise = Promise::new(&mut |resolve, reject| {
            let value = normalized_query.clone();
            let core = self.core.clone();
            let onsucess = Closure::once(Box::new(move |event: web_sys::Event| {
                let request: IdbRequest = event.target().unwrap().dyn_into().unwrap();
                let result = request.result().unwrap();
                // Filter documents based on query
                let filtered = Array::new();


                if !result.is_undefined() && !result.is_null() {
                    let documents = Array::from(&result);

                    for i in 0..documents.length() {
                        let doc = documents.get(i);
                        if let Ok(matches) = core.document_matches_query(&doc, &value) {
                            if matches {
                                filtered.push(&doc);
                            }
                        }
                    }
                }
                
                resolve.call1(&JsValue::undefined(), &filtered).unwrap();
            }));
            
            request.set_onsuccess(Some(onsucess.as_ref().unchecked_ref()));
            onsucess.forget();
        });

        JsFuture::from(promise).await
    }

    async fn find_document_by_id(&self, collection_name: &str, primary_key_value: JsValue) -> Result<JsValue, JsValue> {
        // console::log_2(&JsValue::from_str("Finding document by ID:"), &primary_key_value);
        let store_name = "documents";
        let transaction = self.db.transaction_with_str(store_name)?;
        let store = transaction.object_store(store_name)?;
        
        let request = store.get(&primary_key_value)?;
        
        let promise = Promise::new(&mut |resolve, reject| {
            let onsucess = Closure::once(Box::new(move |event: web_sys::Event| {
                let request: IdbRequest = event.target().unwrap().dyn_into().unwrap();
                let result = request.result().unwrap();
                
                if result.is_undefined() {
                    reject.call1(&JsValue::undefined(), &JsValue::from_str("Document not found")).unwrap();
                } else {
                    resolve.call1(&JsValue::undefined(), &result).unwrap();
                }
            }));
            
            request.set_onsuccess(Some(onsucess.as_ref().unchecked_ref()));
            onsucess.forget();
        });

        JsFuture::from(promise).await
    }

    async fn count(&self,collection_name: &str,   query: Query) -> Result<JsValue, JsValue> {
        let store_name = "documents";
        let transaction = self.db.transaction_with_str(store_name)?;
        let store = transaction.object_store(store_name)?;
        
        let normalized_query = query.parse()?;
        let request = store.get_all()?;
        let normalized_query = normalized_query.clone();        
        let promise = Promise::new(&mut |resolve, reject| {
            let value = normalized_query.clone();
            let core = self.core.clone();
            let onsucess = Closure::once(Box::new(move |event: web_sys::Event| {
                let request: IdbRequest = event.target().unwrap().dyn_into().unwrap();
                let result = request.result().unwrap();
                let documents = Array::from(&result);
                
                let mut count = 0;
                for i in 0..documents.length() {
                    let doc = documents.get(i);
                    if let Ok(matches) = core.document_matches_query(&doc, &value) {
                        if matches {
                            count += 1;
                        }
                    }
                }
                
                resolve.call1(&JsValue::undefined(), &JsValue::from_f64(count as f64)).unwrap();
            }));
            
            request.set_onsuccess(Some(onsucess.as_ref().unchecked_ref()));
            onsucess.forget();
        });

        JsFuture::from(promise).await
    }

    async fn close(&self) -> Result<JsValue, JsValue> {
        self.db.close();
        Ok(JsValue::from_str("IndexDB database closed"))
    }
}

#[wasm_bindgen]
impl IndexDB {
    #[wasm_bindgen]
    pub async fn create(name: &str, schema_type: JsValue, migrations: JsValue) -> Result<IndexDB, JsValue> {
        let base = BaseStorage::new(
            name.to_string(),
            schema_type.clone(),
            migrations
        )?;

        // Try to get existing connection from pool
        let db = match POOL.get_connection(name) {
            Some(db) => db,
            None => {
                // Create new connection if none exists
                let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window object"))?;
                let idb = window.indexed_db()?.ok_or_else(|| JsValue::from_str("IndexedDB not available"))?;
                
                let version = 1;
                let db_request = idb.open_with_u32(name, version)?;

                let db = JsFuture::from(Promise::new(&mut |resolve, reject| {
                    let onupgradeneeded = Closure::once(Box::new(move |event: web_sys::Event| {
                        let db: IdbDatabase = event.target()
                            .unwrap()
                            .dyn_into::<IdbOpenDbRequest>()
                            .unwrap()
                            .result()
                            .unwrap()
                            .dyn_into()
                            .unwrap();
                        
                        if !db.object_store_names().contains("documents") {
                            db.create_object_store("documents")
                                .expect("Failed to create object store");
                        }
                    }));

                    let onsuccess = Closure::once(Box::new(move |event: web_sys::Event| {
                        let db: IdbDatabase = event.target()
                            .unwrap()
                            .dyn_into::<IdbOpenDbRequest>()
                            .unwrap()
                            .result()
                            .unwrap()
                            .dyn_into()
                            .unwrap();
                        resolve.call1(&JsValue::undefined(), &db).unwrap();
                    }));

                    let onerror = Closure::once(Box::new(move |e: web_sys::Event| {
                        reject.call1(&JsValue::undefined(), &e).unwrap();
                    }));

                    db_request.set_onupgradeneeded(Some(onupgradeneeded.as_ref().unchecked_ref()));
                    db_request.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
                    db_request.set_onerror(Some(onerror.as_ref().unchecked_ref()));

                    onupgradeneeded.forget();
                    onsuccess.forget();
                    onerror.forget();
                })).await?;

                // Store new connection in pool
                let db = Arc::new(db.dyn_into::<IdbDatabase>()?);
                POOL.store_connection(name.to_string(), Arc::downgrade(&db));
                db
            }
        };

        Ok(IndexDB {
            base,
            core: CoreStorage {},
            db: (*db).clone(),
            _error_handler: None,
            _success_handler: None,
        })
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
        let store_name = "documents";
        let transaction = self.db.transaction_with_str_and_mode(
            store_name,
            web_sys::IdbTransactionMode::Readwrite,
        )?;
        let store = transaction.object_store(store_name)?;

        match op.op_type {
            OpType::CREATE | OpType::UPDATE => {
                let document = op.data.clone();
                let pk_value = Reflect::get(&document, &JsValue::from_str(&self.base.schema.primary_key))?;
                
                // Validate and store
                self.base.schema.validate_schema(document.clone())?;
                let request = store.put_with_key(&document, &pk_value)?;
                
                let promise = self.create_request_promise(request);
                JsFuture::from(promise).await.map(|_| document)
            },
            OpType::DELETE => {
                let request = store.delete(&op.data)?;
                let promise = self.create_request_promise(request);
                JsFuture::from(promise).await
            },
            _ => Err(JsValue::from_str("Unsupported operation type")),
        }
    }

    // Helper method to create request promises
    fn create_request_promise(&self, request: IdbRequest) -> Promise {
        Promise::new(&mut |resolve, reject| {
            let success_handler = Closure::once(Box::new(move |event: web_sys::Event| {
                let request: IdbRequest = event.target().unwrap().dyn_into().unwrap();
                resolve.call1(&JsValue::undefined(), &request.result().unwrap()).unwrap();
            }));
            
            let error_handler = Closure::once(Box::new(move |e: web_sys::Event| {
                reject.call1(&JsValue::undefined(), &e).unwrap();
            }));
            
            request.set_onsuccess(Some(success_handler.as_ref().unchecked_ref()));
            request.set_onerror(Some(error_handler.as_ref().unchecked_ref()));
            
            success_handler.forget();
            error_handler.forget();
        })
    }

    #[wasm_bindgen(js_name = "find")]
    pub async fn find_js(&self, collection_name: &str, query: JsValue) -> Result<JsValue, JsValue> {
        self.find(collection_name, Query::new(query, self.schema())?).await
    }

    #[wasm_bindgen(js_name = "findDocumentById")]
    pub async fn find_document_by_id_js(&self, collection_name: &str, primary_key: JsValue) -> Result<JsValue, JsValue> {
        self.find_document_by_id(collection_name, primary_key).await
    }

    #[wasm_bindgen(js_name = "count")]
    pub async fn count_js(&self, collection_name: &str, query: JsValue) -> Result<JsValue, JsValue> {
        self.count(collection_name, Query::new(query, self.schema())?).await
    }

    #[wasm_bindgen(js_name = "close")]
    pub async fn close_js(&self) -> Result<JsValue, JsValue> {
        self.close().await
    }
}
// Global connection pool
lazy_static! {
    static ref POOL: IndexDBPool = IndexDBPool::new();
}

// Add these trait implementations before the IndexDBPool struct
unsafe impl Send for IndexDBPool {}
unsafe impl Sync for IndexDBPool {}

pub struct IndexDBPool {
    connections: Arc<Mutex<HashMap<String, Arc<IdbDatabase>>>>,
}

impl IndexDBPool {
    fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    fn get_connection(&self, name: &str) -> Option<Arc<IdbDatabase>> {
        let connections = self.connections.lock();
        if let Some(db) = connections.get(name) {
            Some(db.clone())
        } else {
            None
        }
    }

    fn store_connection(&self, name: String, db: Weak<IdbDatabase>) {
        let mut connections = self.connections.lock();
        if let Some(arc_db) = db.upgrade() {
            connections.insert(name, arc_db);
        }
    }
}


/* 
#[cfg(test)]
mod tests {
    use super::*;
    use js_sys::Object;
    use wasm_bindgen_test::*;
    use serde_json::Value;
    use wasm_bindgen::JsCast;

    // Configure tests to run in browser
    wasm_bindgen_test_configure!(run_in_browser);

    // Helper function to ensure window is available
    fn setup_window() -> web_sys::Window {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::window().expect("no global `window` exists")
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            panic!("Tests must be run in a browser environment");
        }
    }

    // Helper function to clean up IndexDB between tests
    async fn cleanup_db(name: &str) -> Result<(), JsValue> {
        let window = setup_window();
        let idb = window.indexed_db()?.expect("IndexDB not available");
        let delete_req = idb.delete_database(name)?;
        
        let promise = Promise::new(&mut |resolve, reject| {
            let onsuccess = Closure::once(Box::new(move |event: web_sys::Event| {
                resolve.call0(&JsValue::undefined()).unwrap();
            }));
            
            let onerror = Closure::once(Box::new(move |e: web_sys::Event| {
                reject.call1(&JsValue::undefined(), &e).unwrap();
            }));
            
            delete_req.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
            delete_req.set_onerror(Some(onerror.as_ref().unchecked_ref()));
            onsuccess.forget();
            onerror.forget();
        });

        JsFuture::from(promise).await?;
        Ok(())
    }

    fn json_str_to_js_value(json_str: &str) -> Result<JsValue, JsValue> {
        let json_value: Value =
            serde_json::from_str(json_str).map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(value_to_js_value(&json_value))
    }

    fn value_to_js_value(value: &Value) -> JsValue {
        match value {
            Value::Null => JsValue::null(),
            Value::Bool(b) => JsValue::from_bool(*b),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    JsValue::from_f64(i as f64)
                } else if let Some(f) = n.as_f64() {
                    JsValue::from_f64(f)
                } else {
                    JsValue::undefined()
                }
            }
            Value::String(s) => JsValue::from_str(s),
            Value::Array(arr) => {
                let js_array = Array::new();
                for item in arr {
                    js_array.push(&value_to_js_value(item));
                }
                js_array.into()
            }
            Value::Object(obj) => {
                let js_obj = Object::new();
                for (key, value) in obj {
                    js_sys::Reflect::set(
                        &js_obj,
                        &JsValue::from_str(key),
                        &value_to_js_value(value),
                    )
                        .unwrap();
                }
                js_obj.into()
            }
        }
    }

    #[wasm_bindgen_test(async)]
    async fn test_empty_indexdb_storage() {
        let schema_str = "{ \"version\": 1, \"primaryKey\": \"id\", \"type\": \"object\", \"properties\": { \"id\": { \"type\": \"string\", \"maxLength\": 60 } } }";
        let schema_name = "test_empty_db";
        
        // Clean up any existing database
        let _ = cleanup_db(schema_name).await;
        
        let schema = json_str_to_js_value(schema_str).unwrap();
        let migrations = json_str_to_js_value("{}").unwrap();
        let indexdb = IndexDB::create(schema_name, schema, migrations).await;
        assert!(indexdb.is_ok());
    }

    #[wasm_bindgen_test(async)]
    async fn test_indexdb_storage_write() {
        let schema_str = r#"{
            "version": 1,
            "primaryKey": "id",
            "type": "object",
            "properties": {
                "id": { "type": "string", "maxLength": 60 },
                "name": { "type": "string" }
            }
        }"#;
        let schema_name = "test_write_db";
        
        // Clean up any existing database
        let _ = cleanup_db(schema_name).await;
        
        let schema = json_str_to_js_value(schema_str).unwrap();
        let migrations = json_str_to_js_value("{}").unwrap();

        let mut indexdb = IndexDB::create(schema_name, schema, migrations).await.unwrap();

        // Create a new item
        let new_item = Object::new();
        Reflect::set(&new_item, &JsValue::from_str("id"), &JsValue::from_str("1234")).unwrap();
        Reflect::set(&new_item, &JsValue::from_str("name"), &JsValue::from_str("Test Item")).unwrap();

        let op = Operation {
            collection: schema_name.to_string(),
            op_type: OpType::CREATE,
            data: new_item.into(),
            indexes: vec![],
        };

        let created = indexdb.write(&op).await.unwrap();
        assert_eq!(
            Reflect::get(&created, &JsValue::from_str("id")).unwrap(),
            JsValue::from_str("1234")
        );

        // Try to retrieve the document
        let found = indexdb
            .find_document_by_id(JsValue::from_str("1234"))
            .await
            .unwrap();

        assert_eq!(
            Reflect::get(&found, &JsValue::from_str("name")).unwrap(),
            JsValue::from_str("Test Item")
        );
    }

    #[wasm_bindgen_test(async)]
    async fn test_indexdb_storage_update_operation() {
        let schema_name = "test_update_db";
        let _ = cleanup_db(schema_name).await;
        
        let schema_str = r#"{
            "version": 1,
            "primaryKey": "id",
            "type": "object",
            "properties": {
                "id": { "type": "string", "maxLength": 60 },
                "name": { "type": "string" }
            }
        }"#;
        let schema = json_str_to_js_value(schema_str).unwrap();
        let migrations = json_str_to_js_value("{}").unwrap();

        let mut indexdb = IndexDB::create(schema_name, schema, migrations).await.unwrap();

        // Create initial item
        let new_item = Object::new();
        Reflect::set(&new_item, &JsValue::from_str("id"), &JsValue::from_str("1234")).unwrap();
        Reflect::set(&new_item, &JsValue::from_str("name"), &JsValue::from_str("Test Item")).unwrap();

        let create_op = Operation {
            collection: schema_name.to_string(),
            op_type: OpType::CREATE,
            data: new_item.into(),
            indexes: vec![],
        };

        indexdb.write(&create_op).await.unwrap();

        // Update the item
        let updated_item = Object::new();
        Reflect::set(&updated_item, &JsValue::from_str("id"), &JsValue::from_str("1234")).unwrap();
        Reflect::set(&updated_item, &JsValue::from_str("name"), &JsValue::from_str("Updated Item")).unwrap();

        let update_op = Operation {
            collection: schema_name.to_string(),
            op_type: OpType::UPDATE,
            data: updated_item.into(),
            indexes: vec![],
        };

        let updated = indexdb.write(&update_op).await.unwrap();
        assert_eq!(
            Reflect::get(&updated, &JsValue::from_str("name")).unwrap(),
            JsValue::from_str("Updated Item")
        );

        // Verify update
        let found = indexdb
            .find_document_by_id(JsValue::from_str("1234"))
            .await
            .unwrap();
        assert_eq!(
            Reflect::get(&found, &JsValue::from_str("name")).unwrap(),
            JsValue::from_str("Updated Item")
        );
    }

    #[wasm_bindgen_test(async)]
    async fn test_indexdb_storage_delete_operation() {
        let schema_name = "test_delete_db";
        let _ = cleanup_db(schema_name).await;
        
        let schema_str = r#"{
            "version": 1,
            "primaryKey": "id",
            "type": "object",
            "properties": {
                "id": { "type": "string", "maxLength": 60 },
                "name": { "type": "string" }
            }
        }"#;
        let schema = json_str_to_js_value(schema_str).unwrap();
        let migrations = json_str_to_js_value("{}").unwrap();

        let mut indexdb = IndexDB::create(schema_name, schema, migrations).await.unwrap();

        // Create initial item
        let new_item = Object::new();
        Reflect::set(&new_item, &JsValue::from_str("id"), &JsValue::from_str("1234")).unwrap();
        Reflect::set(&new_item, &JsValue::from_str("name"), &JsValue::from_str("Test Item")).unwrap();

        let create_op = Operation {
            collection: schema_name.to_string(),
            op_type: OpType::CREATE,
            data: new_item.into(),
            indexes: vec![],
        };

        indexdb.write(&create_op).await.unwrap();

        // Delete the item
        let delete_op = Operation {
            collection: schema_name.to_string(),
            op_type: OpType::DELETE,
            data: JsValue::from_str("1234"),
            indexes: vec![],
        };

        let delete_result = indexdb.write(&delete_op).await.unwrap();
        assert_eq!(delete_result, JsValue::from_str("Document deleted"));

        // Verify deletion
        let find_result = indexdb.find_document_by_id(JsValue::from_str("1234")).await;
        assert!(find_result.is_err());
    }

    #[wasm_bindgen_test(async)]
    async fn test_indexdb_storage_find() {
        let schema_name = "test_find_db";
        let _ = cleanup_db(schema_name).await;
        
        let schema_str = r#"{
            "version": 1,
            "primaryKey": "id",
            "type": "object",
            "properties": {
                "id": { "type": "string", "maxLength": 60 },
                "name": { "type": "string" },
                "age": { "type": "number" },
                "status": { "type": "string" }
            }
        }"#;
        let schema = json_str_to_js_value(schema_str).unwrap();
        let migrations = json_str_to_js_value("{}").unwrap();
        let mut indexdb = IndexDB::create(schema_name, schema, migrations).await.unwrap();

        // Create test items
        let items = vec![
            json_str_to_js_value(r#"{
                "id": "1",
                "name": "Alice",
                "age": 30,
                "status": "active"
            }"#).unwrap(),
            json_str_to_js_value(r#"{
                "id": "2",
                "name": "Bob",
                "age": 25,
                "status": "inactive"
            }"#).unwrap(),
            json_str_to_js_value(r#"{
                "id": "3",
                "name": "Charlie",
                "age": 35,
                "status": "active"
            }"#).unwrap(),
        ];

        for item in items {
            let create_op = Operation {
                collection: schema_name.to_string(),
                op_type: OpType::CREATE,
                data: item,
                indexes: vec![],
            };
            indexdb.write(&create_op).await.unwrap();
        }

        // Test query
        let query_value = json_str_to_js_value(r#"{
            "status": "active",
            "age": { "$gt": 30 }
        }"#).unwrap();
        let query = Query::new(query_value, indexdb.schema()).unwrap();
        let result = indexdb.find(query).await.unwrap();

        let result_array = Array::from(&result);
        assert_eq!(result_array.length(), 1);
        let first_doc = result_array.get(0);
        assert_eq!(
            Reflect::get(&first_doc, &JsValue::from_str("name")).unwrap(),
            JsValue::from_str("Charlie")
        );
    }

    #[wasm_bindgen_test(async)]
    async fn test_indexdb_storage_count() {
        let schema_name = "test_count_db";
        let _ = cleanup_db(schema_name).await;
        
        let schema_str = r#"{
            "version": 1,
            "primaryKey": "id",
            "type": "object",
            "properties": {
                "id": { "type": "string", "maxLength": 60 },
                "name": { "type": "string" },
                "age": { "type": "number" },
                "status": { "type": "string" }
            }
        }"#;
        let schema = json_str_to_js_value(schema_str).unwrap();
        let migrations = json_str_to_js_value("{}").unwrap();
        let mut indexdb = IndexDB::create(schema_name, schema, migrations).await.unwrap();

        // Create test items
        let items = vec![
            json_str_to_js_value(r#"{
                "id": "1",
                "name": "Alice",
                "age": 30,
                "status": "active"
            }"#).unwrap(),
            json_str_to_js_value(r#"{
                "id": "2",
                "name": "Bob",
                "age": 25,
                "status": "inactive"
            }"#).unwrap(),
            json_str_to_js_value(r#"{
                "id": "3",
                "name": "Charlie",
                "age": 35,
                "status": "active"
            }"#).unwrap(),
        ];

        for item in items {
            let create_op = Operation {
                collection: schema_name.to_string(),
                op_type: OpType::CREATE,
                data: item,
                indexes: vec![],
            };
            indexdb.write(&create_op).await.unwrap();
        }

        // Test count query
        let query_value = json_str_to_js_value(r#"{
            "status": "active"
        }"#).unwrap();
        let query = Query::new(query_value, indexdb.schema()).unwrap();
        let result = indexdb.count(query).await.unwrap();

        assert_eq!(result.as_f64().unwrap(), 2.0);
    }
}

    */