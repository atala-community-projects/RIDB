use std::collections::HashMap;
use js_sys::{Array, Object, Reflect};
use serde_json::Value;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use crate::query::{ Query};
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
pub struct InMemory {
    pub(crate) base: BaseStorage,
    pub(crate) by_index: HashMap<String, HashMap<String, JsValue>>,
}

impl InMemory {

    fn document_matches_query(&self, document: &JsValue, query: &JsValue) -> Result<bool, JsValue> {
        // Ensure query is an object
        if !query.is_object() {
            return Err(JsValue::from_str("Query must be an object"));
        }

        let keys = Object::keys(&Object::from(query.clone()));

        for i in 0..keys.length() {
            let key = keys.get(i).as_string().unwrap_or_default();
            let value = Reflect::get(query, &JsValue::from_str(&key))?;

            if key == "$and" {
                // $and operator: all conditions must be true
                if !Array::is_array(&value) {
                    return Err(JsValue::from_str("$and must be an array"));
                }
                let arr = Array::from(&value);
                for j in 0..arr.length() {
                    let item = arr.get(j);
                    let matches = self.document_matches_query(document, &item)?;
                    if !matches {
                        return Ok(false);
                    }
                }
                return Ok(true);
            } else if key == "$or" {
                // $or operator: at least one condition must be true
                if !Array::is_array(&value) {
                    return Err(JsValue::from_str("$or must be an array"));
                }
                let arr = Array::from(&value);
                for j in 0..arr.length() {
                    let item = arr.get(j);
                    let matches = self.document_matches_query(document, &item)?;
                    if matches {
                        return Ok(true);
                    }
                }
                return Ok(false);
            } else {
                // Attribute condition
                let doc_value = Reflect::get(document, &JsValue::from_str(&key))?;
                let matches = self.evaluate_condition(&doc_value, &value)?;
                if !matches {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    fn evaluate_condition(&self, doc_value: &JsValue, condition: &JsValue) -> Result<bool, JsValue> {
        if condition.is_object() && !Array::is_array(condition) {
            // Condition is an object with operators
            let keys = Object::keys(&Object::from(condition.clone()));
            for i in 0..keys.length() {
                let key = keys.get(i).as_string().unwrap_or_default();
                let value = Reflect::get(condition, &JsValue::from_str(&key))?;
                match key.as_str() {
                    "$gt" => {
                        let res = self.compare_values(doc_value, &value, |a:f64, b:f64| a > b)?;
                        if !res {
                            return Ok(false);
                        }
                    }
                    "$gte" => {
                        let res = self.compare_values(doc_value, &value, |a:f64, b:f64| a >= b)?;
                        if !res {
                            return Ok(false);
                        }
                    }
                    "$lt" => {
                        let res = self.compare_values(doc_value, &value, |a:f64, b:f64| a < b)?;
                        if !res {
                            return Ok(false);
                        }
                    }
                    "$lte" => {
                        let res = self.compare_values(doc_value, &value, |a:f64, b:f64| a <= b)?;
                        if !res {
                            return Ok(false);
                        }
                    }
                    "$in" => {
                        if !Array::is_array(&value) {
                            return Err(JsValue::from_str("$in value must be an array"));
                        }
                        let arr = Array::from(&value);
                        let mut found = false;
                        for j in 0..arr.length() {
                            let item = arr.get(j);
                            if self.values_equal(doc_value, &item)? {
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            return Ok(false);
                        }
                    }
                    _ => {
                        return Err(JsValue::from_str(&format!("Unsupported operator: {}", key)));
                    }
                }
            }
            Ok(true)
        } else {
            // Direct value comparison
            self.values_equal(doc_value, condition)
        }
    }

    fn compare_values<F>(
        &self,
        doc_value: &JsValue,
        cond_value: &JsValue,
        cmp: F,
    ) -> Result<bool, JsValue>
    where
        F: Fn(f64, f64) -> bool,
    {
        let doc_num = doc_value
            .as_f64()
            .ok_or_else(|| JsValue::from_str("Document value is not a number"))?;
        let cond_num = cond_value
            .as_f64()
            .ok_or_else(|| JsValue::from_str("Condition value is not a number"))?;
        Ok(cmp(doc_num, cond_num))
    }

    fn values_equal(&self, doc_value: &JsValue, cond_value: &JsValue) -> Result<bool, JsValue> {
        if doc_value.is_string() && cond_value.is_string() {
            Ok(doc_value.as_string() == cond_value.as_string())
        } else if doc_value.as_f64().is_some() {
            Ok(doc_value.as_f64() == cond_value.as_f64())
        } else if doc_value.is_truthy() || cond_value.is_falsy() {
            Ok(doc_value.as_bool() == cond_value.as_bool())
        } else {
            Ok(false)
        }
    }

}

impl StorageBase for InMemory {
    async fn write(&mut self, op: &Operation) -> Result<JsValue, JsValue> {
        let primary_key = self.base.schema.primary_key.clone();
        let index_name = format!("pk_{}", primary_key);

        // Get or create the primary key index
        let index = self
            .by_index
            .entry(index_name.clone())
            .or_insert_with(HashMap::new);

        match op.op_type {
            OpType::CREATE | OpType::UPDATE => {
                // op.data is an object containing the document
                let document = op.data.clone();

                // Step 1: Extract the primary key value from the document
                let pk_value = Reflect::get(&document, &JsValue::from_str(&primary_key))
                    .map_err(|e| {
                        JsValue::from_str(&format!("Failed to get primary key: {:?}", e))
                    })?;

                if pk_value.is_undefined() || pk_value.is_null() {
                    return Err(JsValue::from_str("Document must contain a primary key"));
                }

                // Convert primary key value to string
                let pk_str = if let Some(s) = pk_value.as_string() {
                    s
                } else if let Some(n) = pk_value.as_f64() {
                    n.to_string()
                } else {
                    return Err(JsValue::from_str("Primary key must be a string or number"));
                };


                // Handle the operation
                match op.op_type {
                    OpType::CREATE => {
                        // Validate the document against the schema
                        self.base.schema.validate_schema(document.clone())?;
                        if index.contains_key(&pk_str) {

                            return Err(JsValue::from_str(
                                "Document with this primary key already exists",
                            ));
                        }
                        index.insert(pk_str.clone(), document.clone());
                        Ok(document)
                    }
                    OpType::UPDATE => {
                        // Validate the document against the schema
                        self.base.schema.validate_schema(document.clone())?;
                        if !index.contains_key(&pk_str) {

                            return Err(JsValue::from_str(
                                "Document with this primary key does not exist",
                            ));
                        }
                        index.insert(pk_str.clone(), document.clone());
                        Ok(document)
                    }
                    _ => {
                        Err(JsValue::from_str(
                            "Unsupported operation type for this data",
                        ))
                    }
                }
            }
            OpType::DELETE => {
                // op.data is a string (the primary key value)
                let pk_value = op.data.clone();

                if pk_value.is_undefined() || pk_value.is_null() {

                    return Err(JsValue::from_str(
                        "Primary key value is required for delete operation",
                    ));
                }

                // Convert primary key value to string
                let pk_str = if let Some(s) = pk_value.as_string() {
                    s
                } else if let Some(n) = pk_value.as_f64() {
                    n.to_string()
                } else {

                    return Err(JsValue::from_str(
                        "Primary key must be a string or number",
                    ));
                };


                // Handle the delete operation
                if index.remove(&pk_str).is_some() {
                    Ok(JsValue::from_str("Document deleted"))
                } else {
                    Err(JsValue::from_str(
                        "Document with this primary key does not exist",
                    ))
                }
            }
            _ => {
                Err(JsValue::from_str("Unsupported operation type"))
            }
        }
    }

    async fn find(&self, query: Query) -> Result<JsValue, JsValue> {
        // Get the normalized query
        let normalized_query = query.parse()?;

        // Collect matching documents
        let mut results = Array::new();

        // Get all documents from the primary key index
        let primary_key = self.base.schema.primary_key.clone();
        let index_name = format!("pk_{}", primary_key);
        if let Some(index) = self.by_index.get(&index_name) {
            for (_pk, doc) in index.iter() {
                let matches = self.document_matches_query(doc, &normalized_query)?;
                if matches {
                    results.push(doc);
                }
            }
        }

        Ok(results.into())
    }

    async fn count(&self, query: Query) -> Result<JsValue, JsValue> {
        // Get the normalized query
        let normalized_query = query.parse()?;

        // Count matching documents
        let mut count = 0;

        // Get all documents from the primary key index
        let primary_key = self.base.schema.primary_key.clone();
        let index_name = format!("pk_{}", primary_key);
        if let Some(index) = self.by_index.get(&index_name) {
            for (_pk, doc) in index.iter() {
                let matches = self.document_matches_query(doc, &normalized_query)?;
                if matches {
                    count += 1;
                }
            }
        }

        Ok(JsValue::from_f64(count as f64))
    }

    async fn find_document_by_id(
        &self,
        primary_key_value: JsValue,
    ) -> Result<JsValue, JsValue> {
        let primary_key = self.base.schema.primary_key.clone();
        let index_name = format!("pk_{}", primary_key);

        // Convert primary key value to string
        let pk_str = if let Some(s) = primary_key_value.as_string() {
            s
        } else if let Some(n) = primary_key_value.as_f64() {
            n.to_string()
        } else {
            return Err(JsValue::from_str("Invalid primary key value"));
        };


        // Retrieve the index
        if let Some(index) = self.by_index.get(&index_name) {
            if let Some(doc) = index.get(&pk_str) {
                return Ok(doc.clone());
            }
        }

        Err(JsValue::from_str("Document not found"))
    }


    async fn close(&self) -> Result<JsValue, JsValue> {
        Ok(JsValue::from_str("In-memory database closed"))
    }
}


#[wasm_bindgen]
impl InMemory {
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str, schema_type: JsValue) -> Result<InMemory, JsValue> {
        let base_res = BaseStorage::new(name.to_string(), schema_type);
        match base_res {
            Ok(base) => {
                Ok(InMemory {
                    base,
                    by_index: HashMap::new(),
                })
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    #[wasm_bindgen(getter)]
    pub fn by_index(&self) -> Result<JsValue, JsValue> {
        let outer_obj = Object::new();

        for (outer_key, inner_map) in &self.by_index {
            let inner_obj = Object::new();
            for (inner_key, value) in inner_map {
                Reflect::set(&inner_obj, &JsValue::from_str(inner_key), value)
                    .map_err(|_| {
                        JsValue::from_str("Failed to set inner object property")
                    })?;
            }
            Reflect::set(
                &outer_obj,
                &JsValue::from_str(outer_key),
                &JsValue::from(inner_obj),
            ).map_err(|_| {
                JsValue::from_str("Failed to set outer object property")
            })?;
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

    #[wasm_bindgen(js_name = "find")]
    pub async fn find_js(&self, query: JsValue) -> Result<JsValue, JsValue> {
        self.find(Query::new(query, self.schema())?).await
    }

    #[wasm_bindgen(js_name = "findDocumentById")]
    pub async fn find_document_by_id_js(
        &self,
        primary_key: JsValue,
    ) -> Result<JsValue, JsValue> {
        self.find_document_by_id(primary_key).await
    }

    #[wasm_bindgen(js_name = "count")]
    pub async fn count_js(&self, query: JsValue) -> Result<JsValue, JsValue> {
        self.count(Query::new(query, self.schema())?).await
    }

    #[wasm_bindgen(js_name = "close")]
    pub async fn close_js(&self) -> Result<JsValue, JsValue> {
        self.close().await
    }
}


#[cfg(feature = "browser")]
use wasm_bindgen_test::wasm_bindgen_test_configure;
use wasm_bindgen_test::{wasm_bindgen_test};
use crate::operation::{OpType, Operation};

#[cfg(feature = "browser")]
wasm_bindgen_test_configure!(run_in_browser);

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
async fn test_empty_inmemory_storage() {
    let schema_str = "{ \"version\": 0, \"primaryKey\": \"id\", \"type\": \"object\", \"properties\": {  \"id\": { \"type\": \"string\", \"maxLength\": 60 }  } }";
    let schema_name = "demo".to_string();
    let schema = json_str_to_js_value(schema_str).unwrap();
    let inmem = InMemory::new(schema_name.clone().as_str(), schema);
    assert!(inmem.is_ok());
}

#[wasm_bindgen_test(async)]
async fn test_empty_inmemory_storage_write() {
    let schema_str = r#"
    {
        "version": 0,
        "primaryKey": "id",
        "type": "object",
        "properties": {
            "id": { "type": "string", "maxLength": 60 },
            "name": { "type": "string" }
        }
    }
    "#;
    let schema_name = "demo".to_string();
    let schema = json_str_to_js_value(schema_str).unwrap();
    let mut inmem = InMemory::new(&schema_name, schema).unwrap();

    // Create a new item
    let new_item = Object::new();
    Reflect::set(&new_item, &JsValue::from_str("id"), &JsValue::from_str("1234")).unwrap();
    Reflect::set(&new_item, &JsValue::from_str("name"), &JsValue::from_str("Test Item")).unwrap();

    let op = Operation {
        collection: schema_name.clone(),
        op_type: OpType::CREATE,
        data: new_item.into(),
        indexes: vec![],
    };

    let created = inmem.write(&op).await.unwrap();
    assert_eq!(
        Reflect::get(&created, &JsValue::from_str("id")).unwrap(),
        JsValue::from_str("1234")
    );

    // Try to retrieve the document
    let found = inmem
        .find_document_by_id(JsValue::from_str("1234"))
        .await
        .unwrap();

    assert_eq!(
        Reflect::get(&found, &JsValue::from_str("name")).unwrap(),
        JsValue::from_str("Test Item")
    );
}

#[wasm_bindgen_test(async)]
async fn test_inmemory_storage_create_operation() {
    let schema_str = r#"
    {
        "version": 0,
        "primaryKey": "id",
        "type": "object",
        "required":["id", "name"],
        "properties": {
            "id": { "type": "string", "maxLength": 60 },
            "name": { "type": "string" }
        }
    }
    "#;
    let schema_name = "demo".to_string();
    let schema = json_str_to_js_value(schema_str).unwrap();
    let mut inmem = InMemory::new(&schema_name, schema).unwrap();

    // Create a new item
    let new_item = Object::new();
    Reflect::set(&new_item, &JsValue::from_str("id"), &JsValue::from_str("1234")).unwrap();
    Reflect::set(&new_item, &JsValue::from_str("name"), &JsValue::from_str("Test Item")).unwrap();

    let op = Operation {
        collection: schema_name.clone(),
        op_type: OpType::CREATE,
        data: new_item.clone().into(),
        indexes: vec![],
    };

    // Perform the create operation
    let created = inmem.write(&op).await.unwrap();
    assert_eq!(
        Reflect::get(&created, &JsValue::from_str("id")).unwrap(),
        JsValue::from_str("1234")
    );

    // Try to retrieve the document
    let found = inmem
        .find_document_by_id(JsValue::from_str("1234"))
        .await
        .unwrap();
    assert_eq!(
        Reflect::get(&found, &JsValue::from_str("name")).unwrap(),
        JsValue::from_str("Test Item")
    );

    // Check that creating a document with the same primary key fails
    let duplicate_op = Operation {
        collection: schema_name.clone(),
        op_type: OpType::CREATE,
        data: new_item.into(),
        indexes: vec![],
    };

    let duplicate_result = inmem.write(&duplicate_op).await;
    assert!(duplicate_result.is_err());
    assert_eq!(
        duplicate_result.unwrap_err(),
        JsValue::from_str("Document with this primary key already exists")
    );


    // Try to create a document without required fields
    let invalid_item = Object::new();
    Reflect::set(
        &invalid_item,
        &JsValue::from_str("id"),
        &JsValue::from_str("5678341"),
    )
        .unwrap();


    let invalid_op = Operation {
        collection: schema_name.clone(),
        op_type: OpType::CREATE,
        data: invalid_item.into(),
        indexes: vec![],
    };

    let invalid_result = inmem.write(&invalid_op).await;

    assert!(invalid_result.is_err());

}

#[wasm_bindgen_test(async)]
async fn test_inmemory_storage_update_operation() {
    let schema_str = r#"
    {
        "version": 0,
        "primaryKey": "id",
        "type": "object",
        "properties": {
            "id": { "type": "string", "maxLength": 60 },
            "name": { "type": "string" }
        }
    }
    "#;
    let schema_name = "demo".to_string();
    let schema = json_str_to_js_value(schema_str).unwrap();
    let mut inmem = InMemory::new(&schema_name, schema).unwrap();

    // Create a new item
    let new_item = Object::new();
    Reflect::set(&new_item, &JsValue::from_str("id"), &JsValue::from_str("1234")).unwrap();
    Reflect::set(&new_item, &JsValue::from_str("name"), &JsValue::from_str("Test Item")).unwrap();

    let create_op = Operation {
        collection: schema_name.clone(),
        op_type: OpType::CREATE,
        data: new_item.into(),
        indexes: vec![],
    };

    // Perform the create operation
    inmem.write(&create_op).await.unwrap();

    // Update the item
    let updated_item = Object::new();
    Reflect::set(&updated_item, &JsValue::from_str("id"), &JsValue::from_str("1234")).unwrap();
    Reflect::set(
        &updated_item,
        &JsValue::from_str("name"),
        &JsValue::from_str("Updated Item"),
    )
        .unwrap();

    let update_op = Operation {
        collection: schema_name.clone(),
        op_type: OpType::UPDATE,
        data: updated_item.clone().into(),
        indexes: vec![],
    };

    // Perform the update operation
    let updated = inmem.write(&update_op).await.unwrap();
    assert_eq!(
        Reflect::get(&updated, &JsValue::from_str("name")).unwrap(),
        JsValue::from_str("Updated Item")
    );

    // Retrieve the document to verify update
    let found = inmem
        .find_document_by_id(JsValue::from_str("1234"))
        .await
        .unwrap();
    assert_eq!(
        Reflect::get(&found, &JsValue::from_str("name")).unwrap(),
        JsValue::from_str("Updated Item")
    );

    // Try to update a non-existing document
    let non_existing_item = Object::new();
    Reflect::set(&non_existing_item, &JsValue::from_str("id"), &JsValue::from_str("9999")).unwrap();
    Reflect::set(
        &non_existing_item,
        &JsValue::from_str("name"),
        &JsValue::from_str("Non-existing Item"),
    )
        .unwrap();

    let update_non_existing_op = Operation {
        collection: schema_name.clone(),
        op_type: OpType::UPDATE,
        data: non_existing_item.into(),
        indexes: vec![],
    };

    let update_non_existing_result = inmem.write(&update_non_existing_op).await;
    assert!(update_non_existing_result.is_err());
    assert_eq!(
        update_non_existing_result.unwrap_err(),
        JsValue::from_str("Document with this primary key does not exist")
    );
}

#[wasm_bindgen_test(async)]
async fn test_inmemory_storage_delete_operation() {
    let schema_str = r#"
    {
        "version": 0,
        "primaryKey": "id",
        "type": "object",
        "properties": {
            "id": { "type": "string", "maxLength": 60 },
            "name": { "type": "string" }
        }
    }
    "#;
    let schema_name = "demo".to_string();
    let schema = json_str_to_js_value(schema_str).unwrap();
    let mut inmem = InMemory::new(&schema_name, schema).unwrap();

    // Create a new item
    let new_item = Object::new();
    Reflect::set(&new_item, &JsValue::from_str("id"), &JsValue::from_str("1234")).unwrap();
    Reflect::set(&new_item, &JsValue::from_str("name"), &JsValue::from_str("Test Item")).unwrap();

    let create_op = Operation {
        collection: schema_name.clone(),
        op_type: OpType::CREATE,
        data: new_item.into(),
        indexes: vec![],
    };

    // Perform the create operation
    inmem.write(&create_op).await.unwrap();

    // Delete the item by primary key
    let delete_op = Operation {
        collection: schema_name.clone(),
        op_type: OpType::DELETE,
        data: JsValue::from_str("1234"),
        indexes: vec![],
    };

    // Perform the delete operation
    let delete_result = inmem.write(&delete_op).await.unwrap();
    assert_eq!(delete_result, JsValue::from_str("Document deleted"));

    // Try to retrieve the document to verify deletion
    let found_result = inmem.find_document_by_id(JsValue::from_str("1234")).await;
    assert!(found_result.is_err());
    assert_eq!(
        found_result.unwrap_err(),
        JsValue::from_str("Document not found")
    );

    // Try to delete a non-existing document
    let delete_non_existing_op = Operation {
        collection: schema_name.clone(),
        op_type: OpType::DELETE,
        data: JsValue::from_str("9999"),
        indexes: vec![],
    };

    let delete_non_existing_result = inmem.write(&delete_non_existing_op).await;
    assert!(delete_non_existing_result.is_err());
    assert_eq!(
        delete_non_existing_result.unwrap_err(),
        JsValue::from_str("Document with this primary key does not exist")
    );
}


#[wasm_bindgen_test(async)]
async fn test_inmemory_storage_find() {
    let schema_str = r#"{
        "version": 0,
        "primaryKey": "id",
        "type": "object",
        "properties": {
            "id": { "type": "string", "maxLength": 60 },
            "name": { "type": "string" },
            "age": { "type": "number" },
            "status": { "type": "string" }
        }
    }"#;
    let schema_name = "demo".to_string();
    let schema = json_str_to_js_value(schema_str).unwrap();
    let mut inmem = InMemory::new(&schema_name, schema).unwrap();

    // Create items
    let items = vec![
        json_str_to_js_value(r#"{
            "id": "1",
            "name": "Alice",
            "age": 30,
            "status": "active"
        }"#)
            .unwrap(),
        json_str_to_js_value(r#"{
            "id": "2",
            "name": "Bob",
            "age": 25,
            "status": "inactive"
        }"#)
            .unwrap(),
        json_str_to_js_value(r#"{
            "id": "3",
            "name": "Charlie",
            "age": 35,
            "status": "active"
        }"#)
            .unwrap(),
    ];

    for item in items {
        let create_op = Operation {
            collection: schema_name.clone(),
            op_type: OpType::CREATE,
            data: item,
            indexes: vec![],
        };
        inmem.write(&create_op).await.unwrap();
    }

    // Define a query
    let query_value = json_str_to_js_value(r#"{
        "status": "active",
        "age": { "$gt": 30 }
    }"#)
        .unwrap();
    let query = Query::new(query_value, inmem.schema()).unwrap();
    let result = inmem.find(query).await.unwrap();

    // Should return only one document (Charlie)
    let result_array = Array::from(&result);
    assert_eq!(result_array.length(), 1);
    let first_doc = result_array.get(0);
    assert_eq!(
        Reflect::get(&first_doc, &JsValue::from_str("name")).unwrap(),
        JsValue::from_str("Charlie")
    );
}

#[wasm_bindgen_test(async)]
async fn test_inmemory_storage_count() {
    let schema_str = r#"{
        "version": 0,
        "primaryKey": "id",
        "type": "object",
        "properties": {
            "id": { "type": "string", "maxLength": 60 },
            "name": { "type": "string" },
            "age": { "type": "number" },
            "status": { "type": "string" }
        }
    }"#;
    let schema_name = "demo".to_string();
    let schema = json_str_to_js_value(schema_str).unwrap();
    let mut inmem = InMemory::new(&schema_name, schema).unwrap();

    // Create items
    let items = vec![
        json_str_to_js_value(r#"{
            "id": "1",
            "name": "Alice",
            "age": 30,
            "status": "active"
        }"#)
            .unwrap(),
        json_str_to_js_value(r#"{
            "id": "2",
            "name": "Bob",
            "age": 25,
            "status": "inactive"
        }"#)
            .unwrap(),
        json_str_to_js_value(r#"{
            "id": "3",
            "name": "Charlie",
            "age": 35,
            "status": "active"
        }"#)
            .unwrap(),
    ];

    for item in items {
        let create_op = Operation {
            collection: schema_name.clone(),
            op_type: OpType::CREATE,
            data: item,
            indexes: vec![],
        };
        inmem.write(&create_op).await.unwrap();
    }

    // Define a query
    let query_value = json_str_to_js_value(r#"{
        "status": "active"
    }"#)
        .unwrap();
    let query = Query::new(query_value, inmem.schema()).unwrap();
    let result = inmem.count(query).await.unwrap();

    // Should return 2
    assert_eq!(result.as_f64().unwrap(), 2.0);
}

#[wasm_bindgen_test(async)]
async fn test_inmemory_storage_find_with_logical_operators() {
    let schema_str = r#"{
        "version": 0,
        "primaryKey": "id",
        "type": "object",
        "properties": {
            "id": { "type": "string" },
            "name": { "type": "string" },
            "age": { "type": "number" },
            "city": { "type": "string" }
        }
    }"#;
    let schema_name = "demo".to_string();
    let schema = json_str_to_js_value(schema_str).unwrap();
    let mut inmem = InMemory::new(&schema_name, schema).unwrap();

    // Create items
    let items = vec![
        json_str_to_js_value(r#"{
            "id": "1",
            "name": "Alice",
            "age": 30,
            "city": "New York"
        }"#)
            .unwrap(),
        json_str_to_js_value(r#"{
            "id": "2",
            "name": "Bob",
            "age": 22,
            "city": "Los Angeles"
        }"#)
            .unwrap(),
        json_str_to_js_value(r#"{
            "id": "3",
            "name": "Charlie",
            "age": 35,
            "city": "Chicago"
        }"#)
            .unwrap(),
        json_str_to_js_value(r#"{
            "id": "4",
            "name": "Diana",
            "age": 28,
            "city": "New York"
        }"#)
            .unwrap(),
    ];

    for item in items {
        let create_op = Operation {
            collection: schema_name.clone(),
            op_type: OpType::CREATE,
            data: item,
            indexes: vec![],
        };
        inmem.write(&create_op).await.unwrap();
    }

    // Define a complex query
    let query_value = json_str_to_js_value(r#"{
        "$or": [
            { "city": "New York" },
            { "age": { "$lt": 25 } }
        ]
    }"#)
        .unwrap();
    let query = Query::new(query_value, inmem.schema()).unwrap();
    let result = inmem.find(query).await.unwrap();

    // Should return Alice, Bob, and Diana
    let result_array = Array::from(&result);
    assert_eq!(result_array.length(), 3);

    let names: Vec<String> = result_array
        .iter()
        .map(|doc| {
            Reflect::get(&doc, &JsValue::from_str("name"))
                .unwrap()
                .as_string()
                .unwrap()
        })
        .collect();

    assert!(names.contains(&"Alice".to_string()));
    assert!(names.contains(&"Bob".to_string()));
    assert!(names.contains(&"Diana".to_string()));
}
