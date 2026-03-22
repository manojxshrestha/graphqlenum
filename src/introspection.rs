use std::collections::HashSet;
use std::{error::Error, fs, io::BufReader, ops::Deref};

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Query {
    data: Data,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Data {
    #[serde(rename = "__schema")]
    schema: Option<Schema>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    pub query_type: NamedType,
    pub mutation_type: Option<NamedType>,
    pub types: Vec<SchemaType>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NamedType {
    pub name: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaType {
    pub name: String,
    pub fields: Option<Vec<Field>>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: FieldType,
    #[serde(default)]
    pub args: Vec<FieldArg>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldType {
    pub name: Option<String>,
    pub kind: String,
    pub of_type: Box<Option<FieldType>>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldArg {
    pub name: String,
}

impl Schema {
    pub fn new(introspection_query_result_path: &str) -> Result<Schema, Box<dyn Error>> {
        let file = fs::File::open(introspection_query_result_path)?;
        let reader = BufReader::new(file);

        let query: Query = serde_json::from_reader(reader)?;

        match query.data.schema {
            Some(schema) => Ok(schema),
            None => Err(From::from("Introspection file had empty data.")),
        }
    }
}

impl FieldType {
    pub fn get_graph_object_name(&self) -> Option<String> {
        if &self.kind == "OBJECT" {
            return self.name.clone();
        }

        // Walk down the type hierarchy until we reach an OBJECT.
        // This skips the LIST, NON_NULL and other object-wrappers
        // that we don't want in the results.
        let mut field_type = self;
        while let Some(sub_type) = field_type.of_type.deref() {
            if sub_type.kind == "OBJECT" {
                return sub_type.name.clone();
            }
            field_type = sub_type;
        }

        None
    }
}

impl Field {
    pub fn get_connection_type_name(&self) -> Option<String> {
        if let Some(type_name) = &self.field_type.get_graph_object_name() {
            if type_name.ends_with("Connection") {
                // Maybe the schema has an object that ends in Connection but
                // isn't a GraphQL connection so we check a few args to make sure
                let arg_names: HashSet<&str> = self.args.iter().map(|x| &x.name[..]).collect();
                if arg_names.contains("first")
                    && arg_names.contains("last")
                    && arg_names.contains("before")
                    && arg_names.contains("after")
                {
                    return Some(type_name.to_string());
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_graph_object_name_simple_object() {
        let field_type = FieldType {
            kind: "OBJECT".to_string(),
            name: Some("Team".to_string()),
            of_type: Box::default(),
        };
        let name = field_type.get_graph_object_name();
        assert!(name.is_some());
        assert_eq!("Team", name.unwrap());
    }

    #[test]
    fn get_graph_object_name_non_null_object() {
        let field_type = FieldType {
            kind: "NON_NULL".to_string(),
            name: None,
            of_type: Box::new(Some(FieldType {
                kind: "OBJECT".to_string(),
                name: Some("Team".to_string()),
                of_type: Box::default(),
            })),
        };
        let name = field_type.get_graph_object_name();
        assert!(name.is_some());
        assert_eq!("Team", name.unwrap());
    }

    #[test]
    fn get_graph_object_name_object_list() {
        let field_type = FieldType {
            kind: "LIST".to_string(),
            name: None,
            of_type: Box::new(Some(FieldType {
                kind: "OBJECT".to_string(),
                name: Some("Team".to_string()),
                of_type: Box::default(),
            })),
        };
        let name = field_type.get_graph_object_name();
        assert!(name.is_some());
        assert_eq!("Team", name.unwrap());
    }

    #[test]
    fn get_graph_object_name_deeply_nested() {
        let field_type = FieldType {
            kind: "LIST".to_string(),
            name: None,
            of_type: Box::new(Some(FieldType {
                kind: "NON_NULL".to_string(),
                name: None,
                of_type: Box::new(Some(FieldType {
                    kind: "OBJECT".to_string(),
                    name: Some("Team".to_string()),
                    of_type: Box::default(),
                })),
            })),
        };
        let name = field_type.get_graph_object_name();
        assert!(name.is_some());
        assert_eq!("Team", name.unwrap());
    }

    #[test]
    fn get_graph_object_name_non_null_scalar() {
        let field_type = FieldType {
            kind: "NON_NULL".to_string(),
            name: None,
            of_type: Box::new(Some(FieldType {
                kind: "SCALAR".to_string(),
                name: Some("Int".to_string()),
                of_type: Box::default(),
            })),
        };
        let name = field_type.get_graph_object_name();
        assert!(name.is_none());
    }

    #[test]
    fn get_graph_object_name_scalar_list() {
        let field_type = FieldType {
            kind: "LIST".to_string(),
            name: None,
            of_type: Box::new(Some(FieldType {
                kind: "SCALAR".to_string(),
                name: Some("Int".to_string()),
                of_type: Box::default(),
            })),
        };
        let name = field_type.get_graph_object_name();
        assert!(name.is_none());
    }

    #[test]
    fn get_graph_object_name_boolean() {
        let field_type = FieldType {
            kind: "SCALAR".to_string(),
            name: Some("Boolean".to_string()),
            of_type: Box::default(),
        };
        let name = field_type.get_graph_object_name();
        assert!(name.is_none());
    }

    #[test]
    fn get_graph_object_name_int() {
        let field_type = FieldType {
            kind: "SCALAR".to_string(),
            name: Some("Int".to_string()),
            of_type: Box::default(),
        };
        let name = field_type.get_graph_object_name();
        assert!(name.is_none());
    }

    #[test]
    fn get_graph_object_name_regression_test() {
        let json = "
            {
                \"kind\": \"OBJECT\",
                \"name\": \"SpeciesEdge\",
                \"description\": \"An edge in a connection.\",
                \"fields\": [
                  {
                    \"name\": \"node\",
                    \"description\": \"The item at the end of the edge\",
                    \"args\": [],
                    \"type\": {
                      \"kind\": \"OBJECT\",
                      \"name\": \"Species\",
                      \"ofType\": null
                    },
                    \"isDeprecated\": false,
                    \"deprecationReason\": null
                  },
                  {
                    \"name\": \"cursor\",
                    \"description\": \"A cursor for use in pagination\",
                    \"args\": [],
                    \"type\": {
                      \"kind\": \"NON_NULL\",
                      \"name\": null,
                      \"ofType\": {
                        \"kind\": \"SCALAR\",
                        \"name\": \"String\",
                        \"ofType\": null
                      }
                    },
                    \"isDeprecated\": false,
                    \"deprecationReason\": null
                  }
                ],
                \"inputFields\": null,
                \"interfaces\": [],
                \"enumValues\": null,
                \"possibleTypes\": null
              }
          ";

        let schema_type: SchemaType = serde_json::from_str(json).unwrap();
        assert!(schema_type.fields.is_some());

        let fields = schema_type.fields.unwrap();
        assert_eq!(2, fields.len());

        let node_name = fields[0].field_type.get_graph_object_name();
        assert!(node_name.is_some());
        assert_eq!("Species", node_name.unwrap());

        let cursor_name = fields[1].field_type.get_graph_object_name();
        assert_eq!(None, cursor_name);
    }

    #[test]
    fn get_connection_type_name_happy_path() {
        let json = "
        {
            \"name\": \"thanks_items\",
            \"description\": null,
            \"args\": [
              {
                \"name\": \"after\",
                \"description\": \"Returns the elements in the list that come after the specified cursor.\",
                \"type\": {
                  \"kind\": \"SCALAR\",
                  \"name\": \"String\",
                  \"ofType\": null
                },
                \"defaultValue\": null
              },
              {
                \"name\": \"before\",
                \"description\": \"Returns the elements in the list that come before the specified cursor.\",
                \"type\": {
                  \"kind\": \"SCALAR\",
                  \"name\": \"String\",
                  \"ofType\": null
                },
                \"defaultValue\": null
              },
              {
                \"name\": \"first\",
                \"description\": \"Returns the first _n_ elements from the list.\",
                \"type\": {
                  \"kind\": \"SCALAR\",
                  \"name\": \"Int\",
                  \"ofType\": null
                },
                \"defaultValue\": null
              },
              {
                \"name\": \"last\",
                \"description\": \"Returns the last _n_ elements from the list.\",
                \"type\": {
                  \"kind\": \"SCALAR\",
                  \"name\": \"Int\",
                  \"ofType\": null
                },
                \"defaultValue\": null
              }
            ],
            \"type\": {
              \"kind\": \"OBJECT\",
              \"name\": \"ThanksItemConnection\",
              \"ofType\": null
            },
            \"isDeprecated\": false,
            \"deprecationReason\": null
          }
          ";

        let field: Field = serde_json::from_str(json).unwrap();
        let connection_object = field.get_connection_type_name();

        assert!(connection_object.is_some());
        assert_eq!("ThanksItemConnection", connection_object.unwrap());
    }
}
