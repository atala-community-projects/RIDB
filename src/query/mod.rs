use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use crate::error::RIDBError;

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
/**
 * Represents an operation to be performed on a collection.
 *
 * @template T - The schema type of the collection.
 */
export type Operation<T extends SchemaType> = {
    /**
     * The name of the collection on which the operation will be performed.
     */
    collection: string,

    /**
     * The type of operation to be performed (e.g., CREATE, UPDATE, DELETE).
     */
    opType: OpType,

    /**
     * The data involved in the operation, conforming to the schema type.
     */
    data: Doc<T>,

    /**
     * An array of indexes related to the operation.
     */
    indexes: Array<string>
}
"#;

#[derive(Debug, Clone)]
#[wasm_bindgen]
/// Represents the type of operation to be performed on the collection.
pub enum OpType {
    /// Create operation.
    CREATE,
    /// Update operation.
    UPDATE,
    /// Delete operation.
    DELETE
}

#[derive(Debug, Clone)]
#[wasm_bindgen(skip_typescript)]
/// Represents an operation to be performed on a collection.
pub struct Operation {
    /// The name of the collection on which the operation will be performed.
    pub(crate) collection: String,
    /// The type of operation (create, update, delete).
    pub(crate) op_type: OpType,
    /// The data involved in the operation.
    pub(crate) data: JsValue,
    /// The indexes related to the operation.
    pub(crate) indexes: Vec<String>
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyValClause {
    pub key: String,
    pub value: Value
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpressionTreeClause {
    pub operator: String,
    pub expressions: Vec<Expression>
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Clause {
    KeyVal(KeyValClause),
    Expression(ExpressionTreeClause),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Leaf(LeafValue),
    Operators(Vec<Operator>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Operator {
    List(ListOperator),
    Value(ValueOperator),
    ExpressionOperator(OperatorExpressionOperator),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListOperator {
    pub operator: String,
    pub values: Vec<LeafValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValueOperator {
    pub operator: String,
    pub value: LeafValue,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperatorExpressionOperator {
    pub operator: String,
    pub operators: Vec<Operator>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeafValue {
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Expression {
    pub clauses: Vec<Clause>
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct Query {
    pub(crate) expression: Expression
}

#[wasm_bindgen]
impl Query {

    #[wasm_bindgen(constructor)]
    pub fn from() -> Query {
        todo!()
    }


}


#[wasm_bindgen]
impl Operation {

    /// Retrieves the name of the collection.
    ///
    /// # Returns
    ///
    /// * `String` - The name of the collection.
    #[wasm_bindgen(getter)]
    pub fn collection(&self) -> String {
        self.collection.clone()
    }

    /// Retrieves the type of operation.
    ///
    /// # Returns
    ///
    /// * `OpType` - The type of operation.
    #[wasm_bindgen(getter, js_name="opType")]
    pub fn op_type(&self) -> OpType {
        self.op_type.clone()
    }

    /// Retrieves the data involved in the operation.
    ///
    /// # Returns
    ///
    /// * `JsValue` - The data involved in the operation.
    #[wasm_bindgen(getter)]
    pub fn data(&self) -> JsValue {
        self.data.clone()
    }

    /// Retrieves the indexes related to the operation.
    ///
    /// # Returns
    ///
    /// * `Result<JsValue, JsValue>` - A result containing the indexes as a `JsValue` or an error.
    #[wasm_bindgen(getter)]
    pub fn indexes(&self) -> Result<JsValue, JsValue> {
        to_value(&self.indexes.clone())
            .map_err(|e| JsValue::from(RIDBError::from(e)))
    }
}