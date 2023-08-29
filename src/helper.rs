use aws_sdk_cloudformation::types::builders::ParameterBuilder;
use aws_sdk_cloudformation::types::Parameter;
use serde::{Deserialize, Serialize};
use serde_json;
use std::env::var;

#[derive(Serialize, Deserialize, Debug)]
pub struct ParamJson {
    pub parameter_key: String,
    pub parameter_value: String,
    pub use_previous_value: bool,
    pub resolved_value: String,
}

pub fn json_to_param(s: String) -> Option<Vec<Parameter>> {
    let json_input: Vec<ParamJson> = serde_json::from_str(&s).unwrap();

    let mut p = Vec::new();
    for j in json_input {
        let param_builder = ParameterBuilder::default();

        let param = param_builder
            .set_parameter_key(Some(j.parameter_key))
            .set_parameter_value(Some(j.parameter_value))
            .set_use_previous_value(Some(j.use_previous_value))
            .set_resolved_value(Some(j.resolved_value))
            .build();
        p.push(param);
    }

    Some(p)
}

pub fn get_editor () -> String{
    // first env var
    // then config file
    // if none, create a config file with default editor as nano
    var("EDITOR").unwrap()
}