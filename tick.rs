// Once a tick: say and play what the screen's `&self` hooks left behind, and
// bank experience while its Bank button is held. The vanilla command moves the
// experience, so levels, the bar and other players on the server all agree.
fn skill_trees_tick(game: &mut crate::game::Game<'_>) {
    use crate::game::tkatt_skills as m;
    if !game.native_on(m::ID) {
        return;
    }
    for note in m::take_notes() {
        match note {
            m::Note::Bar(text) => {
                game.native_client_action_bar(horizon_client::text::Text::literal(&text), true);
            }
            m::Note::Sound(name, volume, pitch) => {
                if let Some(p) = game.native_player() {
                    game.native_play_local_sound(name, [p.x, p.y, p.z], volume, pitch, 0);
                }
            }
            m::Note::Close => game.native_mod_screen_close(m::ID),
        }
    }
    let open = game.native_mod_screen() == Some(m::ID);
    let mut state = m::screen();
    if !open {
        // Escape and the gamepad close the screen without telling it.
        if state.banking {
            state.banking = false;
            m::set_screen(state);
        }
        return;
    }
    if !state.banking || !game.native_window() {
        return;
    }
    let mut skills = m::load_once(|key| {
        game.native_setting(m::ID, key)
            .or_else(|| game.native_setting(m::LEGACY_ID, key))
    });
    let available = game.experience.total.max(0) as u32;
    let before = skills.points;
    let taken = skills.pump(available);
    if taken == 0 {
        return;
    }
    game.run_command_quietly(&format!("experience add @s -{taken} points"));
    m::store(skills.clone());
    // A point gained is worth writing down and saying; the rest waits.
    if skills.points != before {
        game.native_set_setting(m::ID, "state", &skills.encode());
        game.native_chat(&format!(
            "Skill point earned ({} to spend). Open the skill trees to spend it.",
            skills.points
        ));
    }
}
