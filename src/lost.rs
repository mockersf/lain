use bevy::prelude::*;

use crate::{
    assets::{CloneWeak, UiAssets},
    game::stats::Stats,
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
            done: Timer::from_seconds(10.0, false),
        })
        .add_system_set(SystemSet::on_enter(CURRENT_STATE).with_system(setup))
        .add_system_set(SystemSet::on_exit(CURRENT_STATE).with_system(tear_down))
        .add_system_set(SystemSet::on_update(CURRENT_STATE).with_system(done));
    }
}

fn setup(mut commands: Commands, ui_handles: Res<UiAssets>, stats: Res<Stats>) {
    info!("Loading screen");

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

    let lost_text = commands
        .spawn_bundle(TextBundle {
            style: Style {
                size: Size {
                    height: Val::Px(40.),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text::from_section(
                "you lost".to_string(),
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
                format!("you survived {:.1} seconds", stats.time.elapsed_secs(),),
                TextStyle {
                    font: font_details,
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
        .id();
    commands
        .entity(inner_content)
        .push_children(&[lost_text, time_survived]);

    let panel_style = Style {
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
    };

    commands
        .spawn_bundle(bevy_ninepatch::NinePatchBundle {
            style: panel_style,
            nine_patch_data: bevy_ninepatch::NinePatchData::with_single_content(
                panel_handles.1,
                panel_handles.0,
                inner_content,
            ),
            ..Default::default()
        })
        .insert(ScreenTag);
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
