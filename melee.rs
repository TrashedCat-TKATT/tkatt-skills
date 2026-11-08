// Melee damage: scale what `Player.attack` computed by the melee tree. The
// final `amount` store is modified, so the multiplier lands on the number the
// game hands to the target, after the critical hit and the enchantment bonus.
fn skill_trees_melee(game: &crate::game::Game<'_>, value: f32) -> f32 {
    use crate::game::tkatt_skills as m;
    if !game.native_on(m::ID) {
        return value;
    }
    value * m::current().melee_multiplier(m::tuning())
}
