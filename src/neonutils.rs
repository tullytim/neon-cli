#![allow(dead_code)]

use chrono::Utc;
use serde_json::Value;
use comfy_table::*;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL};
use std::{collections::HashMap, vec::Vec};

/// The postgres-crate does not provide a default mapping to fallback to String for all
/// types: row.get is generic and without a type assignment the FromSql-Trait cannot be inferred.
/// This function matches over the current column-type and does a manual conversion
pub fn reflective_get(row: &postgres::Row, index: usize) -> String {
    let column_type = row.columns().get(index).map(|c| c.type_().name()).unwrap();
    // see https://docs.rs/sqlx/0.4.0-beta.1/sqlx/postgres/types/index.html
    let value = match column_type {
        "bool" => {
            let v: Option<bool> = row.get(index);
            v.map(|v| v.to_string())
        }
        "varchar" | "char(n)" | "text" | "name" => {
            let v: Option<String> = row.get(index);
            v
        }
        // "char" => {
        //     let v: i8 = row.get(index);
        // }
        "int2" | "smallserial" | "smallint" => {
            let v: Option<i16> = row.get(index);
            v.map(|v| v.to_string())
        }
        "int" | "int4" | "serial" => {
            let v: Option<i32> = row.get(index);
            v.map(|v| v.to_string())
        }
        "int8" | "bigserial" | "bigint" => {
            let v: Option<i64> = row.get(index);
            v.map(|v| v.to_string())
        }
        "float4" | "real" => {
            let v: Option<f32> = row.get(index);
            v.map(|v| v.to_string())
        }
        "float8" | "double precision" => {
            let v: Option<f64> = row.get(index);
            v.map(|v| v.to_string())
        }
        "timestamp" | "timestamptz" => {
            // with-chrono feature is needed for this
            let v: Option<chrono::DateTime<Utc>> = row.get(index);
            v.map(|v| v.to_string())
        }
        &_ => Some("CANNOT PARSE".to_string()),
    };
    value.unwrap_or("".to_string())
}


pub fn print_generic_json_table(rows: &Vec<Value>) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);
    let mut saw_first_row = false;

    for row in rows {
        for (_key, _value) in row.as_object().unwrap() {
            if !saw_first_row {
                let col_names: Vec<String> =
                row.as_object().unwrap().iter().map(|c| c.0.to_string()).collect::<Vec<String>>();
                table.set_header(col_names);
                saw_first_row = true;
            }
        }
        let row_strs: Vec<String> = row.as_object().unwrap().iter().map(|c| c.1.to_string()).collect::<Vec<String>>();
        table.add_row(row_strs);
    }
    println!("{table}");
}

pub fn jsonstring_to_map(json_str: &String) -> Box<HashMap<String,String>>{
    let v: Value = serde_json::from_str(json_str).unwrap();
    let mut map: HashMap<String, String> = HashMap::new();
    if let Value::Object(object) = v {
        for (key, value) in object {
            if let Value::String(string_value) = value {
                map.insert(key, string_value);
            }
        }
    }
    return Box::new(map);
}

pub fn json_get_first_object(json_blob: &Value) -> Option<Value> {
    let mut rv: Option<Value> = None;
    if let Some((_key, value)) = json_blob.as_object().unwrap().iter().next() {
        rv = Some(value.clone());
    }
    return rv;
}