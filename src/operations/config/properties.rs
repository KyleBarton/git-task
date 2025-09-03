use gittask::TaskContext;
use crate::property::PropertyManager;
use crate::util::{error_message, read_from_pipe, success_message};

pub(crate) fn task_config_properties_add(context: &TaskContext, name: String, value_type: String, color: String, style: Option<String>, enum_values: Option<Vec<String>>, cond_format: Option<Vec<String>>) -> bool {
    let mut prop_manager = PropertyManager::new(&context);
    match prop_manager.add_property(name.clone(), value_type, color, style, enum_values, cond_format) {
        Ok(_) => success_message(format!("Property {name} has been added")),
        Err(e) => error_message(format!("ERROR: {e}"))
    }
}

pub(crate) fn task_config_properties_delete(context: &TaskContext, name: String, force: bool) -> bool {
    let mut prop_manager = PropertyManager::new(&context);

    if !force {
        if let Ok(tasks) = context.list_tasks() {
            let task_exists = tasks.iter().any(|task| task.has_property(&name));
            if task_exists {
                return error_message("Can't delete a property, some tasks still have it. Use --force option to override.".to_string());
            }
        }
    }

    match prop_manager.delete_property(&name) {
        Ok(_) => success_message(format!("Property {name} has been deleted")),
        Err(e) => error_message(format!("ERROR: {e}"))
    }
}

pub(crate) fn task_config_properties_get(context: &TaskContext, name: String, param: String) -> bool {
    let prop_manager = PropertyManager::new(&context);
    match prop_manager.get_parameter(&name, &param) {
        Some(value) => success_message(value),
        None => error_message(format!("Unknown property {name} or parameter: {param}"))
    }
}

pub(crate) fn task_config_properties_set(context: &TaskContext, name: String, param: String, value: String) -> bool {
    let mut prop_manager = PropertyManager::new(&context);
    match prop_manager.set_parameter(&name, &param, &value) {
        Ok(_) => {
            println!("{name} {param} has been updated");

            if param.as_str() == "name" {
                match context.list_tasks() {
                    Ok(tasks) => {
                        for mut task in tasks {
                            if task.has_property(&name) {
                                let task_prop_value = task.get_property(&name).unwrap().clone();
                                task.set_property(&value, &task_prop_value);
                                task.delete_property(&name);
                                if let Err(e) = context.update_task(task) {
                                    eprintln!("ERROR: {e}");
                                }
                            }
                        }
                    },
                    Err(e) => eprintln!("ERROR: {e}")
                }
            }

            true
        },
        Err(e) => error_message(format!("ERROR: {e}"))
    }
}

pub(crate) fn task_config_properties_list(context: &TaskContext) -> bool {
    let prop_manager = PropertyManager::new(&context);
    println!("Name\tValue type\tColor\tStyle\tEnum values");
    prop_manager.get_properties().iter().for_each(|property| {
        let enums = match property.get_enum_values() {
            Some(enum_values) => {
                enum_values.iter()
                    .map(|saved_enum| saved_enum.get_name().to_string() + "," + saved_enum.get_color() + (if saved_enum.get_style().is_some() { "," } else { "" }) + saved_enum.get_style().unwrap_or_else(|| ""))
                    .collect::<Vec<_>>()
                    .join(";")
            },
            None => String::new()
        };
        println!("{}\t{}\t{}\t{}\t{}", property.get_name(), property.get_value_type(), property.get_color(), property.get_style().unwrap_or_else(|| ""), enums);
    });
    true
}

pub(crate) fn task_config_properties_import(context: &TaskContext) -> bool {
    if let Some(input) = read_from_pipe() {
        match PropertyManager::parse_properties(input) {
            Ok(statuses) => {
                let mut prop_manager = PropertyManager::new(&context);
                match prop_manager.set_properties(statuses) {
                    Ok(_) => success_message("Import successful".to_string()),
                    Err(e) => error_message(format!("ERROR: {e}"))
                }
            },
            Err(e) => error_message(e)
        }
    } else {
        error_message("Can't read from pipe".to_string())
    }
}

pub(crate) fn task_config_properties_export(context: &TaskContext, pretty: bool) -> bool {
    let prop_manager = PropertyManager::new(&context);
    let func = if pretty { serde_json::to_string_pretty } else { serde_json::to_string };

    if let Ok(result) = func(&prop_manager.get_properties()) {
        success_message(result)
    } else {
        error_message("ERROR serializing property list".to_string())
    }
}

pub(crate) fn task_config_properties_reset(context: &TaskContext) -> bool {
    let mut prop_manager = PropertyManager::new(&context);
    match prop_manager.set_defaults() {
        Ok(_) => success_message("Properties have been reset".to_string()),
        Err(e) => error_message(format!("ERROR: {e}"))
    }
}

pub(crate) fn task_config_properties_enum_list(context: &TaskContext, name: String) -> bool {
    let prop_manager = PropertyManager::new(&context);
    let property = prop_manager.get_properties().iter().find(|saved_prop| saved_prop.get_name() == name);
    match property {
        Some(property) => {
            match property.get_enum_values() {
                Some(enum_values) => {
                    for enum_value in enum_values {
                        println!("{} {} {}", enum_value.get_name(), enum_value.get_color(), enum_value.get_style().unwrap_or_else(|| ""));
                    }
                    true
                },
                None => error_message("Property has no enum values".to_string())
            }
        },
        None => error_message("Property not found".to_string())
    }
}

pub(crate) fn task_config_properties_enum_add(context: &TaskContext, name: String, enum_value_name: String, enum_value_color: String, enum_value_style: Option<String>) -> bool {
    let mut prop_manager = PropertyManager::new(&context);
    match prop_manager.add_enum_property(name, enum_value_name, enum_value_color, enum_value_style) {
        Ok(_) => success_message("Property enum has been added".to_string()),
        Err(e) => error_message(format!("ERROR: {e}"))
    }
}

pub(crate) fn task_config_properties_enum_get(context: &TaskContext, property: String, enum_value_name: String, parameter: String) -> bool {
    let prop_manager = PropertyManager::new(&context);
    match prop_manager.get_enum_parameter(property, enum_value_name, parameter) {
        Ok(s) => success_message(s),
        Err(e) => error_message(format!("ERROR: {e}"))
    }
}

pub(crate) fn task_config_properties_enum_set(context: &TaskContext, name: String, enum_value_name: String, enum_value_color: String, enum_value_style: Option<String>) -> bool {
    let mut prop_manager = PropertyManager::new(&context);
    match prop_manager.set_enum_property(name, enum_value_name, enum_value_color, enum_value_style) {
        Ok(_) => success_message("Property enum has been updated".to_string()),
        Err(e) => error_message(format!("ERROR: {e}"))
    }
}

pub(crate) fn task_config_properties_enum_delete(context: &TaskContext, name: String, enum_value_name: String) -> bool {
    let mut prop_manager = PropertyManager::new(&context);
    match prop_manager.delete_enum_property(name, enum_value_name) {
        Ok(_) => success_message("Property enum has been deleted".to_string()),
        Err(e) => error_message(format!("ERROR: {e}"))
    }
}

pub(crate) fn task_config_properties_cond_format_list(context: &TaskContext, name: String) -> bool {
    let prop_manager = PropertyManager::new(&context);
    let property = prop_manager.get_properties().iter().find(|saved_prop| saved_prop.get_name() == name);
    match property {
        Some(property) => {
            match property.get_cond_format() {
                Some(cond_format) => {
                    for cond_format_value in cond_format {
                        println!("{} {} {}", cond_format_value.get_condition(), cond_format_value.get_color(), cond_format_value.get_style().unwrap_or_else(|| ""));
                    }
                    true
                },
                None => error_message("Property has no conditional formatting".to_string())
            }
        },
        None => error_message("Property not found".to_string())
    }
}

pub(crate) fn task_config_properties_cond_format_add(context: &TaskContext, name: String, cond_format_expr: String, cond_format_color: String, cond_format_style: Option<String>) -> bool {
    let mut prop_manager = PropertyManager::new(&context);
    match prop_manager.add_cond_format(name, cond_format_expr, cond_format_color, cond_format_style) {
        Ok(_) => success_message("Property conditional formatting has been added".to_string()),
        Err(e) => error_message(format!("ERROR: {e}"))
    }
}

pub(crate) fn task_config_properties_cond_format_clear(context: &TaskContext, name: String) -> bool {
    let mut prop_manager = PropertyManager::new(&context);
    match prop_manager.clear_cond_format(name) {
        Ok(_) => success_message("Property conditional formatting has been cleared".to_string()),
        Err(e) => error_message(format!("ERROR: {e}"))
    }
}