// A click on the settings page: only "Reset trees" does anything.
fn skill_trees_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use crate::game::tkatt_skills as m;
    if mod_id != m::ID || button != 5 + m::BIG_ROWS {
        return;
    }
    let mut skills = m::load_once(|key| game.native_setting(m::ID, key).or_else(|| game.native_setting(m::LEGACY_ID, key)));
    skills.run(m::Op::Reset);
    m::store(skills.clone());
    game.native_set_setting(m::ID, "state", &skills.encode());
    game.native_chat("tkatt-skills: every tree reset; points kept.");
}
