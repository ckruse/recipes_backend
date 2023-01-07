use actix_web::{
    error, get,
    web::{self, Query},
    Error, HttpResponse, Result,
};
use entity::ingredient_units::Units;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter};
use serde::Deserialize;

use crate::recipes;

#[derive(serde::Serialize)]
pub struct BringItem {
    #[serde(rename = "itemId")]
    pub item_id: String,
    pub spec: String,
}

#[derive(serde::Serialize)]
pub struct BringRecipe {
    pub name: String,
    pub author: String,
    pub items: Vec<BringItem>,
}

#[derive(Deserialize)]
pub struct ProtionsQuery {
    pub portions: Option<f64>,
}

#[get("/recipes/{id}/bring.json")]
pub async fn get_recipe_bring(
    id: web::Path<i64>,
    params: Query<ProtionsQuery>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    let portions = match params.portions {
        Some(portions) => {
            if portions < 0.0 {
                1.0
            } else {
                portions
            }
        }
        None => 1.0,
    };

    let db = db.get_ref();

    log::debug!("id: {:?}", id);

    let recipe = recipes::get_recipe_by_id(*id, db)
        .await
        .map_err(error::ErrorInternalServerError)?;

    log::debug!("recipe: {:?}", recipe);

    if let Some(recipe) = recipe {
        let owner = recipe
            .find_related(entity::users::Entity)
            .one(db)
            .await
            .map_err(error::ErrorInternalServerError)?
            .ok_or_else(|| error::ErrorNotFound("User not found"))?;

        let step_ingredients = recipe
            .find_related(entity::steps::Entity)
            .find_also_related(entity::steps_ingridients::Entity)
            .all(db)
            .await
            .map_err(error::ErrorInternalServerError)?
            .into_iter()
            .flat_map(|(_step, step_ingredients)| step_ingredients)
            .collect::<Vec<entity::steps_ingridients::Model>>();

        let ingredient_ids = step_ingredients
            .iter()
            .map(|step_ingredient| step_ingredient.ingredient_id)
            .collect::<Vec<i64>>();

        let si_unit_ids = step_ingredients
            .iter()
            .filter_map(|step_ingredient| step_ingredient.unit_id)
            .collect::<Vec<i64>>();

        let ingredients = entity::ingredients::Entity::find()
            .filter(entity::ingredients::Column::Id.is_in(ingredient_ids))
            .all(db)
            .await
            .map_err(error::ErrorInternalServerError)?;

        let units = entity::ingredient_units::Entity::find()
            .filter(entity::ingredient_units::Column::Id.is_in(si_unit_ids))
            .all(db)
            .await
            .map_err(error::ErrorInternalServerError)?;

        let desc = BringRecipe {
            name: recipe.name,
            author: owner.name.unwrap_or(owner.email),
            items: step_ingredients
                .iter()
                .map(|step_ingredient| {
                    let ingredient = ingredients
                        .iter()
                        .find(|ingredient| ingredient.id == step_ingredient.ingredient_id)
                        .unwrap();

                    BringItem {
                        item_id: ingredient.name.clone(),
                        spec: calc_amount(step_ingredient, portions, &units),
                    }
                })
                .collect(),
        };

        Ok(HttpResponse::Ok().json(desc))
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

fn calc_amount(
    step_ingredient: &entity::steps_ingridients::Model,
    portions: f64,
    units: &[entity::ingredient_units::Model],
) -> String {
    if let Some(amount) = step_ingredient.amount {
        let amount = amount * portions;

        if let Some(unit_id) = &step_ingredient.unit_id {
            let unit = units.iter().find(|unit| unit.id == *unit_id).unwrap();

            let grams = amount * portions * unit.base_value;

            return format!("{:.2} {} ({:.2}g)", amount, unit_to_str(&unit.identifier), grams);
        }

        format!("{:.2}g", amount)
    } else {
        "".to_owned()
    }
}

fn unit_to_str(unit: &Units) -> &str {
    match unit {
        Units::PCS => "Stück",
        Units::TBSP => "Esslöffel",
        Units::TSP => "Teelöffel",
        Units::SKOSH => "Prise",
        Units::PINCH => "Messerspitze",
    }
}
