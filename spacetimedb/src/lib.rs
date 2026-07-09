use spacetimedb::{
    Identity, ReducerContext, ScheduleAt, SpacetimeType, Table, Timestamp, rand::Rng,
};
use std::time::Duration;

const FOOD_MASS_MIN: i32 = 2;
const FOOD_MASS_MAX: i32 = 4;
const TARGET_FOOD_COUNT: usize = 600;

#[spacetimedb::table(accessor = spawn_food_timer, scheduled(spawn_food))]
pub struct SpawnFoodTimer {
    #[primary_key]
    #[auto_inc]
    scheduled_id: u64,
    scheduled_at: ScheduleAt,
}

#[spacetimedb::table(accessor = config, public)]
pub struct Config {
    #[primary_key]
    pub id: i32,
    pub world_size: i64,
}

#[derive(SpacetimeType, Clone, Debug)]
pub struct DbVector2 {
    pub x: f32,
    pub y: f32,
}

#[spacetimedb::table(accessor = entity, public)]
#[derive(Debug, Clone)]
pub struct Entity {
    #[primary_key]
    pub entity_id: i32,
    pub position: DbVector2,
    pub mass: i32,
}

#[spacetimedb::table(accessor = circle, public)]
pub struct Circle {
    #[primary_key]
    pub entity_id: i32,
    #[index(btree)]
    pub player_id: i32,
    pub direction: DbVector2,
    pub speed: f32,
    pub last_split_time: Timestamp,
}

#[spacetimedb::table(accessor = food, public)]
pub struct Food {
    #[primary_key]
    pub entity_id: i32,
}

#[spacetimedb::table(accessor = player, public)]
#[derive(Debug, Clone)]
pub struct Player {
    #[primary_key]
    pub identity: Identity,
    #[unique]
    #[auto_inc]
    player_id: i32,
    name: String,
}

#[spacetimedb::reducer(init)]
pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    log::info!("Initializing spacetime database");

    ctx.db.config().try_insert(Config {
        id: 0,
        world_size: 1000,
    })?;

    ctx.db.spawn_food_timer().try_insert(SpawnFoodTimer {
        scheduled_id: 0,
        scheduled_at: ScheduleAt::Interval(Duration::from_millis(500).into()),
    })?;

    Ok(())
}

#[spacetimedb::reducer(client_connected)]
pub fn connect(ctx: &ReducerContext) -> Result<(), String> {
    log::debug!("{} just connected", ctx.sender());
    Ok(())
}

#[spacetimedb::reducer]
pub fn spawn_food(ctx: &ReducerContext, _timer: SpawnFoodTimer) -> Result<(), String> {
    if ctx.db.player().count() == 0 {
        return Ok(());
    }

    let world_size = ctx.db.config().id().find(0).ok_or("err")?.world_size;
    let mut rng = ctx.rng();
    let mut food_count = ctx.db.food().count();

    while food_count < TARGET_FOOD_COUNT as u64 {
        let food_mass = rng.gen_range(FOOD_MASS_MIN..=FOOD_MASS_MAX);
        let food_radius = mass_to_radius(food_mass);
        let x = rng.gen_range(food_radius..world_size as f32 - food_radius);
        let y = rng.gen_range(food_radius..world_size as f32 - food_radius);

        let entity = ctx.db.entity().try_insert(Entity {
            entity_id: 0,
            position: DbVector2 { x, y },
            mass: food_mass,
        })?;

        ctx.db.food().try_insert(Food {
            entity_id: entity.entity_id,
        })?;

        food_count += 1;

        log::info!("Spawned food! {}", entity.entity_id);
    }

    Ok(())
}

fn mass_to_radius(mass: i32) -> f32 {
    (mass as f32).sqrt()
}