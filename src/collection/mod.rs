use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use crate::error::RIDBError;
use crate::schema::Schema;
use crate::storage::internals::Internals;

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
/**
 * ExtractType is a utility type that maps a string representing a basic data type to the actual TypeScript type.
 *
 * @template T - A string literal type representing the basic data type ('string', 'number', 'boolean', 'object', 'array').
 *
 * @example
 * type StringType = ExtractType<'string'>; // StringType is string
 * type NumberType = ExtractType<'number'>; // NumberType is number
 * type BooleanType = ExtractType<'boolean'>; // BooleanType is boolean
 * type ObjectType = ExtractType<'object'>; // ObjectType is object
 * type ArrayType = ExtractType<'array'>; // ArrayType is Array<any>
 */
export type ExtractType<T extends string> = T extends 'string' ? string :
    T extends 'number' ? number :
    T extends 'boolean' ? boolean :
    T extends 'object' ? object :
    T extends 'array' ? Array<any> :
    never;

/**
 * Doc is a utility type that transforms a schema type into a document type where each property is mapped to its extracted type.
 *
 * @template T - A schema type with a 'properties' field where each property's type is represented as a string.
 *
 * type Document = Doc<Schema>; // Document is { name: string; age: number; }
 */
export type Doc<T extends SchemaType> = {
    [name in keyof T['properties']]: ExtractType<T['properties'][name]['type']>
}

/**
 * Collection is a class that represents a collection of documents in a database.
 *
 * @template T - A schema type defining the structure of the documents in the collection.
 */
export class Collection<T extends SchemaType> {
    /**
     * Creates a new Collection instance from a given schema.
     *
     * @template TS - A schema type. Defaults to SchemaType.
     * @param name - The name of the collection.
     * @param schema - The schema defining the structure of the documents in the collection.
     * @returns A new Collection instance.
     */
    static from<
        TS extends SchemaType = SchemaType
    >(name: string, schema: TS): Collection<TS>

    schema: T;

    /**
     * Constructs a new Collection instance.
     *
     * @param name - The name of the collection.
     * @param schema - The schema defining the structure of the documents in the collection.
     */
    constructor(name: string, schema: T);

    /**
     * Finds all documents in the collection.
     *
     * @returns A promise that resolves to an array of documents.
     */
    find(): Promise< Doc<T>[]>;

    /**
     * Finds a single document in the collection by its ID.
     *
     * @param id - The ID of the document to find.
     * @returns A promise that resolves to the found document.
     */
    findById(id: string): Promise<Doc<T>>;

    /**
     * Updates a document in the collection by its ID.
     *
     * @param id - The ID of the document to update.
     * @param document - A partial document containing the fields to update.
     * @returns A promise that resolves when the update is complete.
     */
    update(id: string, document: Partial<Doc<T>>): Promise<void>;

    /**
     * Creates a new document in the collection.
     *
     * @param document - The document to create.
     * @returns A promise that resolves to the created document.
     */
    create(document: Doc<T>): Promise<Doc<T>>;

    /**
     * Deletes a document in the collection by its ID.
     *
     * @param id - The ID of the document to delete.
     * @returns A promise that resolves when the deletion is complete.
     */
    delete(id: string): Promise<void>;
}

"#;

#[wasm_bindgen(skip_typescript)]
#[derive(Clone, Deserialize, Serialize)]
pub struct Collection {
    #[serde(skip_serializing, skip_deserializing)]
    pub(crate) name: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub(crate) internals: Internals
}

#[wasm_bindgen]
impl Collection {

    /// Constructs a new `Collection` with the given name and internals.
    ///
    /// # Arguments
    ///
    /// * `name` - A string representing the name of the collection.
    /// * `internals` - Internal storage mechanisms for the collection.
    pub fn from(name: String, internals: Internals) -> Collection {
        Collection {
            name,
            internals
        }
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn schema(&self) -> Schema {
        self.internals.schema.clone()
    }

    /// Finds and returns all documents in the collection.
    ///
    /// This function is asynchronous and returns a `Schema` representing
    /// the documents found in the collection.
    #[wasm_bindgen]
    pub async fn find(&self) -> Schema {
        todo!()
    }

    /// Finds and returns a single document in the collection by its ID.
    ///
    /// This function is asynchronous.
    #[wasm_bindgen(js_name="findById")]
    pub async fn find_by_id(&self, primary_key: JsValue) -> Result<JsValue, JsValue>{
        self.internals.find_document_by_id(primary_key).await
    }

    /// Updates a document in the collection with the given data.
    ///
    /// This function is asynchronous and returns a `Result` indicating success or failure.
    ///
    /// # Arguments
    ///
    /// * `document` - A `JsValue` representing the partial document to update.
    #[wasm_bindgen]
    pub async fn update(&self, _document: JsValue) -> Result<JsValue, JsValue> {
        //write operation to update a single entry
        todo!()
    }

    /// Creates a new document in the collection.
    ///
    /// This function is asynchronous and returns a `Result` indicating success or failure.
    ///
    /// # Arguments
    ///
    /// * `document` - A `JsValue` representing the document to create.
    #[wasm_bindgen]
    pub async fn create(&self, document: JsValue) -> Result<JsValue, JsValue> {
        let result = self.internals.write(document).await;
        result.map_err(|e| JsValue::from(RIDBError::from(e)))
    }

    /// Deletes a document from the collection by its ID.
    ///
    /// This function is asynchronous.
    #[wasm_bindgen]
    pub async fn delete(&self, primary_key: JsValue) {
        //write operation to remove a single entry
        todo!()
    }
}
