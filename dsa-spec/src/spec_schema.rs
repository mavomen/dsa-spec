//! JSON Schema (Draft 7) for DSA-SPEC specification files.

/// JSON Schema (Draft 7) defining the required structure of a spec file.
pub const SPEC_JSON_SCHEMA: &str = r##"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "DSA-SPEC Schema",
  "type": "object",
  "required": ["spec_version", "metadata"],
  "definitions": {
    "ParameterizedType": {
      "type": "object",
      "required": ["base"],
      "properties": {
        "base": { "type": "string" },
        "params": {
          "type": "array",
          "items": {
            "oneOf": [
              { "type": "string" },
              { "$ref": "#/definitions/ParameterizedType" }
            ]
          }
        }
      }
    },
    "TypeRef": {
      "oneOf": [
        { "type": "string" },
        { "$ref": "#/definitions/ParameterizedType" }
      ]
    }
  },
  "properties": {
    "spec_version": { "type": "string" },
    "metadata": {
      "type": "object",
      "required": ["name", "category"],
      "properties": {
        "name": {
          "type": "string",
          "minLength": 1
        },
        "category": {
          "type": "string",
          "minLength": 1
        },
        "complexity": {
          "type": "object",
          "properties": {
            "time": { "type": ["string", "null"] },
            "space": { "type": ["string", "null"] }
          }
        },
        "tags": {
          "type": "array",
          "items": { "type": "string" }
        }
      }
    },
    "contracts": {
      "type": "object",
      "properties": {
        "invariants": {
          "type": "array",
          "items": { "type": "string" }
        },
        "preconditions": {
          "type": "array",
          "items": { "type": "string" }
        },
        "postconditions": {
          "type": "array",
          "items": { "type": "string" }
        }
      }
    },
    "structs": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["name"],
        "properties": {
          "name": {
            "type": "string",
            "minLength": 1
          },
          "generics": {
            "type": "array",
            "items": {
              "type": "object",
              "required": ["name"],
              "properties": {
                "name": { "type": "string" },
                "constraints": {
                  "type": "array",
                  "items": { "type": "string" }
                }
              }
            }
          },
          "fields": {
            "type": "array",
            "items": {
              "type": "object",
              "required": ["name", "type"],
              "properties": {
                "name": { "type": "string" },
                "type": { "$ref": "#/definitions/TypeRef" }
              }
            }
          }
        }
      }
    },
    "methods": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["name"],
        "properties": {
          "name": {
            "type": "string",
            "minLength": 1
          },
          "params": {
            "type": "array",
            "items": {
              "type": "object",
              "required": ["name", "type"],
              "properties": {
                "name": { "type": "string" },
                "type": { "$ref": "#/definitions/TypeRef" }
              }
            }
          },
          "returns": { "type": "string" },
          "preconditions": {
            "type": "array",
            "items": { "type": "string" }
          },
          "postconditions": {
            "type": "array",
            "items": { "type": "string" }
          }
        }
      }
    },
    "verification": {
      "type": "object",
      "properties": {
        "test_cases": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["name"],
            "properties": {
              "name": {
                "type": "string",
                "minLength": 1
              },
              "setup": { "type": "string" },
              "actions": {
                "type": "array",
                "items": { "type": "string" }
              },
              "assertions": {
                "type": "array",
                "items": { "type": "string" }
              }
            }
          }
        }
      }
    }
  }
}"##;
