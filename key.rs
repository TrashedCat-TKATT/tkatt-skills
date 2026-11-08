// The skill key opens the screen, or closes it when it is open. Banking and
// spending happen in the screen, where the pointer is free; nothing here
// hit-tests the world.
fn skill_trees_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    use crate::game::tkatt_skills as m;
    if !game.native_on(m::ID) || !pressed {
        return;
    }
    let panel_key = game.native_mod_key(
        m::ID,
        "panel",
        &game.native_mod_key(m::LEGACY_ID, "panel", "key.keyboard.k"),
    );
    if key != panel_key {
        return;
    }
    if game.native_mod_screen() == Some(m::ID) {
        game.native_mod_screen_close(m::ID);
        m::close_screen();
        return;
    }
    if !game.native_playing() {
        return;
    }
    // The emblem rides in the bundle's data folder; upload it once per open,
    // after a resource reload may have rebuilt the atlas.
    let rgba = crate::native_data!("tkatt_skills", "fist_icon.rgba");
    m::set_icon_ready(game.native_mod_image(24, 24, &rgba[..]));
    if game.native_mod_screen_open(m::ID) {
        m::open_screen();
    }
}
