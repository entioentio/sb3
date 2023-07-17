use std::collections::HashMap;

use bevy::core::Name;
use bevy::prelude::{Commands, Entity, Query, Res, ResMut};
use bevy_egui::egui::{Align, Layout, Window};
use bevy_egui::EguiContexts;
use egui_extras::{Column, TableBuilder};

use macros::measured;

use crate::business::{BuyOrder, ItemType, Manufacturer, SellOrder, Wallet, Worker};
use crate::logs::Pinned;
use crate::money::Money;
use crate::stats::PriceHistory;
use crate::ui::debug::Performance;
use crate::ui::main_layout::UiState;
use crate::ui::utilities::{count_items, items_to_string, label_with_hover_text};

#[allow(clippy::too_many_arguments)]
#[measured]
pub fn render_manufacturers_stats(
    mut egui_context: EguiContexts,
    manufacturers: Query<(Entity, &Name, &Wallet, &Manufacturer)>,
    sell_orders: Query<&SellOrder>,
    buy_orders: Query<&BuyOrder>,
    names: Query<&Name>,
    workers: Query<&Worker>,
    pins: Query<&Pinned>,
    mut ui_state: ResMut<UiState>,
    price_history: Res<PriceHistory>,
    mut commands: Commands,
) {
    Window::new("Manufacturers").show(egui_context.ctx_mut(), |ui| {
        let mut owner_counts: HashMap<Entity, usize> = HashMap::new();

        for order in sell_orders.iter() {
            *owner_counts.entry(order.seller).or_insert(0) += 1;
        }
        let table = TableBuilder::new(ui)
            // .striped(self.striped)
            // .resizable(self.resizable)
            .cell_layout(Layout::left_to_right(Align::Center))
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::initial(80.0).range(80.0..=200.0))
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::remainder())
            .min_scrolled_height(0.0);

        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    if ui.button("Pin").clicked() {
                        ui_state.manufacturers_pinned = !ui_state.manufacturers_pinned;
                    }
                });
                header.col(|ui| {
                    if ui.button("Name").clicked() {
                        ui_state.manufacturers = ManufacturerSort::Name;
                    }
                });
                header.col(|ui| {
                    if ui.button("Produces").clicked() {
                        ui_state.manufacturers = ManufacturerSort::Production;
                    }
                });
                header.col(|ui| {
                    if ui.button("Money").clicked() {
                        ui_state.manufacturers = ManufacturerSort::Money;
                    }
                });
                header.col(|ui| {
                    if ui.button("Workers").clicked() {
                        ui_state.manufacturers = ManufacturerSort::Workers;
                    }
                });
                header.col(|ui| {
                    if ui.button("Items").clicked() {
                        ui_state.manufacturers = ManufacturerSort::Items;
                    }
                });
                header.col(|ui| {
                    if ui.button("Items to sell").clicked() {
                        ui_state.manufacturers = ManufacturerSort::ItemsToSell;
                    }
                });
                header.col(|ui| {
                    if ui.button("On market").clicked() {
                        ui_state.manufacturers = ManufacturerSort::OnMarket;
                    }
                });
                header.col(|ui| {
                    if ui.button("Buy orders").clicked() {
                        ui_state.manufacturers = ManufacturerSort::BuyOrders;
                    }
                });
            })
            .body(|mut body| {
                let buy_order_by_type: HashMap<ItemType, usize> = buy_orders
                    .iter()
                    .map(|x| x.item_type.clone())
                    .fold(HashMap::new(), |mut acc, x| {
                        *acc.entry(x).or_insert(0) += 1;
                        acc
                    });
                let buy_order_by_type_and_buyer: HashMap<(ItemType, Name), usize> = buy_orders
                    .iter()
                    .map(|x| (x.item_type.clone(), names.get(x.buyer).unwrap().clone()))
                    .fold(HashMap::new(), |mut acc, x| {
                        *acc.entry(x).or_insert(0) += 1;
                        acc
                    });
                let mut buy_order_vec: Vec<((ItemType, Name), usize)> =
                    buy_order_by_type_and_buyer.into_iter().collect();
                buy_order_vec.sort_by(|((_, a_name), a), ((_, b_name), b)| {
                    b.cmp(a).then_with(|| a_name.cmp(b_name))
                });
                let mut rows = manufacturers
                    .iter()
                    .map(|(entity, name, wallet, manufacturer)| ManufacturerRow {
                        entity,
                        pinned: pins.get(entity).is_ok(),
                        name: name.to_string(),
                        production: manufacturer.production_cycle.output.0.name.to_string(),
                        production_text: format!("{}", manufacturer.production_cycle),
                        money: wallet.money,
                        workers: manufacturer.hired_workers.len(),
                        workers_text: manufacturer
                            .hired_workers
                            .iter()
                            .map(|x| {
                                format!(
                                    "{} ({})",
                                    names.get(*x).unwrap(),
                                    workers.get(*x).unwrap().salary
                                )
                            })
                            .collect::<Vec<String>>()
                            .join("\n"),
                        items: count_items(&manufacturer.assets.items),
                        items_text: items_to_string(&manufacturer.assets.items),
                        items_to_sell: manufacturer.assets.items_to_sell.len(),
                        on_market: *owner_counts.get(&entity).unwrap_or(&0),
                        on_market_text: price_history
                            .prices
                            .get(&manufacturer.production_cycle.output.0)
                            .and_then(|x| x.last())
                            .map_or_else(
                                || "".to_string(),
                                |price_stats| format!("{}", price_stats),
                            ),
                        buy_orders: *buy_order_by_type
                            .get(&manufacturer.production_cycle.output.0)
                            .unwrap_or(&0),
                        buy_orders_text: buy_order_vec
                            .iter()
                            .filter(|x| x.0 .0 == manufacturer.production_cycle.output.0)
                            .map(|x| format!("{}: {}", x.0 .1, x.1))
                            .collect::<Vec<_>>()
                            .join("\n"),
                    })
                    .collect::<Vec<_>>();
                match ui_state.manufacturers {
                    ManufacturerSort::Name => {
                        rows.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap())
                    }
                    ManufacturerSort::Production => {
                        rows.sort_by(|a, b| a.production.partial_cmp(&b.production).unwrap())
                    }
                    ManufacturerSort::Money => {
                        rows.sort_by(|a, b| b.money.partial_cmp(&a.money).unwrap())
                    }
                    ManufacturerSort::Workers => {
                        rows.sort_by(|a, b| b.workers.partial_cmp(&a.workers).unwrap())
                    }
                    ManufacturerSort::Items => {
                        rows.sort_by(|a, b| b.items.partial_cmp(&a.items).unwrap())
                    }
                    ManufacturerSort::ItemsToSell => {
                        rows.sort_by(|a, b| b.items_to_sell.partial_cmp(&a.items_to_sell).unwrap())
                    }
                    ManufacturerSort::OnMarket => {
                        rows.sort_by(|a, b| b.on_market.partial_cmp(&a.on_market).unwrap())
                    }
                    ManufacturerSort::BuyOrders => {
                        rows.sort_by(|a, b| b.buy_orders.partial_cmp(&a.buy_orders).unwrap())
                    }
                }

                for r in rows.iter() {
                    body.row(20.0, |mut row| {
                        row.col(|ui| {
                            if r.pinned {
                                if ui.button("U").on_hover_text("Unpin this person").clicked() {
                                    commands.entity(r.entity).remove::<Pinned>();
                                }
                            } else if ui.button("P").on_hover_text("Pin this person").clicked() {
                                commands.entity(r.entity).insert(Pinned {});
                            }
                        });
                        row.col(|ui| {
                            ui.label(&r.name);
                        });
                        row.col(|ui| {
                            ui.label(&r.production).on_hover_text(&r.production_text);
                        });
                        row.col(|ui| {
                            ui.label(&r.money.to_string());
                        });
                        row.col(|ui| {
                            label_with_hover_text(ui, r.workers, &r.workers_text);
                        });
                        row.col(|ui| {
                            label_with_hover_text(ui, r.items, &r.items_text);
                        });
                        row.col(|ui| {
                            ui.label(&r.items_to_sell.to_string());
                        });
                        row.col(|ui| {
                            if r.on_market_text.is_empty() {
                                label_with_hover_text(ui, r.on_market, "No price history yet");
                            } else {
                                label_with_hover_text(ui, r.on_market, &r.on_market_text);
                            }
                        });
                        row.col(|ui| {
                            label_with_hover_text(ui, r.buy_orders, &r.buy_orders_text);
                        });
                    });
                }
            });
    });
}

pub enum ManufacturerSort {
    Name,
    Money,
    Workers,
    Items,
    ItemsToSell,
    OnMarket,
    BuyOrders,
    Production,
}

struct ManufacturerRow {
    entity: Entity,
    pinned: bool,
    name: String,
    production: String,
    money: Money,
    workers: usize,
    items: usize,
    items_to_sell: usize,
    on_market: usize,
    on_market_text: String,
    buy_orders: usize,
    items_text: String,
    buy_orders_text: String,
    production_text: String,
    workers_text: String,
}
