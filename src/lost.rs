use std::cmp::Ordering;

use bevy::prelude::*;
use bevy_jornet::{Leaderboard, Score};

use crate::{
    assets::{CloneWeak, UiAssets},
    game::stats::Stats,
    ui_helper::ColorScheme,
};

const CURRENT_STATE: crate::GameState = crate::GameState::Lost;

#[derive(Component)]
struct ScreenTag;

struct Screen {
    done: Timer,
}

pub(crate) struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Screen {
            done: Timer::from_seconds(20.0, false),
        })
        .add_system_set(SystemSet::on_enter(CURRENT_STATE).with_system(setup))
        .add_system_set(SystemSet::on_exit(CURRENT_STATE).with_system(tear_down))
        .add_system_set(
            SystemSet::on_update(CURRENT_STATE)
                .with_system(done)
                .with_system(display_scores),
        );
    }
}

fn setup(
    mut commands: Commands,
    ui_handles: Res<UiAssets>,
    stats: Res<Stats>,
    leaderboard: Res<Leaderboard>,
) {
    info!("Loading screen");

    commands.insert_resource(Screen {
        done: Timer::from_seconds(20.0, false),
    });

    leaderboard.send_score(stats.killed as f32);
    leaderboard.refresh_leaderboard();

    let panel_handles = ui_handles.panel_handle.clone_weak();
    let font = ui_handles.font_main.clone_weak();
    let font_details = ui_handles.font_sub.clone_weak();

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect::<Val> {
                    left: Val::Percent(20.),
                    right: Val::Undefined,
                    bottom: Val::Undefined,
                    top: Val::Percent(25.),
                },
                size: Size::<Val> {
                    height: Val::Px(95.),
                    width: Val::Auto,
                },
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            color: UiColor(Color::NONE),
            ..Default::default()
        })
        .with_children(|title_parent| {
            title_parent.spawn_bundle(TextBundle {
                style: Style {
                    size: Size {
                        height: Val::Px(75.),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                text: Text::from_section(
                    "Lain".to_string(),
                    TextStyle {
                        font: font.clone(),
                        color: crate::ui_helper::ColorScheme::TEXT,
                        font_size: 75.,
                        ..Default::default()
                    },
                ),
                ..Default::default()
            });
        })
        .insert(ScreenTag);

    let time_survived = commands
        .spawn_bundle(TextBundle {
            style: Style {
                size: Size {
                    height: Val::Px(75.),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text::from_section(
                format!("you survived {} seconds", stats.time.elapsed().as_secs()),
                TextStyle {
                    font: font_details.clone(),
                    color: crate::ui_helper::ColorScheme::TEXT,
                    font_size: 40.,
                    ..Default::default()
                },
            ),
            ..Default::default()
        })
        .id();
    let zombie_killed = commands
        .spawn_bundle(TextBundle {
            style: Style {
                size: Size {
                    height: Val::Px(75.),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text::from_section(
                format!("and killed {:.1} zombies", stats.killed),
                TextStyle {
                    font: font_details.clone_weak(),
                    color: crate::ui_helper::ColorScheme::TEXT,
                    font_size: 40.,
                    ..Default::default()
                },
            ),
            ..Default::default()
        })
        .id();

    let inner_content = commands
        .spawn_bundle(NodeBundle {
            color: UiColor(Color::NONE),
            style: Style {
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            ..Default::default()
        })
        .push_children(&[time_survived, zombie_killed])
        .id();

    commands
        .spawn_bundle(bevy_ninepatch::NinePatchBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect::<Val> {
                    left: Val::Px(200.),
                    right: Val::Undefined,
                    bottom: Val::Percent(15.),
                    top: Val::Undefined,
                },
                margin: UiRect::all(Val::Px(0.)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                size: Size::new(Val::Px(400.), Val::Px(300.)),
                align_content: AlignContent::Stretch,
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            nine_patch_data: bevy_ninepatch::NinePatchData::with_single_content(
                panel_handles.1.clone_weak(),
                panel_handles.0.clone_weak(),
                inner_content,
            ),
            ..Default::default()
        })
        .insert(ScreenTag);

    let leaderboard_content = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(180.0), Val::Undefined),
                        flex_direction: FlexDirection::ColumnReverse,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    color: Color::NONE.into(),
                    ..default()
                })
                .insert(LeaderboardMarker::Player);
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(80.0), Val::Undefined),
                        flex_direction: FlexDirection::ColumnReverse,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    color: Color::NONE.into(),
                    ..default()
                })
                .insert(LeaderboardMarker::Score);
        })
        .id();

    commands
        .spawn_bundle(bevy_ninepatch::NinePatchBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect::<Val> {
                    left: Val::Percent(53.),
                    right: Val::Undefined,
                    bottom: Val::Percent(15.),
                    top: Val::Undefined,
                },
                margin: UiRect::all(Val::Px(0.)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                size: Size::new(Val::Px(400.), Val::Px(300.)),
                align_content: AlignContent::Stretch,
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            nine_patch_data: bevy_ninepatch::NinePatchData::with_single_content(
                panel_handles.1,
                panel_handles.0,
                leaderboard_content,
            ),
            ..Default::default()
        })
        .insert(ScreenTag);

    commands
        .spawn_bundle(
            TextBundle::from_sections([
                TextSection {
                    value: "you are: ".to_string(),
                    style: TextStyle {
                        font: font_details.clone_weak(),
                        font_size: 20.0,
                        color: ColorScheme::TEXT_DARK,
                    },
                },
                TextSection {
                    value: leaderboard
                        .get_player()
                        .map(|p| p.name.clone())
                        .unwrap_or_default(),
                    style: TextStyle {
                        font: font_details,
                        font_size: 25.0,
                        color: ColorScheme::TEXT_DARK,
                    },
                },
            ])
            .with_style(Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Px(10.0),
                    bottom: Val::Px(10.0),
                    ..default()
                },
                ..default()
            }),
        )
        .insert_bundle((ScreenTag,));
}

fn tear_down(mut commands: Commands, query: Query<Entity, With<ScreenTag>>) {
    info!("tear down");

    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn done(time: Res<Time>, mut screen: ResMut<Screen>, mut state: ResMut<State<crate::GameState>>) {
    if screen.done.tick(time.delta()).finished() {
        state.set(crate::GameState::Menu).unwrap();
    }
}

#[derive(Component)]
enum LeaderboardMarker {
    Score,
    Player,
}

fn display_scores(
    leaderboard: Res<Leaderboard>,
    mut commands: Commands,
    root_ui: Query<(Entity, &LeaderboardMarker)>,
    assets: Res<UiAssets>,
    stats: Res<Stats>,
) {
    if leaderboard.is_changed() {
        let mut scores = leaderboard.get_leaderboard();
        scores.push(Score {
            score: stats.killed as f32,
            player: leaderboard
                .get_player()
                .map(|p| p.name.clone())
                .unwrap_or_default(),
            meta: None,
            timestamp: "0".to_string(),
        });
        scores
            .sort_unstable_by(|s1, s2| s2.score.partial_cmp(&s1.score).unwrap_or(Ordering::Equal));
        scores.truncate(10);
        for (root_entity, marker) in &root_ui {
            commands.entity(root_entity).despawn_descendants();
            for score in &scores {
                commands.entity(root_entity).with_children(|parent| {
                    parent.spawn_bundle(TextBundle::from_section(
                        match marker {
                            LeaderboardMarker::Score => format!("{} ", score.score),
                            LeaderboardMarker::Player => score.player.clone(),
                        },
                        TextStyle {
                            font: assets.font_sub.clone_weak(),
                            font_size: 25.0,
                            color: ColorScheme::TEXT,
                        },
                    ));
                });
            }
        }
    }
}
