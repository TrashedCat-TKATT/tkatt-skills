// Melee damage: the player's weapon hit on a mob, scaled by the melee tree.
// The hook is the server's own damage event, so the number the target loses is
// scaled where the game applies it, the way a server-side mod would. The HUD
// and the screen stay on each client.
fn skill_trees_melee(
    game: &crate::game::Game<'_>,
    causing: i32,
    amount: &mut f32,
) {
    use crate::game::tkatt_skills as m;
    if !game.native_on(m::ID) {
        return;
    }
    // The hook hears every player's hit, and the skills belong to this game's
    // own player: in single player that is everyone, with LAN guests only the
    // host's own hits are scaled and a guest's stay vanilla.
    let guests = game.native_server_players();
    if !guests.is_empty() {
        let Some(acting) = game.native_acting_player() else {
            return;
        };
        if acting.entity_id != causing || guests.iter().any(|g| g.uuid == acting.uuid) {
            return;
        }
    }
    // The hit is resolved on the game's server thread, where the screen's load
    // has not happened: read the state here (once).
    let skills = m::load_once(|key| {
        game.native_setting(m::ID, key)
            .or_else(|| game.native_setting(m::LEGACY_ID, key))
    });
    *amount *= skills.melee_multiplier(m::tuning());
}
