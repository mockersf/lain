use bevy::{
    prelude::{default, Assets, BuildChildren, Color, Commands, NodeBundle, Res, SystemSet},
    ui::{
        AlignContent, AlignItems, FlexDirection, JustifyContent, PositionType, Size, Style,
        UiColor, UiRect, Val,
    },
};
use tracing::info;

use crate::{assets::UiAssets, GameState};

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system_set(SystemSet::on_enter(GameState::Playing).with_system(setup));
    }
}

fn setup(
    mut commands: Commands,
    ui_handles: Res<UiAssets>,
    buttons: Res<Assets<crate::ui_helper::button::Button>>,
) {
    info!("loading UI");

    let button_handle = ui_handles.button_handle.clone_weak();
    let button = buttons.get(&button_handle).unwrap();
    let font = ui_handles.font_sub_handle.clone_weak();

    let build_button = button.add(
        &mut commands,
        120.,
        40.,
        UiRect::all(Val::Auto),
        font.clone(),
        "Build",
        20.,
    );
    let switch_button = button.add(
        &mut commands,
        120.,
        40.,
        UiRect::all(Val::Auto),
        font.clone(),
        "Switch Plane",
        20.,
    );

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .with_children(|commands| {
            commands
                .spawn_bundle(NodeBundle {
                    style: Style {
                        position: UiRect {
                            left: Val::Px(20.0),
                            top: Val::Px(20.0),
                            ..default()
                        },
                        size: Size {
                            width: Val::Undefined,
                            height: Val::Px(100.0),
                        },
                        flex_direction: FlexDirection::ColumnReverse,
                        justify_content: JustifyContent::SpaceAround,
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    color: UiColor(Color::NONE),
                    ..default()
                })
                .push_children(&[build_button, switch_button]);
        });
}
