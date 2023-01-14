use std::collections::HashMap;

use actix_web::{
    error, get,
    web::{self, Query},
    Error, HttpResponse, Result,
};
use chrono::NaiveDate;
use entity::ingredient_units;
use entity::ingredients;
use entity::steps;
use entity::steps_ingredients;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter};
use serde::Deserialize;

use crate::{recipes, users, weekplan};

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
pub struct PortionsQuery {
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
    params: Query<PortionsQuery>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    let db = db.get_ref();

    let recipe = recipes::get_recipe_by_id(*id, db)
        .await
        .map_err(error::ErrorInternalServerError)?;

    if recipe.is_none() {
        return Ok(HttpResponse::NotFound().finish());
    }

    let recipe = recipe.unwrap();
    let portions = match params.portions {
        Some(portions) => {
            if portions < 0.0 {
                recipe.default_servings as f64
            } else {
                portions
            }
        }
        None => 1.0,
    };

    let owner = recipe
        .find_related(entity::users::Entity)
        .one(db)
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or_else(|| error::ErrorNotFound("User not found"))?;

    let step_ingredients = recipe
        .find_related(entity::steps::Entity)
        .find_also_related(steps_ingredients::Entity)
        .all(db)
        .await
        .map_err(error::ErrorInternalServerError)?
        .into_iter()
        .flat_map(|(_step, step_ingredients)| step_ingredients)
        .collect::<Vec<steps_ingredients::Model>>();

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

    let units = ingredient_units::Entity::find()
        .filter(ingredient_units::Column::Id.is_in(si_unit_ids))
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

#[derive(Deserialize)]
pub struct WeekplanQuery {
    pub week: NaiveDate,
}

#[get("/weekplan/{user_id}/bring.json")]
pub async fn get_weekplan_bring(
    user_id: web::Path<i64>,
    params: Query<WeekplanQuery>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    let db = db.get_ref();

    let user = users::get_user_by_id(*user_id, db).await;

    if user.is_none() {
        return Ok(HttpResponse::NotFound().finish());
    }

    let user = user.unwrap();

    let weekplans = weekplan::list_weekplan(&params.week, &user, db)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let recipe_ids = weekplans.iter().map(|r| r.recipe_id).collect::<Vec<i64>>();
    let step_ingredients = entity::steps::Entity::find()
        .filter(entity::steps::Column::RecipeId.is_in(recipe_ids))
        .find_also_related(steps_ingredients::Entity)
        .all(db)
        .await
        .map_err(error::ErrorInternalServerError)?
        .into_iter()
        .filter(|(_step, step_ingredients)| step_ingredients.is_some())
        .map(|(step, step_ingredients)| (step, step_ingredients.unwrap()))
        .collect::<Vec<(steps::Model, steps_ingredients::Model)>>();

    let ingredient_ids = step_ingredients
        .iter()
        .map(|(_, step_ingredient)| step_ingredient.ingredient_id)
        .collect::<Vec<i64>>();

    let si_unit_ids = step_ingredients
        .iter()
        .filter_map(|(_, step_ingredient)| step_ingredient.unit_id)
        .collect::<Vec<i64>>();

    let ingredients = entity::ingredients::Entity::find()
        .filter(entity::ingredients::Column::Id.is_in(ingredient_ids))
        .all(db)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let units = ingredient_units::Entity::find()
        .filter(ingredient_units::Column::Id.is_in(si_unit_ids))
        .all(db)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let mut all_ingredients: HashMap<i64, HashMap<i64, BringInfo>> = HashMap::new();

    for weekplan_entry in weekplans {
        let step_ingredients = step_ingredients
            .iter()
            .filter(|(step, _)| step.recipe_id == weekplan_entry.recipe_id)
            .map(|(_, step_ingredient)| step_ingredient)
            .collect::<Vec<&steps_ingredients::Model>>();

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
                info.amount += amount * factor * (weekplan_entry.portions as f64);
            }

            if let Some(note) = &si.annotation {
                info.notes.push(note.clone());
            }
        }
    }

    let desc = BringRecipe {
        name: "Weekplan".to_owned(),
        author: user.name.unwrap_or(user.email),
        items: all_ingredients
            .values()
            .flat_map(|row| row.values())
            .map(|info| BringItem {
                item_id: info.ingredient.name.clone(),
                spec: amount_str(info.amount, &info.unit),
            })
            .collect(),
    };

    Ok(HttpResponse::Ok().json(desc))
}

fn amount_str(amount: f64, unit: &Option<ingredient_units::Model>) -> String {
    if amount > 0.0 {
        if let Some(unit) = &unit {
            let grams = amount * unit.base_value;
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
