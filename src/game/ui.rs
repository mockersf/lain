use bevy::{
    prelude::{
        default, Assets, BuildChildren, Button, Camera, Changed, Color, Commands, NodeBundle,
        Query, Res, ResMut, State, SystemSet, Transform, With,
    },
    text::Text,
    time::Time,
    ui::{
        FlexDirection, Interaction, JustifyContent, PositionType, Size, Style, UiColor, UiRect, Val,
    },
};
use tracing::info;

use crate::{
    assets::UiAssets,
    ui_helper::button::{ButtonId, ButtonText},
    GameState,
};

use super::PlayingState;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system_set(SystemSet::on_enter(GameState::Playing).with_system(setup))
            .add_system_set(SystemSet::on_update(GameState::Playing).with_system(button_system));
    }
}

#[derive(Clone, Copy, PartialEq)]
enum UiButtons {
    ZoomIn,
    ZoomOut,
    SwitchPlane,
    BuildTower,
    Cancel,
}

impl From<UiButtons> for String {
    fn from(button: UiButtons) -> Self {
        match button {
            UiButtons::ZoomIn => {
                material_icons::icon_to_char(material_icons::Icon::ZoomIn).to_string()
            }
            UiButtons::ZoomOut => {
                material_icons::icon_to_char(material_icons::Icon::ZoomOut).to_string()
            }
            UiButtons::SwitchPlane => "Switch Plane".to_string(),
            UiButtons::BuildTower => "Build".to_string(),
            UiButtons::Cancel => "Cancel".to_string(),
        }
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
    let font = ui_handles.font_sub.clone_weak();
    let material = ui_handles.font_material.clone_weak();

    let build_button = button.add(
        &mut commands,
        120.,
        40.,
        UiRect::all(Val::Auto),
        font.clone(),
        UiButtons::BuildTower,
        20.,
    );
    let switch_button = button.add(
        &mut commands,
        120.,
        40.,
        UiRect::all(Val::Auto),
        font,
        UiButtons::SwitchPlane,
        20.,
    );

    let zoom_in_button = button.add(
        &mut commands,
        40.,
        40.,
        UiRect::all(Val::Auto),
        material.clone(),
        UiButtons::ZoomIn,
        30.,
    );
    let zoom_out_button = button.add(
        &mut commands,
        40.,
        40.,
        UiRect::all(Val::Auto),
        material,
        UiButtons::ZoomOut,
        30.,
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
                            height: Val::Px(150.0),
                        },
                        flex_direction: FlexDirection::ColumnReverse,
                        justify_content: JustifyContent::SpaceAround,
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    color: UiColor(Color::NONE),
                    ..default()
                })
                .with_children(|builder| {
                    builder
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceAround,
                                ..default()
                            },
                            color: UiColor(Color::NONE),
                            ..default()
                        })
                        .push_children(&[zoom_in_button, zoom_out_button]);
                })
                .push_children(&[build_button, switch_button]);
        });
}

fn button_system(
    mut interaction_query: Query<(
        &Button,
        &Interaction,
        &mut ButtonId<UiButtons>,
        Changed<Interaction>,
    )>,
    mut text_query: Query<(&mut Text, &ButtonText<UiButtons>)>,
    mut camera: Query<&mut Transform, With<Camera>>,
    time: Res<Time>,
    mut playing_state: ResMut<State<PlayingState>>,
) {
    for (_button, interaction, mut button_id, changed) in interaction_query.iter_mut() {
        if *interaction == Interaction::Clicked {
            match (button_id.0, changed) {
                (UiButtons::ZoomIn, _) => {
                    if camera.single().translation.y > 2.0 {
                        camera.single_mut().translation.y -= time.delta_seconds() * 2.0;
                    }
                }
                (UiButtons::ZoomOut, _) => {
                    if camera.single().translation.y < 20.0 {
                        camera.single_mut().translation.y += time.delta_seconds() * 2.0;
                    }
                }
                (UiButtons::SwitchPlane, true) => {
                    if *playing_state.current() != PlayingState::SwitchingPlane {
                        playing_state.set(PlayingState::SwitchingPlane).unwrap();
                    }
                }
                (UiButtons::BuildTower, true) => {
                    playing_state.set(PlayingState::Building).unwrap();
                    button_id.0 = UiButtons::Cancel;
                    for (mut text, button) in &mut text_query {
                        if button.0 == UiButtons::BuildTower {
                            text.sections[0].value = UiButtons::Cancel.into()
                        }
                    }
                }
                (UiButtons::Cancel, true) => {
                    playing_state.set(PlayingState::Playing).unwrap();
                    button_id.0 = UiButtons::BuildTower;
                    for (mut text, button) in &mut text_query {
                        if button.0 == UiButtons::BuildTower {
                            text.sections[0].value = UiButtons::BuildTower.into()
                        }
                    }
                }
                _ => (),
            }
        }
    }
}
