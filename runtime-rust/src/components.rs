extern crate yarrow_validator;
use yarrow_validator::proto;
use crate::utilities;
use crate::base::*;

use ndarray::prelude::*;
use std::collections::HashMap;

extern crate csv;
extern crate num;

use std::str::FromStr;
use yarrow_validator::utilities::buffer::{parse_proto_value, NodeEvaluation, NodeArguments, get_f64, get_array_f64, get_array_bool, get_bool, get_i64};


macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

pub fn component_literal(x: &proto::Literal) -> Result<NodeEvaluation, String> {
    Ok(parse_proto_value(&x.to_owned().value.unwrap()).unwrap())
}

//pub fn component_table(table: &proto::Table, dataset: &proto::Dataset, arguments: &NodeArguments) -> NodeEvaluation {
//    let table = dataset.tables.get(&datasource.dataset_id).unwrap();
//    match table.value.as_ref().unwrap() {
//        proto::table::Value::FilePath(path) => {
//        },
//
//    }
//}

pub fn component_datasource(
    datasource: &proto::DataSource, dataset: &proto::Dataset, arguments: &NodeArguments
) -> Result<NodeEvaluation, String> {
//    println!("datasource");

    let table = dataset.tables.get(&datasource.dataset_id).unwrap();
    Ok(match table.value.as_ref().unwrap() {
        proto::table::Value::FilePath(path) => {

            fn get_column<T>(path: &String, column: &String) -> Vec<T>
                where T: FromStr, <T as std::str::FromStr>::Err: std::fmt::Debug {
                let mut rdr = csv::Reader::from_path(path).unwrap();
                rdr.deserialize().map(|result| {
                    let record: HashMap<String, String> = result.unwrap();
//                    println!("{:?}", record);
                    record[column].parse::<T>().unwrap()
                }).collect()
            }

            match arguments.get("datatype").unwrap() {
                NodeEvaluation::Str(x) => Ok(match x.first().unwrap().as_ref() {
//                    "BYTES" =>
//                        Ok(NodeEvaluation::Bytes(Array1::from(get_column::<u8>(&path, &datasource.column_id)).into_dyn())),
                    "BOOL" =>
                        Ok(NodeEvaluation::Bool(Array1::from(get_column::<bool>(&path, &datasource.column_id)).into_dyn())),
                    "I64" =>
                        Ok(NodeEvaluation::I64(Array1::from(get_column::<i64>(&path, &datasource.column_id)).into_dyn())),
                    "F64" =>
                        Ok(NodeEvaluation::F64(Array1::from(get_column::<f64>(&path, &datasource.column_id)).into_dyn())),
                    "STRING" =>
                        Ok(NodeEvaluation::Str(Array1::from(get_column::<String>(&path, &datasource.column_id)).into_dyn())),
                    _ => Err("Datatype is not recognized.".to_string())
                }.unwrap()),
                _ => Err("Datatype must be a string.".to_string())
            }
        },
        proto::table::Value::Literal(value) => parse_proto_value(&value),
        _ => Err("Only file paths are supported".to_string())
    }.unwrap())
}

pub fn component_add(
    _x: &proto::Add, arguments: &NodeArguments
) -> Result<NodeEvaluation, String> {
//    println!("add");
    match (arguments.get("left").unwrap(), arguments.get("right").unwrap()) {
        (NodeEvaluation::F64(x), NodeEvaluation::F64(y)) =>
            Ok(NodeEvaluation::F64(x + y)),
        (NodeEvaluation::I64(x), NodeEvaluation::I64(y)) =>
            Ok(NodeEvaluation::I64(x + y)),
        _ => Err("Add: Either the argument types are mismatched or non-numeric.".to_string())
    }
}


pub fn component_subtract(
    _x: &proto::Subtract, arguments: &NodeArguments
) -> Result<NodeEvaluation, String> {

    match (arguments.get("left").unwrap(), arguments.get("right").unwrap()) {
        (NodeEvaluation::F64(x), NodeEvaluation::F64(y)) =>
            Ok(NodeEvaluation::F64(x - y)),
        (NodeEvaluation::I64(x), NodeEvaluation::I64(y)) =>
            Ok(NodeEvaluation::I64(x - y)),
        _ => Err("Subtract: Either the argument types are mismatched or non-numeric.".to_string())
    }
}

pub fn component_divide(
    _x: &proto::Divide, arguments: &NodeArguments
) -> Result<NodeEvaluation, String> {

    match (arguments.get("left").unwrap(), arguments.get("right").unwrap()) {
        (NodeEvaluation::F64(x), NodeEvaluation::F64(y)) =>
            Ok(NodeEvaluation::F64(x / y)),
        (NodeEvaluation::I64(x), NodeEvaluation::I64(y)) =>
            Ok(NodeEvaluation::I64(x / y)),
        _ => Err("Divide: Either the argument types are mismatched or non-numeric.".to_string())
    }
}

pub fn component_multiply(
    _x: &proto::Multiply, arguments: &NodeArguments
) -> Result<NodeEvaluation, String> {
    match (arguments.get("left").unwrap(), arguments.get("right").unwrap()) {
        (NodeEvaluation::F64(x), NodeEvaluation::F64(y)) =>
            Ok(NodeEvaluation::F64(x * y)),
        (NodeEvaluation::I64(x), NodeEvaluation::I64(y)) =>
            Ok(NodeEvaluation::I64(x * y)),
        _ => Err("Multiply: Either the argument types are mismatched or non-numeric.".to_string())
    }
}

pub fn component_power(
    _x: &proto::Power, arguments: &NodeArguments
) -> Result<NodeEvaluation, String> {

    let power: f64 = get_f64(&arguments, "right");
    let data = get_array_f64(&arguments, "left");
    Ok(NodeEvaluation::F64(data.mapv(|x| x.powf(power))))
}

pub fn component_negate(
    _x: &proto::Negate, arguments: &NodeArguments
) -> Result<NodeEvaluation, String> {

    match arguments.get("data").unwrap() {
        NodeEvaluation::F64(x) =>
            Ok(NodeEvaluation::F64(-x)),
        NodeEvaluation::I64(x) =>
            Ok(NodeEvaluation::I64(-x)),
        _ => Err("Negate: Argument must be numeric.".to_string())
    }
}

pub fn component_impute_float_uniform(
    _x: &proto::ImputeFloatUniform, arguments: &NodeArguments
) -> Result<NodeEvaluation, String> {

    let data: ArrayD<f64> = get_array_f64(&arguments, "data");
    let min: f64 = get_f64(&arguments, "min");
    let max: f64 = get_f64(&arguments, "max");
    Ok(NodeEvaluation::F64(utilities::transformations::impute_float_uniform(&data, &min, &max)))
}

pub fn component_impute_float_gaussian(
    _x: &proto::ImputeFloatGaussian, arguments: &NodeArguments
) -> Result<NodeEvaluation, String> {

    let data: ArrayD<f64> = get_array_f64(&arguments, "data");
    let shift: f64 = get_f64(&arguments, "shift");
    let scale: f64 = get_f64(&arguments, "scale");
    let min: f64 = get_f64(&arguments, "min");
    let max: f64 = get_f64(&arguments, "max");
    Ok(NodeEvaluation::F64(utilities::transformations::impute_float_gaussian(&data, &shift, &scale, &min, &max)))
}

pub fn component_impute_int_uniform(
    _x: &proto::ImputeIntUniform, arguments: &NodeArguments
) -> Result<NodeEvaluation, String> {

    let data: ArrayD<f64> = get_array_f64(&arguments, "data");
    let min: f64 = get_f64(&arguments, "min");
    let max: f64 = get_f64(&arguments, "max");
    Ok(NodeEvaluation::F64(utilities::transformations::impute_int_uniform(&data, &min, &max)))
}

pub fn component_bin(
    _X: &proto::Bin, arguments: &NodeArguments
) -> Result<NodeEvaluation, String> {

    let data: ArrayD<f64> = get_array_f64(&arguments, "data");
    let edges: ArrayD<f64> = get_array_f64(&arguments, "edges");
    let inclusive_left: ArrayD<bool> = get_array_bool(&arguments, "inclusive_left");
    Ok(NodeEvaluation::Str(utilities::transformations::bin(&data, &edges, &inclusive_left)))
}

pub fn component_row_wise_min(
    _x: &proto::RowMin, arguments: &NodeArguments
) -> Result<NodeEvaluation, String> {

    match (*arguments.get("left").unwrap(), *arguments.get("right").unwrap()) {
        (NodeEvaluation::F64(left), NodeEvaluation::F64(right)) =>
            Ok(NodeEvaluation::F64(utilities::transformations::broadcast_map(
                &left, &right, &|l: &f64, r: &f64| l.min(*r))?)),
        (NodeEvaluation::I64(left), NodeEvaluation::I64(right)) =>
            Ok(NodeEvaluation::I64(utilities::transformations::broadcast_map(
                &left, &right, &|l: &i64, r: &i64| std::cmp::min(*l, *r))?)),
        _ => Err("Unsupported types in row_wise_min.".to_string())
    }
}

pub fn component_row_wise_max(
    _x: &proto::RowMax, arguments: &NodeArguments
) -> Result<NodeEvaluation, String> {

    match (*arguments.get("left").unwrap(), *arguments.get("right").unwrap()) {
        (NodeEvaluation::F64(left), NodeEvaluation::F64(right)) =>
            Ok(NodeEvaluation::F64(utilities::transformations::broadcast_map(
                &left, &right, &|l: &f64, r: &f64| l.max(*r))?)),
        (NodeEvaluation::I64(left), NodeEvaluation::I64(right)) =>
            Ok(NodeEvaluation::I64(utilities::transformations::broadcast_map(
                &left, &right, &|l: &i64, r: &i64| std::cmp::max(*l, *r))?)),
        _ => Err("Unsupported types in row_wise_min.".to_string())
    }
}

pub fn component_clamp(
    _X: &proto::Clamp, arguments: &NodeArguments
) -> Result<NodeEvaluation, String> {
    match (*arguments.get("data").unwrap(),
           *arguments.get("min").unwrap(),
           *arguments.get("max").unwrap(),
           *arguments.get("categories").unwrap(),
           *arguments.get("null_value").unwrap(),) {
                (NodeEvaluation::F64(data), NodeEvaluation::F64(min), NodeEvaluation::F64(max), ..) =>
                    Ok(NodeEvaluation::F64(utilities::transformations::clamp_numeric(data, min, max))),
                (NodeEvaluation::Str(data), NodeEvaluation::Str(categories), NodeEvaluation::Str(null_value), ..) =>
                    Ok(NodeEvaluation::Str(utilities::transformations::clamp_categorical(data, categories, null_value.first().unwrap()))),
                _ => Err("unsupported types".to_string())
    }
}

//pub fn component_count(
//    _X: &proto::Count, arguments: &NodeArguments,
//) -> Result<NodeEvaluation, String> {
//
//    match (arguments.get("data").unwrap(), arguments.get("group_by").unwrap()) {
//        (NodeEvaluation::F64(data), NodeEvaluation::F64(group_by)) =>
//            Ok(NodeEvaluation::F64(utilities::aggregations::count(&get_array_f64(&arguments, "data"), &Some(get_array_f64(&arguments, "group_by"))))),
//        (NodeEvaluation::Str(data), NodeEvaluation::Str(group_by)) =>
//            Ok(NodeEvaluation::F64(utilities::aggregations::count(&get_array_str(&arguments, "data"), &Some(get_array_str(&arguments, "group_by"))))),
//        (NodeEvaluation::Bool(data), NodeEvaluation::Bool(group_by)) =>
//            Ok(NodeEvaluation::F64(utilities::aggregations::count(&get_array_bool(&arguments, "data"), &Some(get_array_bool(&arguments, "group_by"))))),
//        _ => Err("Count: Data type must be f64, string, or bool".to_string())
//    }
//}

//pub fn component_histogram(
//    _X: &proto::Bin, argument: &NodeArguments
//) -> Result<NodeEvaluation, String> {
//    let data: ArrayD<f64> = get_array_f64(&arguments, "data");
//    let edges: ArrayD<f64> = get_array_f64(&arguments, "edges");
//    let inclusive_left: bool = get_bool(&arguments, "inclusive_left");
//    Ok(NodeEvaluation::String_F64_HashMap(utilities::aggregations::histogram(&data, &edges, &inclusive_left))
//}

pub fn component_mean(
    _x: &proto::Mean, arguments: &NodeArguments
) -> Result<NodeEvaluation, String> {

    let data: ArrayD<f64> = get_array_f64(&arguments, "data");
    Ok(NodeEvaluation::F64(utilities::aggregations::mean(&data)))
}

pub fn component_variance(
    _x: &proto::Variance, arguments: &NodeArguments
) -> Result<NodeEvaluation, String> {

    let data: ArrayD<f64> = get_array_f64(&arguments, "data");
    let finite_sample_correction: bool = get_bool(&arguments, "finite_sample_correction");
    Ok(NodeEvaluation::F64(utilities::aggregations::variance(&data, &finite_sample_correction)))
}

pub fn component_kth_raw_sample_moment(
    _x: &proto::KthRawSampleMoment, arguments: &NodeArguments
) -> Result<NodeEvaluation, String> {
    let data: ArrayD<f64> = get_array_f64(&arguments, "data");
    let k: i64 = get_i64(&arguments, "k");
    Ok(NodeEvaluation::F64(utilities::aggregations::kth_raw_sample_moment(&data, &k)))
}

pub fn component_median(
    _x: &proto::Median, arguments: &NodeArguments
) -> Result<NodeEvaluation, String> {
    let data: ArrayD<f64> = get_array_f64(&arguments, "data");
    Ok(NodeEvaluation::F64(utilities::aggregations::median(&data)))
}

pub fn component_sum(
    _x: &proto::Sum, arguments: &NodeArguments
) -> Result<NodeEvaluation, String> {

    let data: ArrayD<f64> = get_array_f64(&arguments, "data");
    Ok(NodeEvaluation::F64(utilities::aggregations::sum(&data)))
}

pub fn component_laplace_mechanism(
    _x: &proto::LaplaceMechanism, arguments: &NodeArguments
) -> Result<NodeEvaluation, String> {
    let epsilon: f64 = get_f64(&arguments, "epsilon");
    let sensitivity: f64 = get_f64(&arguments, "sensitivity");
    Ok(NodeEvaluation::F64(utilities::mechanisms::laplace_mechanism(&epsilon, &sensitivity)))
}

pub fn component_gaussian_mechanism(
    _x: &proto::GaussianMechanism, arguments: &NodeArguments
) -> Result<NodeEvaluation, String> {
    let epsilon: f64 = get_f64(&arguments, "epsilon");
    let delta: f64 = get_f64(&arguments, "delta");
    let sensitivity: f64 = get_f64(&arguments, "sensitivity");
    Ok(NodeEvaluation::F64(utilities::mechanisms::gaussian_mechanism(&epsilon, &delta, &sensitivity)))
}

pub fn component_simple_geometric_mechanism(
    _x: &proto::SimpleGeometricMechanism, arguments: &NodeArguments
) -> Result<NodeEvaluation, String> {
    let epsilon: f64 = get_f64(&arguments, "epsilon");
    let sensitivity: f64 = get_f64(&arguments, "sensitivity");
    let count_min: i64 = get_i64(&arguments, "count_min");
    let count_max: i64 = get_i64(&arguments, "count_max");
    let enforce_constant_time: bool = get_bool(&arguments, "enforce_constant_time");
    Ok(NodeEvaluation::I64(utilities::mechanisms::simple_geometric_mechanism(
                             &epsilon, &sensitivity, &count_min, &count_max, &enforce_constant_time)))
}
