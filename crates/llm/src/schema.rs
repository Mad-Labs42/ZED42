//! JSON schema builder for constrained generation

use serde_json::{json, Value};

/// JSON schema for constraining LLM output
#[derive(Debug, Clone)]
pub struct JsonSchema {
    schema: Value,
}

impl JsonSchema {
    /// Create a new schema builder
    pub fn builder() -> SchemaBuilder {
        SchemaBuilder::new()
    }

    /// Get the raw schema
    pub fn as_value(&self) -> &Value {
        &self.schema
    }
}

/// Builder for JSON schemas
#[derive(Debug)]
pub struct SchemaBuilder {
    schema: Value,
}

impl SchemaBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    /// Add a string field
    pub fn add_string(mut self, name: &str, description: &str, required: bool) -> Self {
        self.add_property(name, json!({
            "type": "string",
            "description": description
        }), required);
        self
    }

    /// Add a number field
    pub fn add_number(mut self, name: &str, description: &str, required: bool) -> Self {
        self.add_property(name, json!({
            "type": "number",
            "description": description
        }), required);
        self
    }

    /// Add a boolean field
    pub fn add_boolean(mut self, name: &str, description: &str, required: bool) -> Self {
        self.add_property(name, json!({
            "type": "boolean",
            "description": description
        }), required);
        self
    }

    /// Add an array field
    pub fn add_array(
        mut self,
        name: &str,
        description: &str,
        item_type: &str,
        required: bool,
    ) -> Self {
        self.add_property(name, json!({
            "type": "array",
            "description": description,
            "items": {
                "type": item_type
            }
        }), required);
        self
    }

    /// Add an enum field
    pub fn add_enum(
        mut self,
        name: &str,
        description: &str,
        values: Vec<&str>,
        required: bool,
    ) -> Self {
        self.add_property(name, json!({
            "type": "string",
            "description": description,
            "enum": values
        }), required);
        self
    }

    /// Add a nested object field
    pub fn add_object(mut self, name: &str, schema: JsonSchema, required: bool) -> Self {
        self.add_property(name, schema.schema, required);
        self
    }

    /// Build the schema
    pub fn build(self) -> JsonSchema {
        JsonSchema {
            schema: self.schema,
        }
    }

    fn add_property(&mut self, name: &str, property: Value, required: bool) {
        if let Some(properties) = self.schema.get_mut("properties") {
            properties
                .as_object_mut()
                .unwrap()
                .insert(name.to_string(), property);
        }

        if required {
            if let Some(required_arr) = self.schema.get_mut("required") {
                required_arr
                    .as_array_mut()
                    .unwrap()
                    .push(json!(name));
            }
        }
    }
}

impl Default for SchemaBuilder {
    fn default() -> Self {
        Self::new()
    }
}
