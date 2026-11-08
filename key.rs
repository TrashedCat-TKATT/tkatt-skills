// A key or mouse button: the panel's key opens and closes it, holding the
// bar banks experience, and a click on a row's "+" spends a point — or, for a
// big skill, unlocks a rank, buys one with Shift held, or toggles it.
fn skill_trees_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    use crate::game::tkatt_skills as m;
    if !game.native_on(m::ID) {
        return;
    }
    let mut skills = m::load_once(|k| game.native_setting(m::ID, k).or_else(|| game.native_setting(m::LEGACY_ID, k)));
    let (mut open, holding) = m::ui();
    let panel_key = game.native_mod_key(
        m::ID,
        "panel",
        &game.native_mod_key(m::LEGACY_ID, "panel", "key.keyboard.k"),
    );
    if key == panel_key {
        if pressed && game.native_playing() {
            open = !open;
            m::set_ui(open, holding);
        }
        return;
    }
    if key != "key.mouse.left" {
        return;
    }
    if !pressed {
        // Let go anywhere: the bar stops banking.
        m::set_ui(open, false);
        return;
    }
    if !game.native_playing() {
        return;
    }
    let origin = m::origin();
    let at = (game.cursor.0 - origin.0, game.cursor.1 - origin.1);
    let view = m::View::of(&skills, open, holding);
    match m::hit(&view, at) {
        Some(m::Hit::Bar) => m::set_ui(open, true),
        Some(m::Hit::Spend(row)) => {
            let tree = m::TREES[row];
            let buy = game.has_shift_down();
            let tune = m::tuning();
            let reply = match skills.press(tree, buy, tune) {
                m::Press::Spent(row) => {
                    format!("{} is now level {}", tree.title(), skills.levels[row])
                }
                m::Press::Unlocked(_) => format!("{} unlocked", tree.title()),
                m::Press::Ranked(row) => format!(
                    "{} is now rank {}/{}",
                    tree.title(),
                    skills.big_rank(row),
                    tune.max(row)
                ),
                m::Press::On(_) => format!("{} on", tree.title()),
                m::Press::Off(_) => format!("{} off", tree.title()),
                m::Press::NoPoints => {
                    "No skill points to spend: hold the bar to bank experience.".to_owned()
                }
                m::Press::Maxed => format!("{} is already at its last level.", tree.title()),
            };
            m::store(skills.clone());
            game.native_set_setting(m::ID, "state", &skills.encode());
            game.native_chat(&reply);
        }
        None => {}
    }
}
