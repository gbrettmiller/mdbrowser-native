//! Steps for features/settings.feature: appearance, the home page and the Home button.

use std::fs;

use cucumber::{given, then, when};
use mdbrowser::content;
use mdbrowser::session::{Event, Status};
use mdbrowser::theme::Appearance;

use crate::world::Browser;

fn appearance_named(name: &str) -> Appearance {
    Appearance::ALL.into_iter().find(|a| content::appearance(*a) == name).expect("an appearance menu label")
}

fn write_settings(w: &Browser, json: &str) {
    let path = w.home.path().join(".mdbrowser/settings.json");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, json).unwrap();
}

#[when(expr = "I choose the appearance {string}")]
#[given(expr = "I chose the appearance {string}")]
fn choose_appearance(w: &mut Browser, name: String) {
    w.send(Event::ChooseAppearance(appearance_named(&name)));
}

#[given(expr = "the stored appearance is {string}")]
fn stored_appearance(w: &mut Browser, key: String) {
    write_settings(w, &format!(r#"{{"appearance":"{key}"}}"#));
}

#[given(expr = "the stored settings are {string}")]
fn stored_settings(w: &mut Browser, json: String) {
    write_settings(w, &json.replace("\\\"", "\""));
}

#[then(expr = "the appearance is {string}")]
fn appearance_is(w: &mut Browser, name: String) {
    assert_eq!(w.state().settings.appearance, appearance_named(&name));
}

#[when(expr = "I set the home page to {string}")]
#[given(expr = "I set the home page to {string}")]
fn set_home(w: &mut Browser, value: String) {
    w.send(Event::ChooseHome(value));
}

#[when(expr = "I set the home page to the absolute path of {string}")]
#[given(expr = "I set the home page to the absolute path of {string}")]
fn set_home_path(w: &mut Browser, name: String) {
    let path = w.absolute(&name);
    w.send(Event::ChooseHome(path));
}

#[when(expr = "I clear the home page")]
fn clear_home(w: &mut Browser) {
    w.send(Event::ChooseHome(String::new()));
}

#[then(expr = "the home page is the absolute path of {string}")]
fn home_is_path(w: &mut Browser, name: String) {
    assert_eq!(w.state().settings.home, w.absolute(&name));
}

#[then("no home page is set")]
fn no_home(w: &mut Browser) {
    assert_eq!(w.state().settings.home, "");
}

#[when("I press Home")]
fn press_home(w: &mut Browser) {
    w.send(Event::Home);
}

#[when("the app starts with no address given")]
fn starts_bare(w: &mut Browser) {
    w.restart();
    w.send(Event::Home);
}

#[then(expr = "I see the welcome page")]
fn welcome_page(w: &mut Browser) {
    let Status::Ready(shown) = &w.state().status else { panic!("no document: {:?}", w.state().status) };
    assert_eq!(shown.title, "Welcome");
    assert_eq!(w.state().address(), "", "the welcome page has a blank address");
}

#[then("the welcome page is not in History")]
fn welcome_not_in_history(w: &mut Browser) {
    assert!(!w.history_titles().iter().any(|t| t == "Welcome"), "{:?}", w.history_titles());
}
