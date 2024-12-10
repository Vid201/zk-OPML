use serde::Deserialize;

pub type Data = Vec<Vec<f64>>;

#[derive(Clone, Debug, Deserialize, Default, PartialEq)]
pub struct DataFile {
    pub input_data: Data,
    pub output_data: Option<Data>,
}
