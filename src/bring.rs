use std::collections::HashMap;

use actix_web::{
    error, get,
    web::{self, Query},
    Error, HttpResponse, Result,
};
use entity::ingredient_units;
use entity::ingredients;
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

struct BringInfo {
    ingredient: ingredients::Model,
    unit: Option<ingredient_units::Model>,
    amount: f64,
    notes: Vec<String>,
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

    let recipe = recipes::get_recipe_by_id(*id, db)
        .await
        .map_err(error::ErrorInternalServerError)?;

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

        let mut all_ingredients: HashMap<i64, HashMap<i64, BringInfo>> = HashMap::new();

        for si in step_ingredients {
            let mut unit_key = si.unit_id.unwrap_or(-1);
            let mut unit = units.iter().find(|u| u.id == unit_key).cloned();
            let mut factor = unit.as_ref().map(|u| u.base_value).unwrap_or(1.0);
            let ingredient = ingredients.iter().find(|i| i.id == si.ingredient_id).unwrap().clone();

            if let Some(iunit) = &unit {
                if iunit.identifier == ingredient_units::Units::PCS {
                    factor = 1.0;
                } else {
                    unit_key = -1;
                    unit = None;
                }
            }

            let row = all_ingredients.entry(si.ingredient_id).or_insert_with(HashMap::new);
            let info = row.entry(unit_key).or_insert(BringInfo {
                ingredient,
                unit,
                amount: 0.0,
                notes: Vec::new(),
            });

            if let Some(amount) = si.amount {
                info.amount += amount * factor;
            }

            if let Some(note) = si.annotation {
                info.notes.push(note);
            }
        }

        let desc = BringRecipe {
            name: recipe.name,
            author: owner.name.unwrap_or(owner.email),
            items: all_ingredients
                .values()
                .flat_map(|row| row.values())
                .map(|info| BringItem {
                    item_id: info.ingredient.name.clone(),
                    spec: calc_amount(info.amount, portions, &info.unit),
                })
                .collect(),
        };

        Ok(HttpResponse::Ok().json(desc))
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

fn calc_amount(amount: f64, portions: f64, unit: &Option<ingredient_units::Model>) -> String {
    if amount > 0.0 {
        let amount = amount * portions;

        if let Some(unit) = &unit {
            let grams = amount * portions * unit.base_value;

            return format!("{:.2} {} ({:.2}g)", amount, unit_to_str(&unit.identifier), grams);
        }

        format!("{:.2}g", amount)
    } else {
        "".to_owned()
    }
}

fn unit_to_str(unit: &ingredient_units::Units) -> &str {
    match unit {
        ingredient_units::Units::PCS => "Stück",
        ingredient_units::Units::TBSP => "Esslöffel",
        ingredient_units::Units::TSP => "Teelöffel",
        ingredient_units::Units::SKOSH => "Prise",
        ingredient_units::Units::PINCH => "Messerspitze",
    }
}
