use bevy::{
    prelude::{App as BevyApp, AssetServer, Commands, Res, ResMut},
    window::WindowDescriptor,
    DefaultPlugins,
};
use kayak_ui::bevy::{BevyContext, BevyKayakUIPlugin, FontMapping, UICameraBundle};
use kayak_ui::core::{
    render, rsx,
    styles::{Style, StyleProp, Units},
    use_state, widget, EventType, Index, OnEvent,
};
use kayak_ui::widgets::{App, Button, Text, Window};

#[widget]
fn Counter(context: &mut KayakContext) {
    let text_styles = Style {
        bottom: StyleProp::Value(Units::Stretch(1.0)),
        left: StyleProp::Value(Units::Stretch(0.1)),
        right: StyleProp::Value(Units::Stretch(0.1)),
        top: StyleProp::Value(Units::Stretch(1.0)),
        width: StyleProp::Value(Units::Stretch(1.0)),
        height: StyleProp::Value(Units::Pixels(28.0)),
        ..Default::default()
    };

    let button_text_styles = Style {
        bottom: StyleProp::Value(Units::Stretch(1.0)),
        left: StyleProp::Value(Units::Stretch(1.0)),
        right: StyleProp::Value(Units::Stretch(1.0)),
        top: StyleProp::Value(Units::Stretch(1.0)),
        width: StyleProp::Value(Units::Pixels(67.0)),
        height: StyleProp::Value(Units::Pixels(39.0)),
        ..Default::default()
    };

    let (count, set_count, ..) = use_state!(0i32);
    let on_add = {
        let set_count = set_count.clone();
        OnEvent::new(move |_, event| {
            if event.event_type == EventType::Click {
                set_count(count + 1)
            }
        })
    };

    let on_sub = OnEvent::new(move |_, event| {
        if event.event_type == EventType::Click {
            set_count(count - 1)
        }
    });

    rsx! {
        <>
            <Window position={(50.0, 50.0)} size={(300.0, 300.0)} title={"Counter Example".to_string()}>
                <Text styles={Some(text_styles)} size={32.0} content={format!("Current Count: {}", count)}>{}</Text>
                <Button on_event={Some(on_add)}>
                    <Text styles={Some(button_text_styles)} size={24.0} content={"+1".to_string()}>{}</Text>
                </Button>
                <Button on_event={Some(on_sub)}>
                    <Text styles={Some(button_text_styles)} size={24.0} content={"-1".to_string()}>{}</Text>
                </Button>
            </Window>
        </>
    }
}

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn_bundle(UICameraBundle::new());

    font_mapping.add(asset_server.load("roboto.kayak_font"));

    let context = BevyContext::new(|context| {
        render! {
            <App>
                <Counter />
            </App>
        }
    });

    commands.insert_resource(context);
}

fn main() {
    BevyApp::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("UI Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(BevyKayakUIPlugin)
        .add_startup_system(startup)
        .run();
}
