// Once a tick: while the bar is held, take experience from the player and
// bank it. The vanilla command moves the experience, so levels, the bar and
// other players on the server all agree.
fn skill_trees_tick(game: &mut crate::game::Game<'_>) {
    use crate::game::tkatt_skills as m;
    if !game.native_on(m::ID) {
        return;
    }
    let (_, holding) = m::ui();
    if !holding || !game.native_window() {
        return;
    }
    let mut skills = m::load_once(|key| game.native_setting(m::ID, key).or_else(|| game.native_setting(m::LEGACY_ID, key)));
    let available = game.experience.total.max(0) as u32;
    let before = skills.points;
    let taken = skills.pump(available);
    if taken == 0 {
        return;
    }
    game.run_command_quietly(&format!("experience add @s -{taken} points"));
    m::store(skills.clone());
    // A point gained is worth writing down; the rest waits for the release.
    if skills.points != before {
        game.native_set_setting(m::ID, "state", &skills.encode());
        game.native_chat(&format!(
            "Skill point earned ({} to spend). Press the skill key to spend it.",
            skills.points
        ));
    }
}
