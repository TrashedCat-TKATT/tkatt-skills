// Haste: breaking a block with the big skill on gives the player a Haste
// effect at the rank's duration and strength from the stored tuning, the same
// way a beacon or a potion would, so the game's own effect code applies it and
// every other mod sees a normal effect.
fn skill_trees_haste(game: &mut crate::game::Game<'_>) {
    use crate::game::tkatt_skills as m;
    if !game.native_on(m::ID) {
        return;
    }
    let skills = m::current();
    if !skills.haste_on() {
        return;
    }
    let tune = m::tuning();
    game.add_player_effect(horizon_client::effects::EffectInstance::new(
        "haste",
        skills.haste_ticks(tune),
        skills.haste_amp(tune),
    ));
}
