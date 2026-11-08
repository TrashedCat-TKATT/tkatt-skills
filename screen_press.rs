// A press on the open skill screen: select a node, use the footer buttons,
// switch branches or hold Bank. The hook is `&self`, so the screen's state and
// the player's skills are updated here and the words and sounds wait for the
// tick.
fn skill_trees_screen_press(
    game: &crate::game::Game<'_>,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
    button: i32,
    modifiers: u32,
    canceled: &mut bool,
) -> std::ops::ControlFlow<()> {
    use crate::game::tkatt_skills as m;
    let _ = modifiers;
    if screen.id != "mod" || game.native_mod_screen() != Some(m::ID) || button != 0 {
        return std::ops::ControlFlow::Continue(());
    }
    let at = m::panel_origin(screen.rect);
    let local = [screen.mouse[0] - at[0], screen.mouse[1] - at[1]];
    let mut skills = m::load_once(|key| {
        game.native_setting(m::ID, key)
            .or_else(|| game.native_setting(m::LEGACY_ID, key))
    });
    let view = m::ScreenView::of(&skills, m::screen(), local);
    let Some(hit) = m::hit(&view, local) else {
        return std::ops::ControlFlow::Continue(());
    };
    // The screen took the click: it must not reach the world or the hotbar.
    *canceled = true;
    let before = skills.clone();
    let (state, notes) = m::press(&view, hit, &mut skills);
    m::set_screen(state);
    if skills != before {
        m::store(skills.clone());
        game.native_set_setting(m::ID, "state", &skills.encode());
    }
    for note in notes {
        m::note(note);
    }
    std::ops::ControlFlow::Continue(())
}
