mod business;
mod commands;
mod config;
mod debug_ui;
mod init;
mod money;
mod people;
mod stats;
mod ui;
mod user_input;

use crate::config::Config;
use crate::ui::{ManufacturerSort, PeopleSort};
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use serde::Deserialize;
use serde_json::from_reader;
use std::fs::File;

#[derive(Deserialize, Resource, Debug)]
pub struct BuildInfo {
    timestamp: String,
    version: String,
    commit_hash: String,
    branch_name: String,
}

fn main() {
    let file = File::open("build_info.json").expect("Failed to open file");
    let info: BuildInfo = from_reader(file).expect("Failed to deserialize");
    info!("Build Info: {:?}", info);
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            filter: "info,wgpu_core=warn,wgpu_hal=warn,sb3=info".into(),
            level: bevy::log::Level::WARN,
        }))
        .add_plugins((EguiPlugin, config::ConfigPlugin, FrameTimeDiagnosticsPlugin))
        .insert_resource(Days {
            days: 0,
            next_turn: false,
            last_update: 0.0,
        })
        .insert_resource(stats::PriceHistory::default())
        .insert_resource(init::Templates::default())
        .insert_resource(people::Names::default())
        .insert_resource(info)
        .insert_resource(debug_ui::Performance::new(100))
        .insert_resource(ui::SortOrder {
            manufacturers: ManufacturerSort::Name,
            people: PeopleSort::Name,
        })
        .add_event::<commands::GameCommand>()
        .add_systems(Startup, (init::init_names, init::init_manufacturers))
        .add_systems(First, user_input::input_system)
        .add_systems(PreUpdate, date_update_system.run_if(should_advance_day))
        .add_systems(
            Update,
            (
                // those system run in sequence
                business::salary_payout,
                business::execute_orders_for_manufacturers,
                business::produce,
                (business::create_buy_orders, business::create_sell_orders), // those run in parallel
                business::update_sell_order_prices,
                stats::add_sell_orders_to_history,
            )
                .chain()
                .run_if(next_turn),
        )
        .add_systems(Update, commands::command_system)
        .add_systems(Update, debug_ui::debug_window)
        .add_systems(
            Update,
            (
                ui::render_manufacturers_stats,
                ui::render_people_stats,
                ui::render_panels,
                ui::render_price_history,
                ui::render_template_editor,
                ui::render_todays_prices,
            ),
        )
        .add_systems(PostUpdate, turn_end_system)
        .run();
}

#[derive(Resource)]
pub struct Days {
    days: usize,
    next_turn: bool,
    last_update: f32,
}

impl Days {
    fn next_day(&mut self, time: &Res<Time>) {
        self.days += 1;
        self.next_turn = true;
        self.last_update = time.elapsed_seconds();
    }
}

fn date_update_system(mut days: ResMut<Days>, time: Res<Time>) {
    days.next_day(&time);
}

fn should_advance_day(time: Res<Time>, days: Res<Days>, config: Res<Config>) -> bool {
    if config.game.speed.value == 0.0 {
        return false;
    }
    time.elapsed_seconds() - days.last_update > config.game.speed.value
}

fn turn_end_system(mut days: ResMut<Days>) {
    days.next_turn = false;
}

fn next_turn(days: Res<Days>) -> bool {
    days.next_turn
}
