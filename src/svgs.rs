use crate::entities::svgs::Model;
use crate::icons::IconWeight;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Svg {
    pub id: i32,
    pub icon_id: i32,
    pub weight: IconWeight,
    pub src: String,
}

impl From<Model> for Svg {
    fn from(model: Model) -> Self {
        Svg {
            id: model.id,
            icon_id: model.icon_id,
            weight: IconWeight::from_str(&model.weight).unwrap_or_default(), // Default to IconWeight::Default if parsing fails
            src: model.src,
        }
    }
}

impl From<Svg> for Model {
    fn from(svg: Svg) -> Self {
        Model {
            id: svg.id,
            icon_id: svg.icon_id,
            weight: svg.weight.to_string(),
            src: svg.src,
        }
    }
}
