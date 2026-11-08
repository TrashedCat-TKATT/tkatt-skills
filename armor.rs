// Damage the player takes: the armour tree and the Bulwark ranks cut their
// shares of the amount, together never more than half and never down to zero,
// before the health component sees it, so blocking and knockback still follow
// vanilla on the reduced number.
fn skill_trees_armor(value: f32) -> f32 {
    use crate::game::tkatt_skills as m;
    if !m::enabled() {
        return value;
    }
    m::current().incoming_damage(m::tuning(), value)
}
