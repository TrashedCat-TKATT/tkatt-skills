// Fortune: while the big skill is on, some broken blocks drop one more of the
// first item they already drop. The roll and the choice of stack are
// `fortune_roll` and `fortune_target` in runtime.rs, where they are
// unit-tested; this helper only applies the result to the game's loot list.
fn skill_trees_fortune(
    value: Vec<horizon_client::items::ItemStack>,
    game: &crate::game::Game<'_>,
    pos: [i32; 3],
) -> Vec<horizon_client::items::ItemStack> {
    use crate::game::tkatt_skills as m;
    let mut loot = value;
    if loot.is_empty() || !game.native_on(m::ID) {
        return loot;
    }
    // The drops are rolled on the game's server thread, where the screen's
    // load has not happened: read the state here (once).
    let skills = m::load_once(|key| {
        game.native_setting(m::ID, key)
            .or_else(|| game.native_setting(m::LEGACY_ID, key))
    });
    if !skills.fortune_on() {
        return loot;
    }
    let chance = skills.fortune_chance(m::tuning());
    if let Some(i) = m::fortune_target(loot.len(), chance, m::fortune_roll(pos)) {
        loot[i].count = loot[i].count.saturating_add(1);
    }
    loot
}
