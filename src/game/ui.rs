use bevy::prelude::*;
use tracing::info;

use crate::{
    assets::{CloneWeak, UiAssets},
    game::stats::GameTag,
    ui_helper::button::{ButtonId, ButtonText},
    GameState,
};

use super::{stats::Stats, PlayingState};

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system_set(SystemSet::on_enter(GameState::Playing).with_system(setup))
            .add_system_set(
                SystemSet::on_update(GameState::Playing)
                    .with_system(button_system)
                    .with_system(update_ui),
            );
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

#[derive(Component)]
struct LiveMarker;

#[derive(Component)]
struct CreditsMarker;

fn setup(
    mut commands: Commands,
    ui_handles: Res<UiAssets>,
    buttons: Res<Assets<crate::ui_helper::button::Button>>,
    stats: Res<Stats>,
) {
    info!("loading UI");

    let button_handle = ui_handles.button_handle.clone_weak();
    let button = buttons.get(&button_handle).unwrap();
    let font = ui_handles.font_sub.clone_weak();
    let material = ui_handles.font_material.clone_weak();
    let panel_handles = ui_handles.panel_handle.clone_weak();

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
        font.clone(),
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

    let lives_text = commands
        .spawn_bundle(TextBundle {
            style: Style {
                size: Size {
                    height: Val::Px(20.),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text::from_sections([
                TextSection {
                    value: "lives: ".to_string(),
                    style: TextStyle {
                        font: font.clone(),
                        color: crate::ui_helper::ColorScheme::TEXT,
                        font_size: 20.,
                        ..Default::default()
                    },
                },
                TextSection {
                    value: format!("{}", stats.life),
                    style: TextStyle {
                        font: font.clone(),
                        color: crate::ui_helper::ColorScheme::TEXT,
                        font_size: 20.,
                        ..Default::default()
                    },
                },
            ]),
            ..Default::default()
        })
        .insert(LiveMarker)
        .id();
    let credits_text = commands
        .spawn_bundle(TextBundle {
            style: Style {
                size: Size {
                    height: Val::Px(20.),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text::from_sections([
                TextSection {
                    value: "credits: ".to_string(),
                    style: TextStyle {
                        font: font.clone(),
                        color: crate::ui_helper::ColorScheme::TEXT,
                        font_size: 20.,
                        ..Default::default()
                    },
                },
                TextSection {
                    value: format!("{}", stats.credits),
                    style: TextStyle {
                        font: font,
                        color: crate::ui_helper::ColorScheme::TEXT,
                        font_size: 20.,
                        ..Default::default()
                    },
                },
            ]),
            ..Default::default()
        })
        .insert(CreditsMarker)
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
        .push_children(&[lives_text, credits_text])
        .id();
    let panel = commands
        .spawn_bundle(bevy_ninepatch::NinePatchBundle {
            style: Style {
                size: Size::new(Val::Px(120.), Val::Px(80.)),
                align_content: AlignContent::Stretch,
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            nine_patch_data: bevy_ninepatch::NinePatchData::with_single_content(
                panel_handles.1,
                panel_handles.0,
                inner_content,
            ),
            ..Default::default()
        })
        .id();

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .insert(GameTag)
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
                            height: Val::Px(250.0),
                        },
                        flex_direction: FlexDirection::ColumnReverse,
                        justify_content: JustifyContent::SpaceAround,
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    color: UiColor(Color::NONE),
                    ..default()
                })
                .push_children(&[panel])
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
    interaction_query: Query<(&Interaction, &ButtonId<UiButtons>, Changed<Interaction>)>,
    mut text_query: Query<(&mut Text, &ButtonText<UiButtons>)>,
    mut camera: Query<&mut Transform, With<Camera>>,
    time: Res<Time>,
    mut playing_state: ResMut<State<PlayingState>>,
    mut building: Local<bool>,
) {
    if *playing_state.current() != PlayingState::SwitchingPlane {
        for (interaction, button_id, changed) in interaction_query.iter() {
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
                        playing_state.set(PlayingState::SwitchingPlane).unwrap();
                        for (mut text, button) in &mut text_query {
                            if button.0 == UiButtons::BuildTower {
                                text.sections[0].value = UiButtons::BuildTower.into();
                                *building = false;
                            }
                        }
                    }
                    (UiButtons::BuildTower, true) => {
                        if *building {
                            playing_state.set(PlayingState::Playing).unwrap();
                            for (mut text, button) in &mut text_query {
                                if button.0 == UiButtons::BuildTower {
                                    text.sections[0].value = UiButtons::BuildTower.into();
                                    *building = false;
                                }
                            }
                        } else {
                            playing_state.set(PlayingState::Building).unwrap();
                            for (mut text, button) in &mut text_query {
                                if button.0 == UiButtons::BuildTower {
                                    text.sections[0].value = UiButtons::Cancel.into();
                                    *building = true;
                                }
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
    }
}

fn update_ui(
    stats: Res<Stats>,
    mut live_text: Query<&mut Text, (With<LiveMarker>, Without<CreditsMarker>)>,
    mut credits_text: Query<&mut Text, With<CreditsMarker>>,
) {
    live_text.single_mut().sections[1].value = format!("{}", stats.life);
    credits_text.single_mut().sections[1].value = format!("{}", stats.credits);
}
