// Fortune: while the big skill is on, some broken blocks drop one more of the
// first item they already drop. The roll and the choice of stack are
// `fortune_roll` and `fortune_target` in runtime.rs, where they are
// unit-tested; this helper only applies the result to the game's loot list, in
// the server's own drop event, so the extra item is a normal drop.
fn skill_trees_fortune(
    game: &crate::game::Game<'_>,
    context: &horizon_client::mod_api::NativeLootDrops<'_>,
    drops: &mut Vec<horizon_client::items::ItemStack>,
) {
    use crate::game::tkatt_skills as m;
    // Only a broken block's own table is ours: mob drops, chests and a player's
    // inventory are left alone.
    let Some((pos, _state)) = context.block else {
        return;
    };
    if !context.table.starts_with("minecraft:blocks/") || drops.is_empty() {
        return;
    }
    if !game.native_on(m::ID) {
        return;
    }
    // The drops are rolled on the game's server thread, where the screen's
    // load has not happened: read the state here (once).
    let skills = m::load_once(|key| {
        game.native_setting(m::ID, key)
            .or_else(|| game.native_setting(m::LEGACY_ID, key))
    });
    if !skills.fortune_on() {
        return;
    }
    let chance = skills.fortune_chance(m::tuning());
    if let Some(i) = m::fortune_target(drops.len(), chance, m::fortune_roll(pos)) {
        drops[i].count = drops[i].count.saturating_add(1);
    }
}
