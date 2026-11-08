// Melee damage: scale what `Player.attack` computed by the melee tree. The
// tuple is (amount, attack scale, critical, magic).
fn skill_trees_melee(
    game: &mut crate::game::Game<'_>,
    value: (f32, f32, bool, bool),
) -> (f32, f32, bool, bool) {
    use crate::game::tkatt_skills as m;
    let (amount, scale, crit, magic) = value;
    if !game.native_on(m::ID) {
        return (amount, scale, crit, magic);
    }
    (amount * m::current().melee_multiplier(m::tuning()), scale, crit, magic)
}
