// Letting go of the Bank button stops banking. The press was cancelled, so the
// release is too, and nothing behind the screen sees it.
fn skill_trees_screen_release(
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
    let mut state = m::screen();
    if state.banking {
        state.banking = false;
        state.held_ticks = 0;
        m::set_screen(state);
        *canceled = true;
    }
    std::ops::ControlFlow::Continue(())
}
